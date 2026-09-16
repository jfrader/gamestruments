use std::collections::HashMap;

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{MusicEvent, PortableScore, PortableSection};
use crate::suspense::{generate_suspense, SuspenseInput};
use crate::suspense_pool::{
    all_phases_form, compose, figure_for_composition, figure_spec, phase_spec, take_seed,
    FigureSpec, Intent, PhaseRole, PhaseSpec, FIGURE_POOL, PHASE_POOL,
};

/// The authority for the Suspense recipe is the phase pool. The frozen
/// Original/Extended/Theme presets are retired; their legacy names still parse,
/// but they resolve to the pool default rather than to a byte-frozen score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SuspenseArrangement {
    /// Every pool phase once, in canonical order.
    AllPhases,
    /// The seeded composer chooses a form over the pool.
    #[default]
    Seeded,
}

impl SuspenseArrangement {
    /// Legacy frozen-preset names (`""`, `original`, `extended`, `theme`) all
    /// resolve to the pool default, so old callers keep generating a song
    /// instead of an error. Anything unknown is still rejected.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" | "extended" | "theme" | "seeded" => Ok(Self::Seeded),
            "all-phases" => Ok(Self::AllPhases),
            other => Err(format!("Unknown suspense arrangement: {other}")),
        }
    }
}

pub fn generate_suspense_arrangement(
    input: &SuspenseInput,
    arrangement: SuspenseArrangement,
) -> Result<PortableScore, String> {
    generate_suspense_arrangement_intent(input, arrangement, Intent::Arc)
}

pub fn generate_suspense_arrangement_intent(
    input: &SuspenseInput,
    arrangement: SuspenseArrangement,
    intent: Intent,
) -> Result<PortableScore, String> {
    generate_suspense_arrangement_take(input, arrangement, intent, 0)
}

/// Entry point that also carries a version (take) index. Index 0 is the first
/// version of the family, not a separate piece: every index derives one take
/// seed from the same `suspense-reel-material-1` domain, so `Versión 1..N` is
/// contiguous and each version is a distinct performance of the same piece.
pub fn generate_suspense_arrangement_take(
    input: &SuspenseInput,
    arrangement: SuspenseArrangement,
    intent: Intent,
    reel_index: u32,
) -> Result<PortableScore, String> {
    // One take seed drives the material, the figures and the surface, so the
    // reel produces a different song rather than the same song in a new order.
    let take = take_seed(
        &input.secret,
        &input.seed,
        "suspense-reel-material-1",
        reel_index,
    );
    match arrangement {
        SuspenseArrangement::AllPhases => generate_all_phases(input, take),
        SuspenseArrangement::Seeded => generate_seeded(input, intent, take),
    }
}

/// The material domain is keyed by the take seed alone, so the first version is
/// part of the same family as every later one (it used to fall back to a
/// `suspense-pool-phases-1` domain of its own, which made "Versión 1" a
/// different derivation instead of the start of the reel).
///
/// The take drives the MATERIAL, not only the form: deriving the pool from the
/// level seed alone left every version with the same drum break, the same fills
/// and the same figures. Shared with the transition pass so the joins belong to
/// the same take as the sections.
fn pool_seed(input: &SuspenseInput, take: u32) -> u32 {
    hash_text(&format!(
        "{}\0{}\0suspense-pool-phases-2\0{take}",
        input.secret, input.seed
    ))
}

/// Build the full section pool (the 14 base sections, the `scan-ii` /
/// `breach-ii` / `anomaly` phases, and the pool-authored phases) in
/// canonical order. Each section is developed to its authored length with a
/// harmonic/melodic progression; the form is chosen afterwards, over this
/// pool, by [`generate_all_phases`] and [`generate_seeded`].
fn build_pool_score(input: &SuspenseInput, take: u32) -> Result<PortableScore, String> {
    let mut score = generate_suspense(input)?;
    let root = arrangement_root(&score)?;
    let bar = score.bar_ticks();
    let seed = pool_seed(input, take);

    let base_sections = std::mem::take(&mut score.sections);
    let mut by_id: HashMap<String, PortableSection> = HashMap::new();
    for mut section in base_sections {
        if let Some(spec) = phase_spec(&section.id) {
            develop_phase(&mut section, root, bar, seed, spec, take);
        }
        by_id.insert(section.id.clone(), section);
    }
    for (id, label) in [("scan-ii", "Scan II"), ("breach-ii", "Breach II")] {
        let section = variation(id, label, root, bar, seed, arc_entry(seed));
        by_id.insert(id.into(), section);
    }
    let anomaly_section = anomaly(root, bar, seed, arc_entry(seed));
    by_id.insert("anomaly".into(), anomaly_section);
    for (id, section) in build_new_phases(root, bar, seed) {
        by_id.insert(id, section);
    }

    // Assemble in canonical pool order so the lab's section list reads like a
    // song. Every built section must be present in the pool.
    score.sections = PHASE_POOL
        .iter()
        .filter_map(|spec| by_id.remove(spec.id))
        .collect();
    debug_assert!(by_id.is_empty(), "built a section absent from the pool");
    for section in &mut score.sections {
        normalize_pool_percussion(section, bar);
        tilt_high_register(section, root);
        strip_confirmed_glass_cell(section);
        anchor_phase_edges(section, root, bar);
    }
    Ok(score)
}

/// The confirmed sustained glass-cell beep was removed from Decrypt, Other Hall
/// and Full Breach. The old Extended preset dropped it while building, and the
/// pool develops the base sections, so the pool has to drop the same beep
/// explicitly — retiring the preset must not resurrect it.
fn strip_confirmed_glass_cell(section: &mut PortableSection) {
    if !matches!(section.id.as_str(), "solo" | "bridge-b" | "chorus-final") {
        return;
    }
    let cell_lane = format!("{}-cell", section.id);
    section.events.retain(|event| {
        !matches!(
            event,
            MusicEvent::Note { voice, lane, .. } if voice == "glass" && lane == &cell_lane
        )
    });
}

/// The layer a phase's material belongs to, from the bed up to the colour.
/// 0 bed · 1 tick · 2 rhythm · 3 melodic cell · 4 arp/upper colour.
fn layer_rank(event: &MusicEvent) -> u8 {
    match event {
        MusicEvent::Note { lane, .. } => {
            if lane.ends_with("-drone") || lane.ends_with("-drone-upper") {
                0
            } else if lane.ends_with("-pulse") {
                1
            } else if lane.ends_with("-cell") {
                3
            } else if lane.ends_with("-arp") {
                4
            } else {
                2
            }
        }
        MusicEvent::Percussion { .. } => 2,
    }
}

/// The musical pass (GURI-789): a phase stops holding one texture for its whole
/// length. Layers enter and leave across its 4-bar blocks, a material entry
/// lands on an impact, and the closing bar is left intact so the seam still
/// resolves. That is Mr. Robot's additive/subtractive layering plus
/// Santaolalla's development by reduction, expressed only with the events the
/// score already carries.
///
/// The shape follows the phase's role, so the pool does not breathe in lockstep.
/// The bed (rank 0) is never masked, so the drone stays continuous.
fn apply_development_arc(section: &mut PortableSection, bar: u32, _seed: u32) {
    if bar == 0 {
        return;
    }
    let Some(spec) = phase_spec(&section.id) else {
        return;
    };
    // A break or a wait state is the drop itself; it carries its own shape.
    if is_break_or_wait(spec) {
        return;
    }
    let bars = section.length_ticks / bar;
    let blocks = (bars / 4) as usize;
    if blocks < 2 {
        return;
    }
    let id = section.id.clone();
    // The shape follows the job the phase does, so the pool does not breathe in
    // lockstep. Rank 1 is bed+tick only, 2 adds the kit, 3 the cell, 4 the arp.
    //   Peak / Build  climb out of a sparse exposition to a full peak
    //   Groove / Loop keep the kit and develop the melodic layers
    //   Bridge        stays light in the middle
    //   Intro / Outro grow out of the bed and settle back
    let arc: &[u8] = match spec.role {
        PhaseRole::Peak | PhaseRole::Build => &[1, 2, 3, 4],
        PhaseRole::Groove | PhaseRole::Loop => &[2, 3, 4, 3],
        PhaseRole::Bridge => &[3, 2, 3, 4],
        PhaseRole::Intro | PhaseRole::Outro => &[1, 2, 4, 3],
        PhaseRole::Break => return,
    };
    let schedule: Vec<u8> = (0..blocks)
        .map(|block| arc[(block * arc.len()) / blocks])
        .collect();
    let block_ticks = bar * 4;
    let last_bar_start = section.length_ticks.saturating_sub(bar);
    section.events.retain(|event| {
        let start = event.start_tick();
        // The closing bar is the release: it stays intact so the phase still
        // lands on the seam the shared arc expects.
        if start >= last_bar_start {
            return true;
        }
        let block = (start / block_ticks) as usize;
        layer_rank(event) <= schedule.get(block).copied().unwrap_or(4)
    });
    // Every time the arc adds a layer, the block lands on an impact. Ids get
    // their own prefix so they never collide with the kit's `:kit:dev:` onsets.
    for block in 0..blocks.saturating_sub(1) {
        if schedule[block + 1] > schedule[block] && schedule[block + 1] >= 3 {
            let start = (block as u32 + 1) * block_ticks;
            section.events.push(MusicEvent::Percussion {
                id: format!("{id}:kit:arc:{block}"),
                section: id.clone(),
                lane: format!("{id}-kit"),
                start_tick: start,
                duration_ticks: (bar / 2).max(1),
                velocity: 0.4,
                voice: "air-impact".to_string(),
            });
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn generate_all_phases(input: &SuspenseInput, take: u32) -> Result<PortableScore, String> {
    let mut score = build_pool_score(input, take)?;
    score.form = Some(all_phases_form());
    apply_surface_variation(&mut score, input, take);
    apply_development_pass(&mut score, pool_seed(input, take))?;
    apply_transition_pass(&mut score, pool_seed(input, take));
    score.id.push_str("-all-phases");
    score.title.push_str(" — All phases");
    score.validate()?;
    Ok(score)
}

fn generate_seeded(
    input: &SuspenseInput,
    intent: Intent,
    take: u32,
) -> Result<PortableScore, String> {
    // The form comes from the same take seed as the material, so every version
    // is a different piece arrangement, not a reordering of the same one.
    let form_seed = take;
    let mut score = build_pool_score(input, take)?;
    score.form = Some(compose(form_seed, intent));
    apply_surface_variation(&mut score, input, take);
    apply_development_pass(&mut score, pool_seed(input, take))?;
    apply_transition_pass(&mut score, pool_seed(input, take));
    score.id.push_str(&format!("-seeded-{}", intent.as_str()));
    score.title.push_str(" — Seeded");
    score.validate()?;
    Ok(score)
}

/// The seam vocabulary: how one phase hands the music to the next.
///
/// A pool section is reusable and built on its own, so the treatment cannot live
/// in the section: it is planned from the chosen form, join by join, from the
/// take seed. The gesture, its length and its carrier all follow the seed and
/// the two phases it connects, so no two unions are treated the same way and the
/// piece does not hand off identically every time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeamGesture {
    /// Percussion run rising into the downbeat.
    Fill,
    /// Reverse-cymbal swell across the join.
    Riser,
    /// Short melodic pickup from the set's own arc.
    Lift,
    /// A swell plus a melodic pickup: both carriers.
    Both,
    /// A long quiet low note: the bed fades out instead of hitting.
    Tail,
}

fn seam_lane(event: &MusicEvent) -> &str {
    match event {
        MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. } => lane,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_seam_note(
    events: &mut Vec<MusicEvent>,
    serial: &mut usize,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    pitch: u8,
    voice: &str,
) {
    let index = *serial;
    *serial += 1;
    events.push(MusicEvent::Note {
        id: format!("{section}:seam-note:{index}"),
        section: section.to_string(),
        lane: format!("{section}-seam"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.4),
        pitch,
        voice: voice.to_string(),
        role: None,
    });
}

fn push_seam_perc(
    events: &mut Vec<MusicEvent>,
    serial: &mut usize,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    voice: &str,
) {
    let index = *serial;
    *serial += 1;
    events.push(MusicEvent::Percussion {
        id: format!("{section}:seam-perc:{index}"),
        section: section.to_string(),
        lane: format!("{section}-seam"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.3),
        voice: voice.to_string(),
    });
}

/// Plan every join of the chosen form. Runs after the form exists because the
/// treatment belongs to the *pair*, not to a reusable section.
fn apply_transition_pass(score: &mut PortableScore, seed: u32) {
    let bar = score.bar_ticks();
    if bar == 0 {
        return;
    }
    let Some(form) = score.form.clone() else {
        return;
    };
    let Ok(root) = arrangement_root(score) else {
        return;
    };
    let entry = arc_entry(seed);
    let mut pairs: Vec<(String, String)> = form
        .steps
        .windows(2)
        .map(|window| (window[0].section.clone(), window[1].section.clone()))
        .collect();
    if let (Some(loop_from), Some(last)) = (form.loop_from, form.steps.last()) {
        if let Some(first) = form.steps.get(loop_from as usize) {
            pairs.push((last.section.clone(), first.section.clone()));
        }
    }
    // A section can appear in more than one join; the last planned treatment
    // wins, so clear the previous one first.
    for section in &mut score.sections {
        section
            .events
            .retain(|event| !seam_lane(event).ends_with("-seam"));
    }
    let mut serial = 0usize;
    for (out_id, in_id) in pairs {
        let out_role = phase_spec(&out_id).map(|spec| spec.role);
        let in_role = phase_spec(&in_id).map(|spec| spec.role);
        let mut rng =
            DeterministicRandom::new(seed ^ hash_text(&format!("{out_id}>{in_id}:seam")));
        // Different unions get different resources: what is arriving and what is
        // leaving steer the vocabulary.
        let choices: &[SeamGesture] = if in_role == Some(PhaseRole::Peak) {
            &[SeamGesture::Fill, SeamGesture::Riser, SeamGesture::Both]
        } else if in_role == Some(PhaseRole::Break) {
            &[SeamGesture::Tail, SeamGesture::Riser]
        } else if out_role == Some(PhaseRole::Break) {
            &[SeamGesture::Riser, SeamGesture::Tail]
        } else {
            &[
                SeamGesture::Fill,
                SeamGesture::Riser,
                SeamGesture::Lift,
                SeamGesture::Both,
                SeamGesture::Tail,
            ]
        };
        let gesture = choices[rng.integer(choices.len() as u32) as usize];
        // Long, short or very short: the join decides.
        let span = match rng.integer(3) {
            0 => bar,
            1 => bar * 2,
            _ => bar / 2,
        };
        let landing = rng.integer(3);
        let Some(out_len) = score.section(&out_id).map(|section| section.length_ticks) else {
            continue;
        };
        let out_span = span.min(out_len).max(bar / 4);
        let out_start = out_len.saturating_sub(out_span);
        let mut out_events: Vec<MusicEvent> = Vec::new();
        let mut in_events: Vec<MusicEvent> = Vec::new();
        match gesture {
            SeamGesture::Fill => {
                for (index, step) in [8u32, 10, 12, 14, 15].iter().enumerate() {
                    let offset = (step * bar / 16).min(out_span.saturating_sub(bar / 16));
                    push_seam_perc(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 16,
                        0.12 + 0.015 * index as f64,
                        "tom",
                    );
                }
            }
            SeamGesture::Riser => {
                push_seam_perc(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.12,
                    "reverse-cymbal",
                );
            }
            SeamGesture::Lift => {
                for index in 0..3usize {
                    let degree = PROGRESSION_ARC[(index + entry) % PROGRESSION_ARC.len()];
                    let offset = (index as u32 * out_span / 3)
                        .min(out_span.saturating_sub(bar / 4));
                    push_seam_note(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 4,
                        0.12,
                        aeolian(root, degree),
                        "dusk",
                    );
                }
            }
            SeamGesture::Both => {
                push_seam_perc(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.12,
                    "reverse-cymbal",
                );
                for index in 0..3usize {
                    let degree = PROGRESSION_ARC[(index + entry) % PROGRESSION_ARC.len()];
                    let offset = (index as u32 * out_span / 3)
                        .min(out_span.saturating_sub(bar / 4));
                    push_seam_note(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 4,
                        0.12,
                        aeolian(root, degree),
                        "pulse",
                    );
                }
            }
            SeamGesture::Tail => {
                push_seam_note(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.1,
                    root,
                    "warm",
                );
            }
        }
        match landing {
            1 => push_seam_perc(
                &mut in_events,
                &mut serial,
                &in_id,
                0,
                bar / 8,
                0.12,
                "air-impact",
            ),
            2 => push_seam_perc(&mut in_events, &mut serial, &in_id, 0, bar / 8, 0.22, "kick"),
            _ => {}
        }
        if !out_events.is_empty() {
            if let Some(section) = score.sections.iter_mut().find(|s| s.id == out_id) {
                section.events.extend(out_events);
                section.events.sort_by_key(MusicEvent::start_tick);
            }
        }
        if !in_events.is_empty() {
            if let Some(section) = score.sections.iter_mut().find(|s| s.id == in_id) {
                section.events.extend(in_events);
                section.events.sort_by_key(MusicEvent::start_tick);
            }
        }
    }
}

/// Re-time a composed score's surface without touching its harmony, timbre, or
/// Run the development arc over the whole pool. It runs after the surface pass
/// so the ghost hats land on the arranged material instead of refilling the
/// blocks the arc just emptied, and the edge anchors are re-applied afterwards
/// so a masked first/last bar still carries the shared root.
fn apply_development_pass(score: &mut PortableScore, seed: u32) -> Result<(), String> {
    let bar = score.bar_ticks();
    let root = arrangement_root(score)?;
    for section in &mut score.sections {
        apply_development_arc(section, bar, seed);
    }
    for section in &mut score.sections {
        anchor_phase_edges(section, root, bar);
    }
    Ok(())
}

/// identity: seeded hat drops, ghost hats, velocity jitter (figures now drive
/// base rhythm onsets). The reel index (when non-zero) derives its own surface
/// domain so each take gets a distinct surface too.
fn apply_surface_variation(score: &mut PortableScore, input: &SuspenseInput, take: u32) {
    let bar = score.bar_ticks();
    // The take owns the surface as well, so a version is a different
    // performance rather than a reordering of the same one.
    let seed = hash_text(&format!(
        "{}\0{}\0suspense-surface-2\0{take}",
        input.secret, input.seed
    ));
    for section in &mut score.sections {
        if matches!(
            section.id.as_str(),
            "intro"
                | "break"
                | "drum-break"
                | "false-stop"
                | "filter-break"
                | "sparse"
                | "outro"
                | "coda"
        ) {
            continue;
        }
        let mut random =
            DeterministicRandom::new(seed ^ hash_text(&format!("{}:surface", section.id)));

        // (a) Drop a hat ~1 in 4 (never a downbeat kick), jitter velocities,
        // then add low-velocity ghost hats on empty 16ths.
        let mut drop_hats = vec![false; section.events.len()];
        for (index, event) in section.events.iter_mut().enumerate() {
            let MusicEvent::Percussion {
                voice, velocity, ..
            } = event
            else {
                continue;
            };
            match voice.as_str() {
                "hat" => {
                    if random.integer(4) == 0 {
                        drop_hats[index] = true;
                    }
                    *velocity = (*velocity + random.next() * 0.04 - 0.02).clamp(0.08, 0.55);
                }
                "kick" | "tom" => {
                    *velocity = (*velocity + random.next() * 0.04 - 0.02).clamp(0.08, 0.55);
                }
                _ => {}
            }
        }
        let mut kept = Vec::with_capacity(section.events.len());
        for (event, dropped) in section.events.drain(..).zip(drop_hats) {
            if !dropped {
                kept.push(event);
            }
        }
        section.events = kept;

        let sixteenth = bar / 16;
        let mut ghosts = Vec::new();
        for start in (0..section.length_ticks).step_by(sixteenth as usize) {
            if section.events.iter().any(|event| {
                matches!(event, MusicEvent::Percussion { start_tick, .. } if *start_tick == start)
            }) {
                continue;
            }
            if random.integer(6) == 0 {
                ghosts.push(start);
            }
        }
        for start in ghosts {
            section.events.push(MusicEvent::Percussion {
                id: format!("{}:surface:ghost:{start}", section.id),
                section: section.id.clone(),
                lane: format!("{}-kit", section.id),
                start_tick: start,
                duration_ticks: sixteenth / 2,
                velocity: 0.09,
                voice: "hat".into(),
            });
        }

        section.events.sort_by_key(MusicEvent::start_tick);
    }
}

// ---------------------------------------------------------------------------
// Pool-phase development (GURI-789) and the new phase authors (GURI-787).
//
// Everything below runs only in the pool path, over the sections
// `generate_suspense` produces as the recipe's phase identity.
// ---------------------------------------------------------------------------

/// The musical identity of a base section, read from the events `generate_suspense`
/// produced so `develop_phase` can rebuild it with progression and breath.
struct PhaseIdentity {
    /// Low tension-anchor drone: (pitch, voice, velocity).
    drone: Option<(u8, String, f64)>,
    /// The higher fifth drone some Drive/Alarm sections carry.
    drone_fifth: Option<(u8, String, f64)>,
    /// Melodic cell: (voice, base velocity).
    cell: Option<(String, f64)>,
    /// Harmonic pulse voice.
    pulse: Option<String>,
    /// Arpeggio voice.
    arp: Option<String>,
}

impl PhaseIdentity {
    fn read(section: &PortableSection, root: u8) -> Self {
        let mut drone = None;
        let mut drone_fifth = None;
        let mut cell = None;
        let mut pulse = None;
        let mut arp = None;
        for event in &section.events {
            let MusicEvent::Note {
                lane,
                pitch,
                voice,
                velocity,
                ..
            } = event
            else {
                continue;
            };
            if lane.ends_with("-drone") {
                if *pitch <= root {
                    drone.get_or_insert_with(|| (*pitch, voice.clone(), *velocity));
                } else {
                    drone_fifth.get_or_insert_with(|| (*pitch, voice.clone(), *velocity));
                }
            } else if lane.ends_with("-pulse") {
                pulse.get_or_insert_with(|| voice.clone());
            } else if lane.ends_with("-arp") {
                arp.get_or_insert_with(|| voice.clone());
            } else if lane.ends_with("-cell") {
                cell.get_or_insert_with(|| (voice.clone(), *velocity));
            }
        }
        Self {
            drone,
            drone_fifth,
            cell,
            pulse,
            arp,
        }
    }
}

/// A fixed i → VI → iv → VII → i arc in Aeolian scale degrees.
///
/// One arc entry per take: `arc_entry` derives a single rotation from the take
/// seed, so every phase in the set walks the same progression instead of
/// rolling its own. Sixteen-bar phases still advance the arc for their second
/// half, so bar 1 is never bar 8.
///
/// Seam contract: every phase opens on the tonic. A resolving phase (Outro)
/// also closes on the tonic, but every other phase hands off on the bVII (the
/// approach chord), so a join is a modal bVII → i cadence rather than a reset.
/// The shared tonic drone keeps both edges sharing a pitch class even when the
/// harmony leans.
const PROGRESSION_ARC: [i32; 8] = [0, 0, 5, 5, 3, 3, 6, 0];

/// The arc entry for the whole take. A pure function of the take seed, so the
/// pool authors agree on one entry without threading a parameter through every
/// builder.
fn arc_entry(seed: u32) -> usize {
    (hash_text(&format!("{seed}:arc-entry")) as usize) % PROGRESSION_ARC.len()
}

fn progression_degrees(bars: u32, seed: u32, resolve: bool) -> Vec<i32> {
    let entry = arc_entry(seed);
    let mut degrees: Vec<i32> = (0..bars)
        .map(|bar| {
            let half = if bars >= 16 {
                (bar / 8) as usize * 2
            } else {
                0
            };
            PROGRESSION_ARC[(bar as usize + entry + half) % PROGRESSION_ARC.len()]
        })
        .collect();
    if let Some(first) = degrees.first_mut() {
        *first = 0;
    }
    if bars >= 2 {
        if let Some(last) = degrees.last_mut() {
            *last = if resolve { 0 } else { 6 };
        }
    }
    degrees
}

/// A pure-arithmetic density/velocity arc: rises to a peak just past the middle
/// and releases. No transcendental functions, so native and wasm agree exactly.
fn arc_factor(bar_index: u32, bars: u32) -> f64 {
    if bars <= 1 {
        return 1.0;
    }
    let peak = f64::from(bars) * 0.62;
    let position = f64::from(bar_index);
    if position <= peak {
        0.75 + 0.35 * (position / peak)
    } else {
        1.1 - 0.35 * ((position - peak) / (f64::from(bars) - peak))
    }
}

fn clamp_pitch(pitch: i32) -> u8 {
    pitch.clamp(28, 91) as u8
}

fn cell_root_for(id: &str, root: u8) -> u8 {
    match id {
        "solo" => root + 12,
        "chorus-final" => root + 7,
        _ => root,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_dev_note(
    events: &mut Vec<MusicEvent>,
    section: &str,
    lane_suffix: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    pitch: u8,
    voice: &str,
    melody: bool,
) {
    let index = events.len();
    events.push(MusicEvent::Note {
        id: format!("{section}:{lane_suffix}:dev:{index}"),
        section: section.to_string(),
        lane: format!("{section}-{lane_suffix}"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.72),
        pitch,
        voice: voice.to_string(),
        role: if melody {
            Some("melody".to_string())
        } else {
            None
        },
    });
}

fn push_dev_perc(
    events: &mut Vec<MusicEvent>,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    voice: &str,
) {
    let index = events.len();
    events.push(MusicEvent::Percussion {
        id: format!("{section}:kit:dev:{index}"),
        section: section.to_string(),
        lane: format!("{section}-kit"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.55),
        voice: voice.to_string(),
    });
}

fn new_phase_section(
    id: &str,
    label: &str,
    feeling: &str,
    color: &str,
    length_ticks: u32,
) -> PortableSection {
    PortableSection {
        id: id.into(),
        label: label.into(),
        feeling: feeling.into(),
        color: color.into(),
        length_ticks,
        events: Vec::new(),
    }
}

/// The places the piece catches its breath: the Break-role phases plus the two
/// low-energy wait states the pool authors as grooves.
fn is_break_or_wait(spec: &PhaseSpec) -> bool {
    spec.role == PhaseRole::Break || matches!(spec.id, "sparse" | "interlude")
}

/// A phase that carries momentum, so a portion of its takes may stretch past
/// the authored length. Adventure and Racing both let their sections run long;
/// without this, every strong Suspense phase resolved at 16 bars.
fn is_momentum(spec: &PhaseSpec) -> bool {
    matches!(
        spec.role,
        PhaseRole::Groove | PhaseRole::Peak | PhaseRole::Build | PhaseRole::Bridge
    ) && spec.energy >= 45
}

/// The take decides how long a phase plays: very short, short, as authored, or
/// long. Breaks and waits may shrink to one or two bars so the piece can turn
/// around quickly, and momentum phases may stretch into the 24/32-bar range the
/// other recipes already use. The authored bar count is a starting point, not a
/// fixed rendering, so the piece breathes instead of every phase using its
/// written length.
fn phase_bars(spec: &PhaseSpec, seed: u32) -> u32 {
    let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:bars", spec.id)));
    if is_break_or_wait(spec) {
        // A portion of the breaks/waits become one or two bars; the rest keep
        // their authored length.
        return match rng.integer(3) {
            0 => 1,
            1 => 2,
            _ => spec.bars.max(2),
        };
    }
    let authored = spec.bars.max(4);
    let bars = match rng.integer(4) {
        0 => 4,
        1 => (authored / 2).max(4),
        2 => authored,
        _ => {
            let doubled = authored * 2;
            if is_momentum(spec) {
                doubled.clamp(16, 32)
            } else {
                doubled.min(16)
            }
        }
    };
    // Every arc phase needs at least two 4-bar blocks for its development to be
    // audible; breaks and waits (which returned above) keep their 1-2 bars.
    bars.max(8)
}

/// Rebuild a base section so it develops across its bars: the low drone is
/// re-articulated into two-bar swells (with an occasional deliberate gap) while
/// the harmony moves through the progression, the melodic cell phrases develop
/// (statement / answer / sequence / release), and the drums enter, arc, and fill.
fn develop_phase(
    section: &mut PortableSection,
    root: u8,
    bar: u32,
    seed: u32,
    spec: &PhaseSpec,
    figure_seed: u32,
) {
    let id = section.id.clone();
    let identity = PhaseIdentity::read(section, root);
    let bars = phase_bars(spec, seed);
    let degrees = progression_degrees(bars, seed, spec.role == PhaseRole::Outro);
    let figure = figure_for_composition(spec.id, figure_seed);
    let mut events = Vec::new();

    // The bed has to be able to leave and come back. A low pedal running under
    // every phase without ever stopping is what Fran heard as a sound that
    // "never transitions to anything": the breaks and the space-oriented phases
    // now drop it, and every other phase scales it with its own energy.
    let bed = !matches!(spec.role, PhaseRole::Break) && !matches!(spec.id, "sparse" | "interlude");
    if bed {
        if let Some(drone) = identity.drone {
            let level = 0.75 + f64::from(spec.energy.min(100)) / 250.0;
            develop_drone(
                &mut events,
                &id,
                (drone.0, drone.1, drone.2 * level),
                bar,
                bars,
                "drone",
                0,
            );
        }
        if let Some(drone_fifth) = identity.drone_fifth {
            develop_drone(
                &mut events,
                &id,
                drone_fifth,
                bar,
                bars,
                "drone-upper",
                2,
            );
        }
    }
    if let Some(pulse) = identity.pulse {
        develop_pulse(
            &mut events,
            &id,
            &pulse,
            root,
            bar,
            bars,
            &degrees,
            figure,
        );
    }
    if let Some(arp) = identity.arp {
        develop_arp(
            &mut events,
            &id,
            &arp,
            root,
            bar,
            bars,
            &degrees,
            figure,
        );
    }
    if let Some(cell) = identity.cell {
        let cell_root = cell_root_for(&id, root);
        develop_cell(
            &mut events,
            &id,
            cell,
            cell_root,
            bar,
            bars,
            &degrees,
            spec.role,
            figure,
        );
    }
    develop_drums(
        &mut events,
        &id,
        bar,
        bars,
        spec.role,
        spec.energy,
        seed,
        figure,
    );

    section.length_ticks = bars * bar;
    section.events = events;
    section.events.sort_by_key(MusicEvent::start_tick);
}

/// Hold a drone voice continuously across the whole phase.
///
/// Ambient and drone practice never re-attack or gap a sustain: cutting it into
/// swells turns the bed into an event and destroys the mood. Progression comes
/// from *layering*, so the upper layer fades in and out by presence
/// (`inset_bars`) while the base bed stays unbroken.
fn develop_drone(
    events: &mut Vec<MusicEvent>,
    id: &str,
    drone: (u8, String, f64),
    bar: u32,
    bars: u32,
    lane_suffix: &str,
    inset_bars: u32,
) {
    let (pitch, voice, base_vel) = drone;
    let total = bars.max(1) * bar;
    let inset = inset_bars.min(bars.saturating_sub(1)) * bar;
    let start = inset;
    let end = total.saturating_sub(inset).max(start + bar / 2);
    push_dev_note(
        events,
        id,
        lane_suffix,
        start,
        end - start,
        base_vel.clamp(0.08, 0.72),
        pitch,
        &voice,
        false,
    );
}

fn bar_has_pitch_class(section: &PortableSection, start: u32, end: u32, pitch_class: u8) -> bool {
    section.events.iter().any(|event| match event {
        MusicEvent::Note {
            start_tick,
            duration_ticks,
            pitch,
            ..
        } => {
            *start_tick < end
                && start_tick + duration_ticks > start
                && pitch % 12 == pitch_class % 12
        }
        _ => false,
    })
}

/// Tilt the pool's own velocities so the mid/high material does not sit as loud
/// as the low end. Delicate and local: it never touches the presets, the synth
/// or the master chain, and the low pedal keeps exactly the level it had.
fn tilt_high_register(section: &mut PortableSection, root: u8) {
    let pivot = i32::from(root) + 10;
    for event in &mut section.events {
        if let MusicEvent::Note {
            pitch, velocity, ..
        } = event
        {
            let above = i32::from(*pitch) - pivot;
            if above > 0 {
                let steps = (above / 3).min(4) as f64;
                *velocity *= 1.0 - 0.06 * steps;
            }
        }
    }
}

/// Normalise pool percussion to the approved recipe's balance.
///
/// The base recipe accents only the entrance bar (kick 0.42) and then sits the
/// downbeat under the melody at 0.28; hats are 0.16. The pool's own builders
/// each authored their own kit, so this is the single place that guarantees the
/// whole pool mixes like the recipe Fran approved.
fn normalize_pool_percussion(section: &mut PortableSection, bar: u32) {
    if bar == 0 {
        return;
    }
    for event in &mut section.events {
        if let MusicEvent::Percussion {
            voice,
            velocity,
            start_tick,
            ..
        } = event
        {
            let ceiling = match voice.as_str() {
                "kick" if *start_tick % bar == 0 && *start_tick > 0 => 0.30,
                "kick" => 0.42,
                "hat" => 0.16,
                "tom" => 0.30,
                _ => 0.42,
            };
            if *velocity > ceiling {
                *velocity = ceiling;
            }
        }
    }
}

/// Give every phase a **tonic anchor** in its first and last bar.
///
/// Across this repertoire the join between sections is timbral over a shared
/// pedal, not harmonic (Eno, Lustmord, No Man's Sky). Anchoring the edges means
/// any legal join shares a pitch class, so the composer never has to measure
/// harmony at the seam: the material is coherent by construction.
fn anchor_phase_edges(section: &mut PortableSection, root: u8, bar: u32) {
    if section.length_ticks == 0 || bar == 0 {
        return;
    }
    let head_end = bar.min(section.length_ticks);
    let last_start = section.length_ticks.saturating_sub(bar);
    let mut anchors: Vec<(u32, u32)> = Vec::new();
    if !bar_has_pitch_class(section, 0, head_end, root) {
        anchors.push((0, head_end));
    }
    if last_start > 0 && !bar_has_pitch_class(section, last_start, section.length_ticks, root) {
        anchors.push((last_start, section.length_ticks - last_start));
    }
    if anchors.is_empty() {
        return;
    }
    let id = section.id.clone();
    for (index, (start, duration)) in anchors.into_iter().enumerate() {
        let voice = if index == 0 { "warm" } else { "organ" };
        push_dev_note(
            &mut section.events,
            &id,
            "anchor",
            start,
            duration,
            0.12,
            root,
            voice,
            false,
        );
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

#[allow(clippy::too_many_arguments)]
fn develop_pulse(
    events: &mut Vec<MusicEvent>,
    id: &str,
    voice: &str,
    root: u8,
    bar: u32,
    bars: u32,
    degrees: &[i32],
    figure: &FigureSpec,
) {
    let sixteenth = bar / 16;
    for bar_index in 0..bars {
        let degree = degrees[bar_index as usize % degrees.len()];
        let tonic = aeolian(root, degree);
        let fifth = aeolian(root, degree + 4);
        let arc = arc_factor(bar_index, bars);
        for &s in figure.steps {
            let start = bar_index * bar + (s as u32) * sixteenth;
            let dur = bar / (figure.subdivision as u32);
            let use_tonic = ((s as u32) / 2).is_multiple_of(2);
            let pitch = if use_tonic { tonic } else { fifth };
            push_dev_note(
                events,
                id,
                "pulse",
                start,
                dur,
                (0.22 * arc).clamp(0.08, 0.72),
                pitch,
                voice,
                false,
            );
        }
    }
}

/// Keep a voice inside the register band the approved recipe uses. Chord
/// transposition must move a figure, not drift it upward until the top notes
/// dominate the mix.
fn fold_register(pitch: u8, low: u8, high: u8) -> u8 {
    let mut folded = i32::from(pitch);
    let (low, high) = (i32::from(low), i32::from(high));
    while folded > high {
        folded -= 12;
    }
    while folded < low {
        folded += 12;
    }
    folded.clamp(28, 91) as u8
}

#[allow(clippy::too_many_arguments)]
fn develop_arp(
    events: &mut Vec<MusicEvent>,
    id: &str,
    voice: &str,
    root: u8,
    bar: u32,
    bars: u32,
    degrees: &[i32],
    figure: &FigureSpec,
) {
    let sixteenth = bar / 16;
    for bar_index in 0..bars {
        let degree = degrees[bar_index as usize % degrees.len()];
        let tones = [
            fold_register(aeolian(root, degree), root, root + 12),
            fold_register(aeolian(root, degree + 2), root, root + 12),
            fold_register(aeolian(root, degree + 4), root, root + 12),
            fold_register(aeolian(root, degree + 8), root, root + 12),
        ];
        let arc = arc_factor(bar_index, bars);
        for &s in figure.steps {
            let start = bar_index * bar + (s as u32) * sixteenth;
            let dur = bar / (figure.subdivision as u32);
            let eighth_idx = (s as u32) / 2;
            push_dev_note(
                events,
                id,
                "arp",
                start,
                dur,
                (0.18 * arc).clamp(0.08, 0.72),
                tones[(eighth_idx % 4) as usize],
                voice,
                false,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn develop_cell(
    events: &mut Vec<MusicEvent>,
    id: &str,
    cell: (String, f64),
    cell_root: u8,
    bar: u32,
    bars: u32,
    degrees: &[i32],
    role: PhaseRole,
    figure: &FigureSpec,
) {
    let (voice, base_vel) = cell;
    let pulse = bar / 8;
    let sixteenth = bar / 16;
    let onsets = figure.steps.len();
    let sparse = matches!(role, PhaseRole::Intro | PhaseRole::Outro | PhaseRole::Break);
    // Hard rules: sustained cell (>=1 beat ~2*pulse) requires low density + tolerance
    let want_sustained = !sparse;
    let mut cell_dur =
        if onsets >= 8 || (want_sustained && (onsets > 4 || figure.sustain_tolerance < 6)) {
            pulse
        } else if want_sustained {
            pulse * 3
        } else {
            pulse * 2
        };
    if onsets >= 8 {
        cell_dur = cell_dur.min(pulse);
    }
    let phrases = bars / 2;
    for phrase in 0..phrases {
        let degree = degrees[(phrase * 2) as usize % degrees.len()];
        let transpose = i32::from(aeolian(cell_root, degree)) - i32::from(cell_root);
        let intervals: [i32; 3] = match phrase % 4 {
            0 => [0, 7, -2],
            1 => [7, 0, -2],
            2 => [3, 10, 1],
            _ => [0, 3, 7],
        };
        // Use figure steps for placement instead of fixed 8-grid
        let fsteps = figure.steps;
        let step_idx = (phrase as usize) % fsteps.len().max(1);
        let base16 = fsteps[step_idx] as u32;
        let start = phrase * 2 * bar + base16 * sixteenth;
        // One gesture per phrase. The base recipe's cell is sparse by design;
        // three notes per phrase reads as a shrill stream in this register.
        let index = if sparse { 0 } else { (phrase % 3) as usize };
        let interval = if id == "bridge-b" {
            -intervals[index]
        } else {
            intervals[index]
        };
        let pitch = clamp_pitch(i32::from(cell_root) + interval + transpose);
        push_dev_note(
            events, id, "cell", start, cell_dur, base_vel, pitch, &voice, true,
        );
        if !sparse && phrase % 2 == 1 {
            let answer16 = fsteps[(step_idx + 1) % fsteps.len().max(1)] as u32;
            let answer_start = phrase * 2 * bar + answer16 * sixteenth;
            let pitch = clamp_pitch(i32::from(cell_root) + transpose + 7);
            push_dev_note(
                events,
                id,
                "answer",
                answer_start,
                pulse * 2,
                base_vel * 0.7,
                pitch,
                "dusk",
                true,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn develop_drums(
    events: &mut Vec<MusicEvent>,
    id: &str,
    bar: u32,
    bars: u32,
    role: PhaseRole,
    energy: u32,
    seed: u32,
    figure: &FigureSpec,
) {
    use PhaseRole::*;
    let mut random = DeterministicRandom::new(seed ^ hash_text(&format!("{id}:drums")));
    let pulse = bar / 8;
    let sixteenth = bar / 16;
    // Seeded groove detail: a fixed offbeat kick and a fixed closing lick in
    // every seed was the "the fill is always the same" bug.
    let offbeat = [4u32, 6, 3][random.integer(3) as usize];
    // Kit mode: the pool owns more than one kit. The full kick–snare backbeat
    // (the Theme's pattern) is one of them, chosen per phase by the seed so a
    // set is not all the same kit. Drawn from its own stream, so every phase's
    // existing groove draws stay exactly as they were.
    let mut kit_rng = DeterministicRandom::new(seed ^ hash_text(&format!("{id}:kit")));
    let backbeat = kit_rng.integer(3) == 0;
    // The backbeat needs a kick on 3 rather than the seeded offbeat.
    let offbeat = if backbeat { 4 } else { offbeat };
    match role {
        Intro | Outro => {
            push_dev_perc(events, id, 0, pulse, 0.3, "kick");
        }
        Break => {
            push_dev_perc(events, id, 0, pulse, 0.2, "kick");
        }
        _ => {
            for bar_index in 0..bars {
                let arc = arc_factor(bar_index, bars);
                // Match the approved recipe's balance: only the entrance bar is
                // accented (0.42); every later downbeat sits under the melody at
                // 0.28. Hitting entrance level on every bar read as too punchy.
                push_dev_perc(
                    events,
                    id,
                    bar_index * bar,
                    pulse,
                    if bar_index == 0 { 0.42 } else { 0.28 },
                    "kick",
                );
                // Full backbeat mode: the Theme's kick–snare–kick–snare. Kicks
                // land on 1 and 3, snares on 2 and 4 at the Theme's velocities.
                if backbeat {
                    for (step, velocity) in [(2u32, 0.4), (6, 0.32)] {
                        push_dev_perc(
                            events,
                            id,
                            bar_index * bar + step * pulse,
                            pulse,
                            velocity,
                            "snare",
                        );
                    }
                }
                // One percussion owner per seam: the previous phase's fill owns
                // the join, so bar 0 keeps only the downbeat. Use figure for off
                if bar_index >= 1 {
                    push_dev_perc(
                        events,
                        id,
                        bar_index * bar + offbeat * pulse,
                        pulse,
                        (0.18 + 0.04 * arc).clamp(0.08, 0.55),
                        "kick",
                    );
                }
                // Hats (and groove) use the figure's steps now; preserve downbeat
                // kick separately. Figure drives rhythm variety per phase.
                if bar_index >= 1 {
                    for &s in figure.steps {
                        if s == 0 {
                            continue;
                        }
                        // place hat on odd-ish or all non zero of figure
                        let st = bar_index * bar + (s as u32) * sixteenth;
                        push_dev_perc(events, id, st, pulse / 3, 0.16, "hat");
                    }
                }
            }
            // A final-bar tom fill once the phase is energetic enough. The lick
            // is seeded, so two takes do not close with the same fill.
            if energy >= 40 && bars >= 4 {
                let fill_bar = bars - 1;
                let licks: [&[u32]; 4] = [
                    &[0, 2, 3, 4, 5, 6, 7],
                    &[0, 1, 2, 3, 4, 6, 7],
                    &[0, 2, 4, 5, 6, 7],
                    &[0, 2, 3, 5, 6, 7],
                ];
                let lick = licks[random.integer(licks.len() as u32) as usize];
                let total = lick.len() as f64;
                for (index, step) in lick.iter().enumerate() {
                    let position = index as f64 / total.max(1.0);
                    push_dev_perc(
                        events,
                        id,
                        fill_bar * bar + step * pulse,
                        pulse / 2,
                        (0.1 + 0.12 * (0.4 + position)).clamp(0.08, 0.3),
                        "tom",
                    );
                }
            }
        }
    }
    if id == "verse" || id == "solo" {
        develop_pilot_percussion(events, id, bar, bars, seed, figure);
    }
}

fn develop_pilot_percussion(
    events: &mut Vec<MusicEvent>,
    id: &str,
    bar: u32,
    bars: u32,
    seed: u32,
    figure: &FigureSpec,
) {
    let mut rng = DeterministicRandom::new(seed ^ crate::rng::hash_text(&format!("{id}:pilot")));
    let pulse = bar / 8;
    let sixteenth = bar / 16;
    let thirtysecond = bar / 32;
    let mut occupied: std::collections::HashSet<u32> = events
        .iter()
        .filter_map(|e| match e {
            MusicEvent::Percussion { start_tick, .. } => Some(*start_tick),
            _ => None,
        })
        .collect();
    let ghost_offs: &[u32] = &[1, 3, 5, 7, 9, 11, 13, 15];
    for bar_index in 0..bars {
        for &off in ghost_offs {
            if rng.integer(12) != 0 {
                continue;
            }
            let t = bar_index * bar + off * sixteenth;
            if occupied.contains(&t) {
                continue;
            }
            let vel = 0.10 + rng.next() * 0.06;
            push_dev_perc(events, id, t, pulse / 4, vel, "snare");
            occupied.insert(t);
        }
    }
    let accent_offs: &[u32] = &[3, 5, 7, 11, 13, 15];
    let target = if bars >= 8 { 2 } else { 1 };
    let mut placed = 0;
    for _ in 0..40 {
        if placed >= target {
            break;
        }
        let bi = if rng.integer(2) == 0 {
            rng.integer(bars)
        } else {
            let bnds: Vec<u32> = (0..bars).filter(|&i| i % 4 == 3 || i + 1 == bars).collect();
            if bnds.is_empty() {
                rng.integer(bars)
            } else {
                *rng.pick(&bnds)
            }
        };
        let off = *rng.pick(accent_offs);
        let main = bi * bar + off * sixteenth;
        if occupied.contains(&main) {
            continue;
        }
        let grace = main.saturating_sub(thirtysecond);
        if grace == main || occupied.contains(&grace) {
            continue;
        }
        let vg = 0.08 + rng.next() * 0.01;
        push_dev_perc(events, id, grace, sixteenth / 2, vg, "snare");
        occupied.insert(grace);
        let vm = 0.14 + rng.next() * 0.03;
        push_dev_perc(events, id, main, pulse / 2, vm, "snare");
        occupied.insert(main);
        placed += 1;
    }
    let hat_steps: Vec<u32> = figure
        .steps
        .iter()
        .copied()
        .filter(|&s| s != 0)
        .map(|s| s as u32)
        .collect();
    for bar_index in 1..bars {
        for w in hat_steps.windows(2) {
            let (a, b) = (w[0], w[1]);
            for m in (a + 1)..b {
                if rng.integer(8) != 0 {
                    continue;
                }
                let t = bar_index * bar + m * sixteenth;
                if occupied.contains(&t) {
                    continue;
                }
                let vel = 0.08 + rng.next() * 0.03;
                push_dev_perc(events, id, t, pulse / 3, vel, "hat");
                occupied.insert(t);
            }
        }
    }
    for g in 0.. {
        let ge = (g + 1) * 4 - 1;
        if ge >= bars {
            break;
        }
        if ge + 1 == bars {
            continue;
        }
        let arc = arc_factor(ge, bars);
        let nh = 1 + rng.integer(2) as usize;
        let base = 11 + rng.integer(4);
        for i in 0..nh {
            let s = base + i as u32;
            if s >= 16 {
                break;
            }
            let t = ge * bar + s * sixteenth;
            if occupied.contains(&t) {
                continue;
            }
            let vel = (0.09 + 0.12 * arc).clamp(0.08, 0.30);
            push_dev_perc(events, id, t, pulse / 2, vel, "tom");
            occupied.insert(t);
        }
    }
    if bars >= 2 {
        let pre = bars - 2;
        let arc = arc_factor(pre, bars);
        let nh = 2 + rng.integer(2) as usize;
        let base = 7 + rng.integer(6);
        for i in 0..nh {
            let s = base + i as u32 * (1 + rng.integer(2));
            if s >= 16 {
                break;
            }
            let t = pre * bar + s * sixteenth;
            if occupied.contains(&t) {
                continue;
            }
            let vel = (0.11 + 0.14 * arc).clamp(0.08, 0.30);
            push_dev_perc(events, id, t, pulse / 2, vel, "tom");
            occupied.insert(t);
        }
        let fin = bars - 1;
        let arc = arc_factor(fin, bars);
        let tails: [&[u32]; 3] = [&[10, 12, 14], &[9, 12, 15], &[8, 11, 14, 15]];
        let tail = tails[rng.integer(tails.len() as u32) as usize];
        for (i, &s) in tail.iter().enumerate() {
            let t = fin * bar + s * sixteenth;
            if occupied.contains(&t) {
                continue;
            }
            let pos = i as f64 / (tail.len() as f64).max(1.0);
            let vel = (0.12 + 0.10 * arc * (1.0 - pos * 0.5)).clamp(0.08, 0.30);
            push_dev_perc(events, id, t, pulse / 2, vel, "tom");
            occupied.insert(t);
        }
    }
}

/// Author the new pool phases (GURI-787). Each groove is genuinely distinct in
/// feel and register; each break/bridge serves a different structural job, and
/// all of them develop across their bars rather than looping a single bar.
fn build_new_phases(root: u8, bar: u32, seed: u32) -> Vec<(String, PortableSection)> {
    vec![
        ("half-time".into(), build_half_time(root, bar, seed)),
        ("sub-groove".into(), build_sub_groove(root, bar, seed)),
        ("syncopated".into(), build_syncopated(root, bar, seed)),
        ("drive".into(), build_drive(root, bar, seed)),
        ("sparse".into(), build_sparse(root, bar, seed)),
        ("drum-break".into(), build_drum_break(bar, seed)),
        ("false-stop".into(), build_false_stop(root, bar, seed)),
        ("filter-break".into(), build_filter_break(root, bar, seed)),
        (
            "harmonic-bridge".into(),
            build_harmonic_bridge(root, bar, seed),
        ),
        (
            "step-up-bridge".into(),
            build_step_up_bridge(root, bar, seed),
        ),
    ]
}

fn build_half_time(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "half-time",
        "Half-Time",
        "wide slow pulse",
        "#7b8fa1",
        8 * bar,
    );
    let degrees = progression_degrees(8, seed, false);
    let pulse = bar / 8;
    develop_drone(
        &mut section.events,
        "half-time",
        (root - 12, "warm".into(), 0.15),
        bar,
        8,
        "drone",
        0,
    );
    for bar_index in 0..8 {
        let degree = degrees[bar_index as usize];
        let arc = arc_factor(bar_index, 8);
        // Half-time bass on beats 1 and 3 only, following the progression.
        for (step, offset) in [(0u32, 0i32), (4, 4)] {
            push_dev_note(
                &mut section.events,
                "half-time",
                "bass",
                bar_index * bar + step * pulse,
                bar / 2,
                0.2 * arc + 0.04,
                aeolian(root, degree + offset),
                "bass",
                false,
            );
        }
        push_dev_perc(
            &mut section.events,
            "half-time",
            bar_index * bar,
            pulse,
            0.3,
            "kick",
        );
        if bar_index >= 1 {
            push_dev_perc(
                &mut section.events,
                "half-time",
                bar_index * bar + 4 * pulse,
                pulse,
                (0.3 * arc).clamp(0.08, 0.55),
                "snare",
            );
        }
    }
    let half_fig = figure_spec("half_time").unwrap_or(&FIGURE_POOL[0]);
    develop_cell(
        &mut section.events,
        "half-time",
        ("glass".into(), 0.18),
        root,
        bar,
        8,
        &degrees,
        PhaseRole::Groove,
        half_fig,
    );
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_sub_groove(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "sub-groove",
        "Sub-Groove",
        "low-register tension",
        "#6b5b8f",
        8 * bar,
    );
    let degrees = progression_degrees(8, seed, false);
    let pulse = bar / 8;
    develop_drone(
        &mut section.events,
        "sub-groove",
        (root - 24, "bass".into(), 0.16),
        bar,
        8,
        "drone",
        0,
    );
    for bar_index in 0..8 {
        let degree = degrees[bar_index as usize];
        let arc = arc_factor(bar_index, 8);
        let tonic = aeolian(root - 12, degree);
        for step in 0..8u32 {
            let pitch = if step.is_multiple_of(2) {
                tonic
            } else {
                aeolian(root - 12, degree + 4)
            };
            push_dev_note(
                &mut section.events,
                "sub-groove",
                "pulse",
                bar_index * bar + step * pulse,
                pulse,
                (0.24 * arc).clamp(0.08, 0.72),
                pitch,
                "bass",
                false,
            );
        }
        push_dev_perc(
            &mut section.events,
            "sub-groove",
            bar_index * bar,
            pulse,
            0.36,
            "kick",
        );
        push_dev_perc(
            &mut section.events,
            "sub-groove",
            bar_index * bar + 4 * pulse,
            pulse,
            0.24,
            "kick",
        );
        if bar_index >= 2 {
            push_dev_perc(
                &mut section.events,
                "sub-groove",
                bar_index * bar + 2 * pulse,
                pulse / 3,
                0.12,
                "hat",
            );
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_syncopated(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "syncopated",
        "Syncopated",
        "off-beat displacement",
        "#9a7c5a",
        8 * bar,
    );
    let degrees = progression_degrees(8, seed, false);
    let pulse = bar / 8;
    develop_drone(
        &mut section.events,
        "syncopated",
        (root - 12, "warm".into(), 0.14),
        bar,
        8,
        "drone",
        0,
    );
    for bar_index in 0..8 {
        let degree = degrees[bar_index as usize];
        let arc = arc_factor(bar_index, 8);
        // Syncopated: hit the off-beats, leave the downbeats open.
        for step in [1u32, 3, 5, 7] {
            let pitch = aeolian(root, degree + if step % 4 == 3 { 4 } else { 0 });
            push_dev_note(
                &mut section.events,
                "syncopated",
                "pulse",
                bar_index * bar + step * pulse,
                pulse,
                0.22 * arc + 0.04,
                pitch,
                "pluck",
                false,
            );
        }
        push_dev_perc(
            &mut section.events,
            "syncopated",
            bar_index * bar,
            pulse,
            0.34,
            "kick",
        );
        push_dev_perc(
            &mut section.events,
            "syncopated",
            bar_index * bar + 5 * pulse,
            pulse,
            0.26,
            "kick",
        );
        if bar_index >= 1 {
            push_dev_perc(
                &mut section.events,
                "syncopated",
                bar_index * bar + 7 * pulse,
                pulse / 3,
                0.14,
                "hat",
            );
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_drive(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section =
        new_phase_section("drive", "Drive", "double-time momentum", "#c76b4a", 8 * bar);
    let degrees = progression_degrees(8, seed, false);
    let pulse = bar / 8;
    let sixteenth = bar / 16;
    develop_drone(
        &mut section.events,
        "drive",
        (root - 12, "warm".into(), 0.15),
        bar,
        8,
        "drone",
        0,
    );
    for bar_index in 0..8 {
        let degree = degrees[bar_index as usize];
        let arc = arc_factor(bar_index, 8);
        // Double-time: a 16th-note ostinato instead of the usual eighths.
        for step in 0..16u32 {
            let pitch = aeolian(root, degree + [0, 2, 4, 2][(step % 4) as usize]);
            push_dev_note(
                &mut section.events,
                "drive",
                "pulse",
                bar_index * bar + step * sixteenth,
                sixteenth / 2,
                (0.16 * arc).clamp(0.08, 0.72),
                pitch,
                "pulse",
                false,
            );
        }
        push_dev_perc(
            &mut section.events,
            "drive",
            bar_index * bar,
            pulse,
            0.4,
            "kick",
        );
        push_dev_perc(
            &mut section.events,
            "drive",
            bar_index * bar + 4 * pulse,
            pulse,
            0.3,
            "kick",
        );
        if bar_index >= 1 {
            for step in [1u32, 3, 5, 7] {
                push_dev_perc(
                    &mut section.events,
                    "drive",
                    bar_index * bar + step * pulse,
                    pulse / 3,
                    0.16,
                    "hat",
                );
            }
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_sparse(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "sparse",
        "Sparse",
        "open space, few gestures",
        "#8a97a0",
        4 * bar,
    );
    let degrees = progression_degrees(4, seed, false);
    let pulse = bar / 8;
    // No bed here on purpose: `sparse` is where the low pedal leaves, so the
    // piece has somewhere to transition to instead of a constant bass.
    for bar_index in 0..4u32 {
        let degree = degrees[bar_index as usize];
        if !bar_index.is_multiple_of(2) {
            push_dev_note(
                &mut section.events,
                "sparse",
                "cell",
                bar_index * bar + 4 * pulse,
                pulse * 3,
                0.16,
                aeolian(root, degree + 4),
                "dusk",
                true,
            );
        }
        if bar_index == 2 {
            push_dev_perc(
                &mut section.events,
                "sparse",
                bar_index * bar,
                pulse,
                0.2,
                "kick",
            );
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_drum_break(bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "drum-break",
        "Drum Break",
        "kit takes over",
        "#8a6a4a",
        4 * bar,
    );
    let mut random = DeterministicRandom::new(seed ^ hash_text("drum-break"));
    let pulse = bar / 8;
    let sixteenth = bar / 16;
    // Seeded fill dimensions: which bar it lands in, how late it starts inside
    // that bar, how many hits it has and how the run is shaped all vary. Varying
    // only the subdivision still left "the same bar, the same number of hits".
    let fill_bar = 2 + random.integer(2);
    let hits = 3 + random.integer(14);
    let start_sixteenth = random.integer(9);
    let shape = random.integer(3);
    let offbeat = [2u32, 3, 6][random.integer(3) as usize];
    for bar_index in 0..4u32 {
        push_dev_perc(
            &mut section.events,
            "drum-break",
            bar_index * bar,
            pulse,
            0.4,
            "kick",
        );
        push_dev_perc(
            &mut section.events,
            "drum-break",
            bar_index * bar + 4 * pulse,
            pulse,
            0.32,
            "snare",
        );
        push_dev_perc(
            &mut section.events,
            "drum-break",
            bar_index * bar + offbeat * pulse,
            pulse,
            0.24,
            "kick",
        );
        for step in [1u32, 3, 5, 7] {
            push_dev_perc(
                &mut section.events,
                "drum-break",
                bar_index * bar + step * pulse,
                pulse / 3,
                0.18,
                "hat",
            );
        }
        if bar_index == fill_bar {
            let available = 16u32.saturating_sub(start_sixteenth).max(1);
            let span = (available - 1) as f64;
            let last = (hits - 1).max(1) as f64;
            for index in 0..hits {
                let position = index as f64 / last;
                let offset = start_sixteenth + (position * span).round() as u32;
                let arc = match shape {
                    0 => 0.5 + position,
                    1 => 1.5 - position,
                    _ => 1.0 - (position * 2.0 - 1.0).abs(),
                };
                push_dev_perc(
                    &mut section.events,
                    "drum-break",
                    bar_index * bar + offset * sixteenth,
                    sixteenth / 2,
                    (0.1 + 0.1 * arc).clamp(0.08, 0.3),
                    "tom",
                );
            }
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_false_stop(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "false-stop",
        "False Stop",
        "everything stops",
        "#5a5a5a",
        4 * bar,
    );
    let mut random = DeterministicRandom::new(seed ^ hash_text("false-stop"));
    let pulse = bar / 8;
    push_dev_note(
        &mut section.events,
        "false-stop",
        "drone",
        0,
        bar,
        0.18,
        root - 12,
        "warm",
        false,
    );
    push_dev_perc(&mut section.events, "false-stop", 0, pulse, 0.3, "kick");
    // Seeded stoppage: how long everything drops out before it comes back.
    let return_at = (1 + random.integer(3)) * bar;
    push_dev_perc(
        &mut section.events,
        "false-stop",
        return_at,
        pulse,
        0.34,
        "kick",
    );
    push_dev_note(
        &mut section.events,
        "false-stop",
        "drone",
        return_at,
        bar / 2,
        0.1,
        root - 12,
        "warm",
        false,
    );
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_filter_break(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "filter-break",
        "Filter Break",
        "submerged ambience",
        "#4a7a8a",
        8 * bar,
    );
    let degrees = progression_degrees(8, seed, false);
    let pulse = bar / 8;
    develop_drone(
        &mut section.events,
        "filter-break",
        (root - 12, "organ".into(), 0.1),
        bar,
        8,
        "drone",
        0,
    );
    for bar_index in 0..8u32 {
        let degree = degrees[bar_index as usize];
        if bar_index.is_multiple_of(2) {
            push_dev_note(
                &mut section.events,
                "filter-break",
                "cell",
                bar_index * bar + 4 * pulse,
                bar / 2,
                0.12,
                aeolian(root, degree),
                "felt",
                false,
            );
        }
        if bar_index == 3 || bar_index == 6 {
            effect(&mut section, "air-impact", bar_index * bar, bar / 4, 0.12);
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_harmonic_bridge(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "harmonic-bridge",
        "Harmonic Bridge",
        "a new sonority opens",
        "#7a5a8a",
        8 * bar,
    );
    // A shifted progression so the bridge lands on fresh chords.
    let degrees: Vec<i32> = progression_degrees(8, seed, false)
        .into_iter()
        .map(|degree| degree + 2)
        .collect();
    let pulse = bar / 8;
    for bar_index in 0..8 {
        let degree = degrees[bar_index as usize];
        let arc = arc_factor(bar_index, 8);
        let tones = [
            fold_register(aeolian(root, degree), root, root + 12),
            fold_register(aeolian(root, degree + 2), root, root + 12),
            fold_register(aeolian(root, degree + 4), root, root + 12),
            fold_register(aeolian(root, degree + 8), root, root + 12),
        ];
        for step in 0..8u32 {
            push_dev_note(
                &mut section.events,
                "harmonic-bridge",
                "arp",
                bar_index * bar + step * pulse,
                pulse,
                (0.17 * arc).clamp(0.08, 0.72),
                tones[(step % 4) as usize],
                "glass",
                false,
            );
        }
    }
    let harm_fig = figure_spec("phasing").unwrap_or(&FIGURE_POOL[0]);
    develop_cell(
        &mut section.events,
        "harmonic-bridge",
        ("epiano".into(), 0.18),
        root,
        bar,
        8,
        &degrees,
        PhaseRole::Bridge,
        harm_fig,
    );
    develop_drone(
        &mut section.events,
        "harmonic-bridge",
        (root - 12, "organ".into(), 0.12),
        bar,
        8,
        "drone",
        0,
    );
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn build_step_up_bridge(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = new_phase_section(
        "step-up-bridge",
        "Step-Up Bridge",
        "rising to the peak",
        "#c75a4a",
        8 * bar,
    );
    let mut random = DeterministicRandom::new(seed ^ hash_text("step-up-bridge"));
    let pulse = bar / 8;
    // Seeded ascent shapes, so the bridge is not the same arpeggio every seed.
    let shapes: [[i32; 4]; 3] = [[0, 2, 4, 2], [0, 4, 2, 4], [0, 2, 4, 6]];
    let shape = shapes[random.integer(shapes.len() as u32) as usize];
    let start_degree = random.integer(2) as i32;
    // Ascending sequence: each two-bar phrase rises a degree toward the peak.
    for phrase in 0..4 {
        let degree = start_degree + phrase as i32;
        for bar_offset in 0..2 {
            let bar_index = phrase * 2 + bar_offset;
            let arc = arc_factor(bar_index, 8);
            for step in 0..8u32 {
                let pitch = aeolian(root, degree + shape[(step % 4) as usize]);
                push_dev_note(
                    &mut section.events,
                    "step-up-bridge",
                    "arp",
                    bar_index * bar + step * pulse,
                    pulse,
                    (0.16 + 0.02 * phrase as f64 * arc).clamp(0.08, 0.72),
                    pitch,
                    "pulse",
                    false,
                );
            }
            push_dev_perc(
                &mut section.events,
                "step-up-bridge",
                bar_index * bar,
                pulse,
                (0.3 + 0.03 * phrase as f64).clamp(0.08, 0.55),
                "kick",
            );
            if phrase >= 2 {
                push_dev_perc(
                    &mut section.events,
                    "step-up-bridge",
                    bar_index * bar + 4 * pulse,
                    pulse,
                    0.26,
                    "snare",
                );
            }
        }
    }
    let rising: [i32; 8] = [0, 1, 2, 3, 4, 3, 2, 1];
    let step_fig = figure_spec("polyrhythm").unwrap_or(&FIGURE_POOL[0]);
    develop_cell(
        &mut section.events,
        "step-up-bridge",
        ("felt".into(), 0.2),
        root + 7,
        bar,
        8,
        &rising,
        PhaseRole::Bridge,
        step_fig,
    );
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn arrangement_root(score: &PortableScore) -> Result<u8, String> {
    score
        .section("intro")
        .and_then(|section| {
            section.events.iter().find_map(|event| match event {
                MusicEvent::Note { lane, pitch, .. } if lane == "intro-drone" => Some(*pitch + 12),
                _ => None,
            })
        })
        .ok_or_else(|| "Suspense arrangement requires the opening drone".into())
}

fn aeolian(root: u8, degree: i32) -> u8 {
    const STEPS: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
    let index = degree.rem_euclid(7) as usize;
    let octave = degree.div_euclid(7);
    (i32::from(root) + STEPS[index] + octave * 12).clamp(0, 127) as u8
}

fn anomaly_extension(
    section: &mut PortableSection,
    root: u8,
    bar: u32,
    seed: u32,
    entry: usize,
) {
    let start = section.length_ticks;
    let scanning = matches!(section.id.as_str(), "verse" | "scan-ii");
    let base = if scanning { "verse" } else { "chorus" };
    let variation = seed ^ hash_text(&format!("{base}:anomaly-extension"));
    let source = anomaly(root, bar, variation, entry);
    for mut event in source.events {
        match &mut event {
            MusicEvent::Note {
                id,
                section: owner,
                lane,
                start_tick,
                velocity,
                voice,
                ..
            } => {
                if lane.ends_with("-fractured-pulse") {
                    if scanning && *start_tick < 4 * bar && *start_tick % bar < bar / 2 {
                        continue;
                    }
                    if scanning {
                        *voice = "felt".into();
                    }
                    *velocity *= if scanning { 0.65 } else { 0.9 };
                    *lane = format!("{}-extension-pulse", section.id);
                } else if lane.ends_with("-drone") {
                    *velocity = if scanning { 0.1 } else { 0.14 };
                    *lane = format!("{}-extension-drone", section.id);
                } else {
                    *velocity *= if scanning { 0.7 } else { 0.9 };
                    *lane = format!("{}-extension-echo", section.id);
                }
                *id = format!("{}:extension:{id}", section.id);
                *owner = section.id.clone();
                *start_tick += start;
            }
            MusicEvent::Percussion {
                id,
                section: owner,
                lane,
                start_tick,
                velocity,
                voice,
                ..
            } => {
                if voice != "kick" && voice != "hat" {
                    continue;
                }
                if *start_tick == 0 && voice == "kick" {
                    *velocity = 0.28;
                }
                *id = format!("{}:extension:{id}", section.id);
                *owner = section.id.clone();
                *lane = format!("{}-kit", section.id);
                *start_tick += start;
            }
        }
        section.events.push(event);
    }
    section.length_ticks += 8 * bar;
    if !scanning {
        effect(
            section,
            "reverse-cymbal",
            section.length_ticks - bar,
            bar,
            0.1,
        );
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn variation(
    id: &str,
    label: &str,
    root: u8,
    bar: u32,
    seed: u32,
    entry: usize,
) -> PortableSection {
    let scanning = id == "scan-ii";
    let mut section = PortableSection {
        id: id.into(),
        label: label.into(),
        feeling: if scanning {
            "staggered motion / quiet responses".into()
        } else {
            "fractured drive / return to the pulse".into()
        },
        color: if scanning {
            "#739fa8".into()
        } else {
            "#cf7c70".into()
        },
        length_ticks: 0,
        events: Vec::new(),
    };
    anomaly_extension(&mut section, root, bar, seed, entry);
    section
        .events
        .retain(|event| event.voice() != "reverse-cymbal");
    for event in &mut section.events {
        if let MusicEvent::Note {
            lane,
            duration_ticks,
            ..
        } = event
        {
            if lane.ends_with("-drone") {
                *duration_ticks += 8 * bar;
            }
        }
    }
    section.length_ticks = 16 * bar;
    for phrase_bar in 8_u32..16 {
        let straight_return = !scanning && phrase_bar >= 14;
        let steps: &[u32] = if straight_return {
            &[0, 1, 2, 3, 4, 5, 6, 7]
        } else if phrase_bar.is_multiple_of(2) {
            &[0, 3, 6]
        } else {
            &[1, 4, 7]
        };
        for (index, step) in steps.iter().enumerate() {
            // The response pulse colours come from the set's shared arc entry.
            let degree =
                PROGRESSION_ARC[(phrase_bar as usize + index + entry) % PROGRESSION_ARC.len()];
            let interval = (i32::from(aeolian(root, degree)) - i32::from(root)) as u8;
            section.events.push(MusicEvent::Note {
                id: format!("{id}:response:{phrase_bar}:{step}"),
                section: id.into(),
                lane: format!("{id}-response-pulse"),
                start_tick: phrase_bar * bar + step * bar / 8,
                duration_ticks: bar / 16,
                pitch: root + interval,
                voice: if straight_return || phrase_bar.is_multiple_of(2) {
                    "pulse".into()
                } else {
                    "felt".into()
                },
                velocity: if scanning { 0.14 } else { 0.19 },
                role: None,
            });
        }
        if !phrase_bar.is_multiple_of(2) && !straight_return {
            texture(
                &mut section,
                Texture {
                    lane: "response-echo",
                    voice: "dusk",
                    start: phrase_bar * bar + bar / 4,
                    duration: bar / 2,
                    pitch: root + {
                        let degree = PROGRESSION_ARC
                            [((phrase_bar / 2) as usize + entry) % PROGRESSION_ARC.len()];
                        (i32::from(aeolian(root, degree)) - i32::from(root)) as u8
                    },
                    velocity: if scanning { 0.12 } else { 0.14 },
                },
            );
        }
    }
    rhythm(&mut section, bar);
    if !scanning {
        effect(&mut section, "reverse-cymbal", 15 * bar, bar, 0.1);
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn rhythm(section: &mut PortableSection, bar: u32) {
    if section.id == "intro" {
        return;
    }
    section.events.retain(|event| !matches!(event, MusicEvent::Percussion { voice, .. } if voice == "kick" || voice == "hat"));
    if matches!(section.id.as_str(), "break" | "outro" | "coda") {
        return;
    }
    for index in 0..section.length_ticks / bar {
        for (voice, step, velocity) in [
            ("kick", 0, if index == 0 { 0.42 } else { 0.28 }),
            ("kick", 4, 0.22),
            ("hat", 1, 0.16),
            ("hat", 3, 0.16),
            ("hat", 5, 0.16),
            ("hat", 7, 0.16),
        ] {
            section.events.push(MusicEvent::Percussion {
                id: format!("{}:flow:{voice}:{index}:{step}", section.id),
                section: section.id.clone(),
                lane: format!("{}-kit", section.id),
                start_tick: index * bar + step * bar / 8,
                duration_ticks: if voice == "hat" { bar / 24 } else { bar / 8 },
                velocity,
                voice: voice.to_string(),
            });
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

struct Texture {
    lane: &'static str,
    voice: &'static str,
    start: u32,
    duration: u32,
    pitch: u8,
    velocity: f64,
}

fn texture(section: &mut PortableSection, note: Texture) {
    if section.events.iter().any(|event| {
        event.is_melody()
            && event.start_tick() < note.start + note.duration
            && event.start_tick() + event.duration_ticks() > note.start
    }) {
        return;
    }
    section.events.push(MusicEvent::Note {
        id: format!("{}:{}:{}:{}", section.id, note.lane, note.voice, note.start),
        section: section.id.clone(),
        lane: format!("{}-{}", section.id, note.lane),
        start_tick: note.start,
        duration_ticks: note.duration,
        velocity: note.velocity,
        pitch: note.pitch,
        voice: note.voice.to_string(),
        role: Some("melody".into()),
    });
}

fn effect(section: &mut PortableSection, voice: &str, start: u32, duration: u32, velocity: f64) {
    section.events.push(MusicEvent::Percussion {
        id: format!("{}:effect:{voice}:{start}", section.id),
        section: section.id.clone(),
        lane: format!("{}-effects", section.id),
        start_tick: start,
        duration_ticks: duration,
        voice: voice.to_string(),
        velocity,
    });
}

fn anomaly(root: u8, bar: u32, seed: u32, entry: usize) -> PortableSection {
    let mut section = PortableSection {
        id: "anomaly".into(),
        label: "Anomaly".into(),
        feeling: "fractured echoes / steady heartbeat".into(),
        color: "#b093e8".into(),
        length_ticks: 8 * bar,
        events: Vec::new(),
    };
    section.events.push(MusicEvent::Note {
        id: "anomaly:drone".into(),
        section: "anomaly".into(),
        lane: "anomaly-drone".into(),
        start_tick: 0,
        duration_ticks: 8 * bar - bar / 4,
        velocity: 0.14,
        pitch: root - 12,
        voice: "warm".into(),
        role: None,
    });
    let mut random = DeterministicRandom::new(seed ^ hash_text("anomaly"));
    let displacement = random.integer(3);
    for phrase_bar in 0..8 {
        let stride = if matches!(phrase_bar, 4 | 5) { 5 } else { 3 };
        for step in (0..16_u32).step_by(stride) {
            let shifted = (step + displacement + phrase_bar % 3) % 16;
            // The stagger is the character and never changes; the chord colours
            // come from the set's shared arc entry.
            let degree = PROGRESSION_ARC[((step / stride as u32
                + phrase_bar
                + entry as u32) as usize)
                % PROGRESSION_ARC.len()];
            let interval = (i32::from(aeolian(root, degree)) - i32::from(root)) as u8;
            section.events.push(MusicEvent::Note {
                id: format!("anomaly:pulse:{phrase_bar}:{step}"),
                section: "anomaly".into(),
                lane: "anomaly-fractured-pulse".into(),
                start_tick: phrase_bar * bar + shifted * bar / 16,
                duration_ticks: bar / 16,
                velocity: if shifted.is_multiple_of(4) {
                    0.24
                } else {
                    0.18
                },
                pitch: root + interval,
                voice: if phrase_bar.is_multiple_of(2) {
                    "pulse".into()
                } else {
                    "felt".into()
                },
                role: None,
            });
        }
    }
    for phrase_bar in [1u32, 3, 5, 7] {
        let degree = PROGRESSION_ARC[(phrase_bar as usize + entry) % PROGRESSION_ARC.len()];
        let interval = (i32::from(aeolian(root, degree)) - i32::from(root)) as u8;
        texture(
            &mut section,
            Texture {
                lane: "echo-cells",
                voice: "dusk",
                start: phrase_bar * bar + bar / 4,
                duration: bar / 2,
                pitch: root + interval,
                velocity: 0.16,
            },
        );
    }
    rhythm(&mut section, bar);
    effect(&mut section, "reverse-cymbal", 3 * bar, bar, 0.14);
    effect(&mut section, "air-impact", 4 * bar, bar / 4, 0.14);
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suspense::SuspenseStyle;
    use crate::suspense_pool::{validate_form_rules, Intent, PhaseRole, PHASE_POOL};

    fn input(seed: &str) -> SuspenseInput {
        SuspenseInput {
            secret: "test".into(),
            seed: seed.into(),
            style: SuspenseStyle::Terminal,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.72,
            pulse: 0.55,
        }
    }

    /// Pitch classes sounding in a phase's first or last bar.
    fn bar_pitch_classes(section: &PortableSection, bar: u32, first: bool) -> [bool; 12] {
        let mut set = [false; 12];
        let start = if first {
            0
        } else {
            section.length_ticks.saturating_sub(bar)
        };
        let end = if first { bar } else { section.length_ticks };
        for event in &section.events {
            if let MusicEvent::Note {
                start_tick,
                duration_ticks,
                pitch,
                ..
            } = event
            {
                let end_tick = start_tick + duration_ticks;
                if *start_tick < end && end_tick > start {
                    set[(*pitch % 12) as usize] = true;
                }
            }
        }
        set
    }

    /// The seam gate: every phase opens and closes on the tonic, so any legal
    /// join shares at least one pitch class. If this fails, phases that sound
    /// fine alone will clash when they are joined.
    #[test]
    fn seeded_seams_share_a_pitch_class() {
        for intent in [Intent::Loop, Intent::Arc, Intent::Long, Intent::Surprise] {
            for take in 0..60u32 {
                let score = generate_suspense_arrangement_take(
                    &input("seam"),
                    SuspenseArrangement::Seeded,
                    intent,
                    take,
                )
                .unwrap();
                let bar = score.bar_ticks();
                let order: Vec<String> = score
                    .form
                    .as_ref()
                    .unwrap()
                    .steps
                    .iter()
                    .map(|step| step.section.clone())
                    .collect();
                for pair in order.windows(2) {
                    let exit = bar_pitch_classes(score.section(&pair[0]).unwrap(), bar, false);
                    let entry = bar_pitch_classes(score.section(&pair[1]).unwrap(), bar, true);
                    assert!(
                        exit.iter().zip(entry.iter()).any(|(a, b)| *a && *b),
                        "seam {} -> {} shares no pitch class ({intent:?} take {take})",
                        pair[0],
                        pair[1]
                    );
                }
            }
        }
    }

    /// One arc for the whole take. Every phase used to roll its own rotation of
    /// the progression and close on the tonic, so the set was 27 independent
    /// harmonic dice rolls and every join was a reset. Now the entry is a pure
    /// function of the take seed and non-resolving phases lean out on the bVII,
    /// so a join is a cadence.
    #[test]
    fn pool_phases_share_one_arc_and_lean_out_at_the_seam() {
        for seed in [7u32, 19, 41, 88, 1234] {
            let open = progression_degrees(16, seed, false);
            assert_eq!(open[0], 0, "a phase must open on the tonic");
            assert_eq!(
                *open.last().unwrap(),
                6,
                "a non-resolving phase must lean out on the bVII"
            );
            let resolved = progression_degrees(16, seed, true);
            assert_eq!(resolved[0], 0);
            assert_eq!(
                *resolved.last().unwrap(),
                0,
                "a resolving phase must close on the tonic"
            );
        }
        // The take drives the entry, and the entry is not frozen to one value.
        assert_eq!(arc_entry(99), arc_entry(99));
        let entries: std::collections::BTreeSet<usize> = (0..8u32).map(arc_entry).collect();
        assert!(entries.len() >= 3, "the arc entry barely moves across takes");

        // The lean has to reach the material, and every phase opens on the tonic.
        // `scan-ii` / `breach-ii` / `anomaly` come from `variation` and `anomaly`,
        // which now walk the same shared arc as the rest of the pool.
        let mut leaning = 0;
        for take in 0..4u32 {
            let score = generate_suspense_arrangement_take(
                &input("arc"),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                take,
            )
            .unwrap();
            let root = arrangement_root(&score).unwrap();
            let bar = score.bar_ticks();
            for section in &score.sections {
                let Some(spec) = phase_spec(&section.id) else {
                    continue;
                };
                assert!(
                    bar_has_pitch_class(section, 0, bar.min(section.length_ticks), root),
                    "{} does not open on the tonic",
                    section.id
                );
                if spec.role == PhaseRole::Outro
                    || matches!(section.id.as_str(), "scan-ii" | "breach-ii" | "anomaly")
                {
                    continue;
                }
                let last = section.length_ticks.saturating_sub(bar);
                if bar_has_pitch_class(section, last, section.length_ticks, root + 10) {
                    leaning += 1;
                }
            }
        }
        assert!(leaning > 0, "no non-resolving phase reaches the bVII at the seam");
    }

    /// The pool owns more than one kit. The Theme's full kick–snare–kick–snare
    /// backbeat is a seeded mode of `develop_drums`, not the only groove, and it
    /// has to stay under the snare ceiling.
    #[test]
    fn pool_backbeat_is_a_seeded_kit_mode() {
        // Only the base sections are voiced by `develop_drums`; the pool-authored
        // phases and `variation`/`anomaly` bring their own kits.
        const DRUMS_PHASES: [&str; 14] = [
            "intro",
            "verse",
            "pre-chorus",
            "chorus",
            "break",
            "verse-b",
            "post-chorus",
            "interlude",
            "bridge",
            "bridge-b",
            "solo",
            "chorus-final",
            "outro",
            "coda",
        ];
        let mut with_backbeat = 0;
        let mut without = 0;
        for take in 0..6u32 {
            let score = build_pool_score(&input("kit"), take).unwrap();
            let bar = score.bar_ticks();
            for section in &score.sections {
                if !DRUMS_PHASES.contains(&section.id.as_str()) {
                    continue;
                }
                let snares: Vec<u32> = section
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        MusicEvent::Percussion {
                            voice,
                            start_tick,
                            velocity,
                            ..
                        } if voice == "snare" => {
                            assert!(
                                *velocity <= 0.42 + 1e-9,
                                "{} snare at {velocity} breaks the ceiling",
                                section.id
                            );
                            Some(*start_tick)
                        }
                        _ => None,
                    })
                    .collect();
                // Detect the kit by its pattern, not by snare presence: `verse`
                // and `solo` also carry the pilot's ghost/flam snares.
                let beat = bar / 4;
                let bars_in = (section.length_ticks / bar) as usize;
                let on = |beat_index: u32| {
                    snares
                        .iter()
                        .filter(|tick| *tick % bar == beat * beat_index)
                        .count()
                };
                if on(1) >= bars_in && on(3) >= bars_in {
                    with_backbeat += 1;
                } else {
                    without += 1;
                }
            }
        }
        assert!(with_backbeat > 0, "no pool phase uses the backbeat kit");
        assert!(
            without > 0,
            "every pool phase uses the backbeat; it is not a seeded mode"
        );
    }

    /// `scan-ii`, `breach-ii` and `anomaly` used to be the only pool phases that
    /// coloured themselves outside the set. They keep their stagger, their
    /// response rhythms and their drum grid, but their notes now come from the
    /// same arc as everyone else.
    #[test]
    fn pool_variation_and_anomaly_join_the_shared_arc() {
        // The arc reaches the material: two entries are not the same notes.
        assert_ne!(
            format!("{:?}", anomaly(60, 480, 4242, 0).events),
            format!("{:?}", anomaly(60, 480, 4242, 1).events),
        );
        for take in 0..3u32 {
            let score = build_pool_score(&input("arc-av"), take).unwrap();
            let root = arrangement_root(&score).unwrap();
            let allowed: Vec<u8> = PROGRESSION_ARC
                .iter()
                .map(|degree| aeolian(root, *degree) % 12)
                .collect();
            for id in ["scan-ii", "breach-ii", "anomaly"] {
                for event in &score.section(id).unwrap().events {
                    if let MusicEvent::Note { lane, pitch, .. } = event {
                        if lane.ends_with("-drone") {
                            continue;
                        }
                        assert!(
                            allowed.contains(&(pitch % 12)),
                            "{id} {lane} pitch {pitch} sits outside the shared arc"
                        );
                    }
                }
            }
        }
    }

    /// Transitions are composed material, not one fixed hand-off: the gesture,
    /// its length and its carrier follow the seed and the two phases it joins.
    #[test]
    fn pool_joins_get_different_transitions() {
        fn seam_signature(score: &PortableScore) -> Vec<String> {
            score
                .sections
                .iter()
                .flat_map(|section| section.events.iter())
                .filter(|event| seam_lane(event).ends_with("-seam"))
                .map(|event| match event {
                    MusicEvent::Note {
                        start_tick,
                        pitch,
                        voice,
                        ..
                    } => format!("n{start_tick}:{pitch}:{voice}"),
                    MusicEvent::Percussion {
                        start_tick, voice, ..
                    } => format!("p{start_tick}:{voice}"),
                })
                .collect()
        }
        let first = generate_suspense_arrangement_take(
            &input("seam-vocab"),
            SuspenseArrangement::AllPhases,
            Intent::Arc,
            0,
        )
        .unwrap();
        let second = generate_suspense_arrangement_take(
            &input("seam-vocab"),
            SuspenseArrangement::AllPhases,
            Intent::Arc,
            1,
        )
        .unwrap();
        let a = seam_signature(&first);
        assert!(!a.is_empty(), "no join wrote transition material");
        assert_ne!(a, seam_signature(&second), "the take does not change the joins");

        // No two joins are treated identically, and the ceilings hold.
        let bar = first.bar_ticks();
        let root = arrangement_root(&first).unwrap();
        let mut per_join: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut joins = 0;
        for section in &first.sections {
            let events: Vec<String> = section
                .events
                .iter()
                .filter(|event| seam_lane(event).ends_with("-seam"))
                .map(|event| match event {
                    MusicEvent::Note {
                        start_tick,
                        pitch,
                        voice,
                        velocity,
                        ..
                    } => {
                        assert!(*velocity <= 0.4 + 1e-9, "seam note too loud");
                        assert!(
                            i32::from(*pitch) <= i32::from(root) + 12,
                            "seam note sits above the register the mix allows"
                        );
                        format!("n{start_tick}:{pitch}:{voice}")
                    }
                    MusicEvent::Percussion {
                        start_tick,
                        voice,
                        velocity,
                        ..
                    } => {
                        assert!(*velocity <= 0.3 + 1e-9, "seam percussion too loud");
                        format!("p{start_tick}:{voice}")
                    }
                })
                .collect();
            if events.is_empty() {
                continue;
            }
            joins += 1;
            per_join.insert(events.join("|"));
        }
        assert!(joins > 1, "the form has no joins to treat");
        assert!(
            per_join.len() > 1,
            "every join got the same transition: {per_join:#?}"
        );
        let _ = bar;
    }

    /// The authored bar count is a starting point, not a fixed rendering: the
    /// take chooses very short, short, authored or long, so the piece breathes
    /// instead of every phase lasting exactly what it was written as.
    #[test]
    fn pool_phase_lengths_follow_the_take() {
        let mut seen: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> =
            std::collections::BTreeMap::new();
        for take in 0..12u32 {
            let score = generate_suspense_arrangement_take(
                &input("bars"),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                take,
            )
            .unwrap();
            let bar = score.bar_ticks();
            for section in &score.sections {
                seen.entry(section.id.clone())
                    .or_default()
                    .insert(section.length_ticks / bar);
            }
        }
        let varying = seen.values().filter(|lengths| lengths.len() > 1).count();
        assert!(
            varying >= 10,
            "only {varying} pool phases change length with the take"
        );
    }

    /// Ambient and drone practice never re-attack or gap a sustain: the bed has
    /// to stay continuous across the whole phase.
    #[test]
    fn pool_drones_stay_continuous() {
        for take in 0..12u32 {
            let score = generate_suspense_arrangement_take(
                &input("drone"),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                take,
            )
            .unwrap();
            let bar = score.bar_ticks();
            for section in &score.sections {
                // A break's whole job is to drop the bed, so silence there is the
                // point; everywhere else the sustain has to stay unbroken.
                if let Some(spec) = crate::suspense_pool::phase_spec(&section.id) {
                    if spec.role == PhaseRole::Break {
                        continue;
                    }
                }
                let mut covered = vec![false; section.length_ticks as usize];
                for event in &section.events {
                    if let MusicEvent::Note {
                        lane,
                        start_tick,
                        duration_ticks,
                        ..
                    } = event
                    {
                        if lane.ends_with("-drone") {
                            let end = (*start_tick + *duration_ticks).min(section.length_ticks);
                            for tick in *start_tick..end {
                                covered[tick as usize] = true;
                            }
                        }
                    }
                }
                if !covered.iter().any(|tick| *tick) {
                    continue;
                }
                let filled = covered.iter().filter(|tick| **tick).count() as u32;
                assert!(
                    filled * 100 >= section.length_ticks * 90,
                    "drone in {} covers {filled}/{} ticks",
                    section.id,
                    section.length_ticks
                );
                let mut gap = 0u32;
                let mut largest = 0u32;
                for tick in &covered {
                    gap = if *tick { 0 } else { gap + 1 };
                    largest = largest.max(gap);
                }
                assert!(
                    largest <= bar,
                    "drone in {} has a {largest}-tick gap (bar is {bar})",
                    section.id
                );
            }
        }
    }

    /// Mix balance: the pool path must not sit hotter than the recipe Fran
    /// approved. The kick used to hit entrance level on every bar (too punchy)
    /// and the `solo` cell used to run three notes per phrase at an octave up
    /// (too shrill).
    #[test]
    fn pool_mix_balance_matches_the_approved_recipe() {
        for arrangement in [SuspenseArrangement::AllPhases, SuspenseArrangement::Seeded] {
            for take in 0..8u32 {
                let score = generate_suspense_arrangement_take(
                    &input("balance"),
                    arrangement,
                    Intent::Arc,
                    take,
                )
                .unwrap();
                let bar = score.bar_ticks();
                for section in &score.sections {
                    for event in &section.events {
                        match event {
                            MusicEvent::Percussion {
                                voice,
                                velocity,
                                start_tick,
                                ..
                            } => {
                                let limit = match voice.as_str() {
                                    // Only the entrance bar carries the accent. The
                                    // limits allow for the +/-0.02 surface jitter.
                                    "kick" if *start_tick % bar == 0 && *start_tick > 0 => 0.32,
                                    "kick" => 0.45,
                                    "hat" => 0.19,
                                    "tom" => 0.34,
                                    _ => 0.45,
                                };
                                assert!(
                                    *velocity <= limit + 1e-9,
                                    "{} {voice} at {start_tick} is {velocity} (> {limit})",
                                    section.id
                                );
                            }
                            MusicEvent::Note {
                                lane,
                                velocity,
                                pitch,
                                ..
                            } => {
                                assert!(
                                    *velocity <= 0.40 + 1e-9,
                                    "{} {lane} velocity {velocity}",
                                    section.id
                                );
                                assert!(*pitch <= 91, "{} {lane} pitch {pitch}", section.id);
                            }
                        }
                    }
                    if section.id == "solo" {
                        let cells = section
                            .events
                            .iter()
                            .filter(|event| {
                                matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-cell"))
                            })
                            .count();
                        assert!(cells <= 8, "solo cell density is {cells}");
                    }
                }
            }
        }
    }

    /// Seeded-variety audit: no pool phase may play the same lick on every seed.
    /// Fran caught the drum break closing with an identical tom fill every time.
    #[test]
    fn pool_percussion_licks_vary_across_seeds() {
        let mut break_licks = std::collections::BTreeSet::new();
        let mut break_shapes = std::collections::BTreeSet::new();
        let mut stop_layouts = std::collections::BTreeSet::new();
        let mut bridge_shapes = std::collections::BTreeSet::new();
        for take in 0..16u32 {
            let score = generate_suspense_arrangement_take(
                &input(&format!("lick-{take}")),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                take,
            )
            .unwrap();
            let bar = score.bar_ticks();
            let break_onsets: Vec<u32> = score
                .section("drum-break")
                .unwrap()
                .events
                .iter()
                .filter_map(|event| match event {
                    MusicEvent::Percussion {
                        voice, start_tick, ..
                    } if voice == "tom" => Some(*start_tick),
                    _ => None,
                })
                .collect();
            if let Some(first) = break_onsets.first() {
                break_shapes.insert((
                    first / bar,
                    break_onsets.len(),
                    (first % bar) / (bar / 16).max(1),
                ));
            }
            break_licks.insert(break_onsets);
            stop_layouts.insert(
                score
                    .section("false-stop")
                    .unwrap()
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        MusicEvent::Percussion {
                            voice, start_tick, ..
                        } if voice == "kick" => Some(*start_tick),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            );
            bridge_shapes.insert(
                score
                    .section("step-up-bridge")
                    .unwrap()
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        MusicEvent::Note { lane, pitch, .. } if lane.ends_with("-arp") => {
                            Some(*pitch)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        assert!(
            break_licks.len() >= 3,
            "drum-break tom licks across seeds: {}",
            break_licks.len()
        );
        let hit_counts: Vec<usize> = break_shapes.iter().map(|(_, hits, _)| *hits).collect();
        assert!(
            break_shapes.len() >= 5,
            "drum-break fill shapes (bar, hits, start): {break_shapes:?}"
        );
        assert!(
            hit_counts.iter().min().copied().unwrap_or(0) <= 6
                && hit_counts.iter().max().copied().unwrap_or(0) >= 12,
            "drum-break hit counts never get short or long: {hit_counts:?}"
        );
        let fill_bars: std::collections::BTreeSet<u32> =
            break_shapes.iter().map(|(bar, _, _)| *bar).collect();
        assert!(
            fill_bars.len() >= 2,
            "drum-break fills land in {fill_bars:?}"
        );
        let fill_starts: std::collections::BTreeSet<u32> =
            break_shapes.iter().map(|(_, _, start)| *start).collect();
        assert!(
            fill_starts.len() >= 3,
            "drum-break fill start offsets: {fill_starts:?}"
        );
        assert!(
            stop_layouts.len() >= 3,
            "false-stop layouts across seeds: {}",
            stop_layouts.len()
        );
        assert!(
            bridge_shapes.len() >= 3,
            "step-up-bridge shapes across seeds: {}",
            bridge_shapes.len()
        );
    }

    /// The reel must change the MATERIAL, not only the form. Deriving the pool
    /// from the level seed alone left every take with the same drum break, the
    /// same fills and the same figures — which is exactly what Fran heard.
    #[test]
    fn seed_reel_changes_the_phase_material() {
        let mut break_licks = std::collections::BTreeSet::new();
        let mut verse_shapes = std::collections::BTreeSet::new();
        for reel in 0..8u32 {
            for arrangement in [SuspenseArrangement::Seeded, SuspenseArrangement::AllPhases] {
                let score = generate_suspense_arrangement_take(
                    &input("reel-material"),
                    arrangement,
                    Intent::Arc,
                    reel,
                )
                .unwrap();
                let onsets = |id: &str, voice: &str| -> Vec<u32> {
                    score
                        .section(id)
                        .unwrap()
                        .events
                        .iter()
                        .filter_map(|event| match event {
                            MusicEvent::Percussion {
                                voice: event_voice,
                                start_tick,
                                ..
                            } if event_voice == voice => Some(*start_tick),
                            _ => None,
                        })
                        .collect()
                };
                if arrangement == SuspenseArrangement::Seeded {
                    break_licks.insert(onsets("drum-break", "tom"));
                }
                verse_shapes.insert(onsets("verse", "kick"));
            }
        }
        assert!(
            break_licks.len() >= 3,
            "the reel never changes the drum break: {} shapes",
            break_licks.len()
        );
        assert!(
            verse_shapes.len() >= 3,
            "the reel never changes the verse kit: {} shapes",
            verse_shapes.len()
        );
    }

    /// Pattern signature without velocity: jitter makes a blind pattern look
    /// varied, so the audit must ignore it.
    fn phase_signature(score: &PortableScore, id: &str) -> Vec<String> {
        score
            .section(id)
            .unwrap()
            .events
            .iter()
            .map(|event| match event {
                MusicEvent::Note {
                    lane,
                    start_tick,
                    pitch,
                    ..
                } => format!("{lane}@{start_tick}:{pitch}"),
                MusicEvent::Percussion {
                    voice, start_tick, ..
                } => {
                    format!("{voice}@{start_tick}")
                }
            })
            .collect()
    }

    /// Systematic audit for the class of bug Fran keeps catching by ear: a pool
    /// phase whose pattern is seed-blind or take-blind. It caught the drum-break
    /// fill and the reel-material bug; it is here so the next one fails CI.
    #[test]
    fn every_pool_phase_varies_with_seed_and_take() {
        use std::collections::{BTreeMap, BTreeSet};
        let mut by_seed: BTreeMap<String, BTreeSet<Vec<String>>> = BTreeMap::new();
        let mut by_take: BTreeMap<String, BTreeSet<Vec<String>>> = BTreeMap::new();
        for index in 0..8u32 {
            let by_seed_score = generate_suspense_arrangement_take(
                &input(&format!("audit-seed-{index}")),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                0,
            )
            .unwrap();
            let by_take_score = generate_suspense_arrangement_take(
                &input("audit-take"),
                SuspenseArrangement::AllPhases,
                Intent::Arc,
                index + 1,
            )
            .unwrap();
            for spec in crate::suspense_pool::PHASE_POOL {
                by_seed
                    .entry(spec.id.to_string())
                    .or_default()
                    .insert(phase_signature(&by_seed_score, spec.id));
                by_take
                    .entry(spec.id.to_string())
                    .or_default()
                    .insert(phase_signature(&by_take_score, spec.id));
            }
        }
        let mut blind: Vec<String> = Vec::new();
        for (id, signatures) in &by_seed {
            if signatures.len() < 3 {
                blind.push(format!("{id}: seed-blind ({})", signatures.len()));
            }
        }
        for (id, signatures) in &by_take {
            if signatures.len() < 3 {
                blind.push(format!("{id}: take-blind ({})", signatures.len()));
            }
        }
        assert!(blind.is_empty(), "seed/take-blind pool phases: {blind:#?}");
    }

    /// The bed must be able to leave and come back: a low pedal that runs under
    /// every phase without ever stopping reads as a sound that never changes.
    #[test]
    fn the_bed_can_leave_between_phases() {
        let score = build_pool_score(&input("bed"), 0).unwrap();
        let bed_coverage = |id: &str| -> u32 {
            let section = score.section(id).unwrap();
            let mut covered = vec![false; section.length_ticks as usize];
            for event in &section.events {
                if let MusicEvent::Note {
                    lane,
                    start_tick,
                    duration_ticks,
                    ..
                } = event
                {
                    if lane.ends_with("-drone") {
                        let end = (*start_tick + *duration_ticks).min(section.length_ticks);
                        for tick in *start_tick..end {
                            covered[tick as usize] = true;
                        }
                    }
                }
            }
            let filled = covered.iter().filter(|tick| **tick).count() as u32;
            filled * 100 / section.length_ticks.max(1)
        };
        for id in ["break", "drum-break", "sparse", "interlude", "false-stop"] {
            assert!(
                bed_coverage(id) < 50,
                "{id} still runs a low pedal through the phase ({}%)",
                bed_coverage(id)
            );
        }
        for id in ["intro", "verse", "chorus"] {
            assert!(
                bed_coverage(id) >= 90,
                "{id} lost its continuous bed ({}%)",
                bed_coverage(id)
            );
        }
    }

    /// The mid/high material must not sit as loud as the low end.
    #[test]
    fn pool_mid_high_material_sits_under_the_low_end() {
        let base = generate_suspense(&input("tilt")).unwrap();
        let root = arrangement_root(&base).unwrap();
        let score = build_pool_score(&input("tilt"), 0).unwrap();
        // Lane-local: within a voice family that appears in both registers, the
        // high notes must sit below the low ones. That is the tilt doing its job.
        let mut by_family: std::collections::BTreeMap<&str, (f64, f64)> =
            std::collections::BTreeMap::new();
        for section in &score.sections {
            for event in &section.events {
                if let MusicEvent::Note {
                    lane,
                    pitch,
                    velocity,
                    ..
                } = event
                {
                    let family = lane.rsplit('-').next().unwrap_or(lane.as_str());
                    let entry = by_family.entry(family).or_insert((0.0, 0.0));
                    if i32::from(*pitch) > i32::from(root) + 12 {
                        entry.0 = entry.0.max(*velocity);
                    } else {
                        entry.1 = entry.1.max(*velocity);
                    }
                }
            }
        }
        let mut untitled: Vec<String> = Vec::new();
        for (family, (high, low)) in &by_family {
            if *high > 0.0 && *low > 0.0 && high >= low {
                untitled.push(format!("{family}: high {high} >= low {low}"));
            }
        }
        assert!(
            untitled.is_empty(),
            "mid/high not sitting under the low end: {untitled:#?}"
        );
    }

    fn lane_onsets(section: &PortableSection, suffix: &str) -> usize {
        section
            .events
            .iter()
            .filter(
                |event| matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with(suffix)),
            )
            .count()
    }

    /// The rhythm vocabulary has to reach the music: the pool may not share one
    /// rhythmic density across every phase, and a dense figure must force short
    /// cell notes.
    #[test]
    fn pool_phases_use_different_rhythm_densities() {
        let score = build_pool_score(&input("rhythm"), 0).unwrap();
        let bar = score.bar_ticks();
        let mut densities = std::collections::BTreeSet::new();
        for section in &score.sections {
            let bars = (section.length_ticks / bar).max(1);
            let busiest = lane_onsets(section, "-arp").max(lane_onsets(section, "-pulse"));
            if busiest == 0 {
                continue;
            }
            let per_bar = busiest as u32 / bars;
            densities.insert(per_bar);
            if per_bar >= 8 {
                for event in &section.events {
                    if let MusicEvent::Note {
                        lane,
                        duration_ticks,
                        ..
                    } = event
                    {
                        if lane.ends_with("-cell") {
                            assert!(
                                *duration_ticks <= bar / 8,
                                "{} cell holds {duration_ticks} ticks under a dense figure",
                                section.id
                            );
                        }
                    }
                }
            }
        }
        assert!(
            densities.len() >= 4,
            "phases share too few rhythm densities: {densities:?}"
        );
    }

    #[test]
    fn extensions_preserve_the_existing_material_and_keep_the_new_groove_continuous() {
        let original = generate_suspense(&input("extensions")).unwrap();
        let bar = original.bar_ticks();
        for id in ["verse", "chorus"] {
            let mut section = original.section(id).unwrap().clone();
            let before = section.clone();
            let start = section.length_ticks;
            anomaly_extension(&mut section, 48, bar, 1234, arc_entry(1234));
            assert_eq!(section.length_ticks, start + 8 * bar);
            let extension_pulse: Vec<_> = section
                .events
                .iter()
                .filter(|event| {
                    matches!(event,
                MusicEvent::Note { lane, .. } if lane.ends_with("extension-pulse"))
                })
                .collect();
            if id == "verse" {
                let early = extension_pulse
                    .iter()
                    .filter(|event| event.start_tick() < start + 4 * bar)
                    .count();
                assert!(
                    early < extension_pulse.len() - early,
                    "Scan should grow from sparse to fuller pulses"
                );
            } else {
                assert!(extension_pulse.iter().any(|event| event.voice() == "pulse"));
            }
            let prefix: Vec<_> = section
                .events
                .iter()
                .filter(|event| event.start_tick() < start)
                .collect();
            assert_eq!(
                serde_json::to_value(prefix).unwrap(),
                serde_json::to_value(&before.events).unwrap()
            );
            for index in 0..8 {
                let base = start + index * bar;
                let kicks: Vec<_> = section
                    .events
                    .iter()
                    .filter(|event| {
                        event.voice() == "kick"
                            && event.start_tick() >= base
                            && event.start_tick() < base + bar
                    })
                    .map(MusicEvent::start_tick)
                    .collect();
                let hats: Vec<_> = section
                    .events
                    .iter()
                    .filter(|event| {
                        event.voice() == "hat"
                            && event.start_tick() >= base
                            && event.start_tick() < base + bar
                    })
                    .map(MusicEvent::start_tick)
                    .collect();
                assert_eq!(kicks, [base, base + bar / 2]);
                assert_eq!(
                    hats,
                    [
                        base + bar / 8,
                        base + 3 * bar / 8,
                        base + 5 * bar / 8,
                        base + 7 * bar / 8
                    ]
                );
            }
            for event in section
                .events
                .iter()
                .filter(|event| event.start_tick() >= start)
            {
                assert_ne!(event.voice(), "glass");
                if let MusicEvent::Note { lane, velocity, .. } = event {
                    if lane.ends_with("extension-pulse") {
                        assert!(event.duration_ticks() <= bar / 16);
                        assert!(*velocity <= 0.216);
                        if id == "verse" {
                            assert_eq!(event.voice(), "felt");
                        }
                    }
                }
            }
        }
    }

    /// `scan-ii` / `breach-ii` develop past their base phase instead of
    /// repeating the first half: the extension pulses and the response pulses
    /// are different material.
    #[test]
    fn independent_variations_develop_instead_of_repeating_the_first_half() {
        let score = build_pool_score(&input("pairs"), 0).unwrap();
        let bar = score.bar_ticks();
        for (base, variant) in [("verse", "scan-ii"), ("chorus", "breach-ii")] {
            assert!(score.section(base).is_some());
            let section = score.section(variant).unwrap();
            assert_eq!(section.length_ticks, 16 * bar);
            let first: Vec<_> = section
                .events
                .iter()
                .filter(|event| matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-extension-pulse")))
                .map(MusicEvent::start_tick)
                .collect();
            let second: Vec<_> = section
                .events
                .iter()
                .filter(|event| matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-response-pulse")))
                .map(|event| event.start_tick() - 8 * bar)
                .collect();
            assert!(!first.is_empty() && !second.is_empty());
            assert_ne!(first, second);
            assert!(section.events.iter().all(|event| event.voice() != "glass"));
        }
    }

    /// Retired frozen-preset names resolve to the pool default instead of
    /// failing, so old Racing-era callers keep generating a song.
    #[test]
    fn retired_presets_parse_to_the_pool_default() {
        for legacy in ["", "original", "extended", "theme", "seeded"] {
            assert_eq!(
                SuspenseArrangement::parse(legacy),
                Ok(SuspenseArrangement::Seeded),
                "{legacy}"
            );
        }
        assert_eq!(
            SuspenseArrangement::parse("all-phases"),
            Ok(SuspenseArrangement::AllPhases)
        );
        for unknown in ["variety", "flow", "featured", "missing"] {
            assert!(
                SuspenseArrangement::parse(unknown).is_err(),
                "{unknown} should not parse"
            );
        }
    }

    #[test]
    fn all_phases_plays_every_pool_phase_once_and_loops_from_a_groove() {
        let score =
            generate_suspense_arrangement(&input("pool"), SuspenseArrangement::AllPhases).unwrap();
        let form = score.form.as_ref().unwrap();
        let ids: Vec<&str> = form
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert_eq!(ids.len(), PHASE_POOL.len());
        for spec in PHASE_POOL.iter() {
            assert_eq!(
                ids.iter().filter(|id| **id == spec.id).count(),
                1,
                "{}",
                spec.id
            );
        }
        let loop_from = form.loop_from.unwrap() as usize;
        assert_eq!(
            PHASE_POOL
                .iter()
                .find(|spec| spec.id == form.steps[loop_from].section)
                .unwrap()
                .role,
            PhaseRole::Groove
        );
        assert_eq!(score.sections.len(), PHASE_POOL.len());
        assert!(score.title.ends_with("All phases"));
    }

    #[test]
    fn seeded_is_deterministic_and_distinct_across_seeds() {
        let first = generate_suspense_arrangement_intent(
            &input("composed"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
        )
        .unwrap();
        let second = generate_suspense_arrangement_intent(
            &input("composed"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );

        let mut forms = std::collections::HashSet::new();
        for index in 0..300 {
            let score = generate_suspense_arrangement_intent(
                &input(&format!("seed-{index}")),
                SuspenseArrangement::Seeded,
                Intent::Arc,
            )
            .unwrap();
            forms.insert(serde_json::to_value(&score.form).unwrap());
        }
        assert!(
            forms.len() >= 270,
            "only {} distinct forms over 300 seeds",
            forms.len()
        );
    }

    #[test]
    fn seeded_sweep_satisfies_every_rule_and_keeps_events_in_section() {
        let mut seen = std::collections::HashSet::new();
        for intent in [Intent::Loop, Intent::Arc, Intent::Long, Intent::Surprise] {
            for index in 0..250 {
                let input = input(&format!("sweep-{intent:?}-{index}"));
                let score = generate_suspense_arrangement_intent(
                    &input,
                    SuspenseArrangement::Seeded,
                    intent,
                )
                .unwrap();
                // generate_seeded already calls validate(); reaching here means
                // it passed (event bounds, id uniqueness, section refs).
                let form = score.form.as_ref().unwrap();
                let ids: Vec<&str> = form
                    .steps
                    .iter()
                    .map(|step| step.section.as_str())
                    .collect();
                validate_form_rules(&ids, form.loop_from)
                    .unwrap_or_else(|error| panic!("{intent:?} seed {index}: {error}"));
                for step in &form.steps {
                    assert!(score.section(&step.section).is_some());
                    seen.insert(step.section.clone());
                }
            }
        }
        // The new phases must actually be reachable by the composer, not just
        // present in the pool.
        for new_phase in [
            "half-time",
            "sub-groove",
            "syncopated",
            "drive",
            "sparse",
            "drum-break",
            "false-stop",
            "filter-break",
            "harmonic-bridge",
            "step-up-bridge",
        ] {
            assert!(
                seen.contains(new_phase),
                "{new_phase} never appears in a composed form"
            );
        }
    }

    #[test]
    fn seeded_intent_changes_the_shape_but_not_the_sections() {
        let base = generate_suspense_arrangement_intent(
            &input("shape"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
        )
        .unwrap();
        let looping = generate_suspense_arrangement_intent(
            &input("shape"),
            SuspenseArrangement::Seeded,
            Intent::Loop,
        )
        .unwrap();
        // Same pool of sections (same secret/seed), different form + id. The
        // join treatments legitimately differ, because they follow the form, so
        // compare each section's own material without the `-seam` lanes.
        let material = |score: &PortableScore| -> Vec<serde_json::Value> {
            score
                .sections
                .iter()
                .map(|section| {
                    let mut section = section.clone();
                    section
                        .events
                        .retain(|event| !seam_lane(event).ends_with("-seam"));
                    serde_json::to_value(section).unwrap()
                })
                .collect()
        };
        assert_eq!(material(&base), material(&looping));
        assert_ne!(base.form, looping.form);
        assert_ne!(base.id, looping.id);
    }

    #[test]
    fn seeded_reel_changes_the_composition_and_jumps_straight_to_a_take() {
        let base = generate_suspense_arrangement_take(
            &input("reel"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
            0,
        )
        .unwrap();
        let take6 = generate_suspense_arrangement_take(
            &input("reel"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
            6,
        )
        .unwrap();
        let take7 = generate_suspense_arrangement_take(
            &input("reel"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
            7,
        )
        .unwrap();
        let take7_again = generate_suspense_arrangement_take(
            &input("reel"),
            SuspenseArrangement::Seeded,
            Intent::Arc,
            7,
        )
        .unwrap();
        // Deterministic and index-addressed (no chaining between takes).
        assert_eq!(take7.form, take7_again.form);
        assert_ne!(base.form, take7.form, "the reel take must change the form");
        assert_ne!(take6.form, take7.form);
    }

    fn bar_pulse_root(section: &PortableSection, bar_index: u32, bar: u32) -> u8 {
        section
            .events
            .iter()
            .find_map(|event| match event {
                MusicEvent::Note {
                    lane,
                    start_tick,
                    pitch,
                    ..
                } if lane.ends_with("-pulse")
                    && *start_tick / bar == bar_index
                    && *start_tick % bar == 0 =>
                {
                    Some(*pitch)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{} has no downbeat pulse in bar {bar_index}", section.id))
    }

    #[test]
    fn phases_move_through_a_progression_with_mixed_lengths() {
        let score = build_pool_score(&input("prog"), 0).unwrap();
        let bar = score.bar_ticks();

        // Mixed lengths: breaks/waits may shrink to 1-2 bars and momentum phases
        // may stretch to 32, so the pool offers the whole range.
        let lengths: Vec<u32> = score
            .sections
            .iter()
            .map(|section| section.length_ticks / bar)
            .collect();
        assert!(lengths.contains(&4));
        assert!(lengths.iter().any(|length| *length >= 16));
        assert!(lengths
            .iter()
            .all(|length| matches!(length, 1 | 2 | 4 | 8 | 16 | 32)));

        // Harmonic progression: the verse's downbeat pulse moves across its
        // bars instead of holding one sonority. Its length is take-chosen now,
        // so compare the tonic opening with the leaning-out final bar.
        let verse = score.section("verse").unwrap();
        let verse_bars = verse.length_ticks / bar;
        assert!(verse_bars >= 4);
        assert_ne!(
            bar_pulse_root(verse, 0, bar),
            bar_pulse_root(verse, verse_bars - 1, bar),
            "verse harmony must progress to its closing bar"
        );
    }

    #[test]
    fn phases_develop_across_their_bars() {
        let score = build_pool_score(&input("breath"), 0).unwrap();
        let bar = score.bar_ticks();

        // Development: a long verse must not read the same in its last bar as
        // its first (layers arrive, harmony moves, a fill lands at the end).
        let verse = score.section("verse").unwrap();
        let verse_bars = verse.length_ticks / bar;
        assert!(verse_bars >= 4);
        let signature = |bar_index: u32| -> Vec<(String, u32, Option<u8>)> {
            let start = bar_index * bar;
            let mut entries: Vec<_> = verse
                .events
                .iter()
                .filter(|event| event.start_tick() >= start && event.start_tick() < start + bar)
                .map(|event| {
                    (
                        event.voice().to_string(),
                        event.start_tick() - start,
                        event.pitch(),
                    )
                })
                .collect();
            entries.sort();
            entries
        };
        assert_ne!(
            signature(0),
            signature(verse_bars - 1),
            "verse first bar must not equal its last"
        );
    }

    /// The development pilot: the two pilot phases stop holding one texture for
    /// their whole length, and every other phase is left untouched.
    #[test]
    fn the_pool_phases_develop_in_blocks() {
        let score =
            generate_suspense_arrangement(&input("dev-arc"), SuspenseArrangement::AllPhases)
                .unwrap();
        let bar = score.bar_ticks();
        let block_ticks = bar * 4;
        let has_kit = |section: &PortableSection, block: u32| {
            section.events.iter().any(|event| {
                matches!(event, MusicEvent::Percussion { voice, start_tick, .. }
                    if matches!(voice.as_str(), "kick" | "snare" | "tom" | "hat")
                        && start_tick / block_ticks == block)
            })
        };
        let has_melody = |section: &PortableSection, block: u32| {
            section.events.iter().any(|event| {
                matches!(event, MusicEvent::Note { lane, start_tick, .. }
                    if (lane.ends_with("-cell") || lane.ends_with("-arp"))
                        && start_tick / block_ticks == block)
            })
        };

        // A groove keeps its kit and develops its melodic layers.
        let verse = score.section("verse").unwrap();
        let blocks = verse.length_ticks / block_ticks;
        assert!(blocks >= 2, "the arc needs two 4-bar blocks");
        assert!(has_kit(verse, 0), "a groove keeps its kit from the start");
        assert!(
            !has_melody(verse, 0),
            "the groove must expose without its melodic layer"
        );
        assert!(
            (1..blocks).any(|block| has_melody(verse, block)),
            "the groove must develop its melodic layer"
        );

        // A peak climbs out of a sparse exposition.
        let chorus = score.section("chorus").unwrap();
        assert!(
            !has_kit(chorus, 0),
            "a peak must build from a block without the kit"
        );

        // A break is the drop itself: it is left alone and stays short.
        let brk = score.section("break").unwrap();
        assert!(brk.length_ticks / bar <= 4, "a break stays short");
    }
}
