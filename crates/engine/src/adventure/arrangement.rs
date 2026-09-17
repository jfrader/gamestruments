//! The Adventure arrangement selector: the default eight-section output, the
//! all-phases tour, and the seeded path that adds the shared development arc,
//! seam gestures with shared tonic pitch class, trait bias, and register
//! ceiling.

use crate::development::{
    clear_seams, develop_section, fold_register_ceiling, plan_joins, push_seam_note,
    push_seam_perc,
};
#[cfg(test)]
use crate::development::mask_to_schedule;
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{MusicEvent, PortableScore, PortableSection, SongForm, SongFormStep};
use crate::theory::{mode_intervals, scale_pitch};

use super::composition::{AdventurePhaseRole, mode_for, Scene, SECTION_PLANS};
use super::pool::{
    adventure_compose, adventure_compose_seed, adventure_phase_bars, adventure_phase_spec,
    ADVENTURE_SECTION_IDS,
};
use super::{adventure_tonic_pitch_class, generate_adventure, AdventureInput, AdventureStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdventureArrangement {
    #[default]
    Original,
    AllPhases,
    Seeded,
}

impl AdventureArrangement {
    /// `""`/`"original"` is the legacy default and stays byte-identical.
    /// `"all-phases"` is the full eight-section quest arc in canonical order;
    /// `"seeded"` is the seeded composer (today's seeded path), with
    /// `"composed"` kept as an alias.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" => Ok(Self::Original),
            "all-phases" => Ok(Self::AllPhases),
            "seeded" | "composed" => Ok(Self::Seeded),
            other => Err(format!("Unknown adventure arrangement: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::AllPhases => "all-phases",
            Self::Seeded => "seeded",
        }
    }
}

pub fn generate_adventure_arrangement(
    input: &AdventureInput,
    arrangement: AdventureArrangement,
) -> Result<PortableScore, String> {
    match arrangement {
        AdventureArrangement::Original => generate_adventure(input),
        AdventureArrangement::AllPhases => generate_all_phases(input),
        AdventureArrangement::Seeded => generate_seeded(input),
    }
}

/// Compose an Adventure song form over the eight-section pool, mirroring
/// Racing's pool + composer. Role/energy metadata feeds the seeded composer;
/// the seeded surface then applies: development arc (layers enter/leave over
/// blocks, pedal continuous), seam gestures (deterministic per seed/pair with
/// shared tonic pc at joins via Adventure's tonic/mode), XOR-neutral trait bias,
/// and register ceiling (folds highs, default path pitches untouched).
fn generate_seeded(input: &AdventureInput) -> Result<PortableScore, String> {
    let mut score = generate_adventure(input)?;
    let bar = score.bar_ticks();
    let form_seed = adventure_compose_seed(&input.secret, &input.seed);
    let traits = AdventureTraits::from_input(input);
    let tonic = adventure_tonic_pitch_class(&input.secret, &input.seed);
    let ceiling = adventure_register_ceiling(input.style);

    // motion / danger / mystery → tempo: a continuous widen on top of the base
    // span, so each knob keeps moving the pulse to its extreme.
    score.bpm = (score.bpm + traits.motion_dev() * 16.0 + traits.danger_dev() * 12.0
        - traits.mystery_dev() * 12.0)
        .clamp(48.0, 130.0);

    // Re-time by role before composing the form so each step references the
    // final length. Sections not chosen by the composer still re-time; they are
    // carried in the score but simply not toured.
    for section in &mut score.sections {
        let Some(spec) = adventure_phase_spec(&section.id) else {
            continue;
        };
        let bars = adventure_phase_bars(&spec, form_seed);
        if bars * bar != section.length_ticks {
            retime_section(section, bars, bar);
        }
    }

    score.form = Some(adventure_compose(form_seed));

    // The seeded surface passes (only for seeded Adventure).
    for section in &mut score.sections {
        apply_adventure_development_arc(section, bar, form_seed);
    }
    apply_adventure_transition_pass(&mut score, input.style, tonic, form_seed);
    for section in &mut score.sections {
        apply_adventure_trait_response(section, bar, tonic, &traits);
        anchor_adventure_edges(section, tonic, bar);
        apply_adventure_register_ceiling(section, ceiling);
    }

    score.id.push_str("-seeded");
    score.title.push_str(" — Seeded");
    score.validate()?;
    Ok(score)
}

/// The complete quest arc: every one of the eight sections once, in canonical
/// order, looping back to the first groove (explore). The sections are the
/// default Adventure output unchanged; only the attached form and the id/title
/// suffix differ.
fn generate_all_phases(input: &AdventureInput) -> Result<PortableScore, String> {
    let mut score = generate_adventure(input)?;
    score.form = Some(adventure_all_phases_form());
    score.id.push_str("-all-phases");
    score.title.push_str(" — All phases");
    score.validate()?;
    Ok(score)
}

/// The canonical all-phases tour over the Adventure pool.
fn adventure_all_phases_form() -> SongForm {
    let steps = ADVENTURE_SECTION_IDS
        .iter()
        .map(|id| SongFormStep {
            section: (*id).to_string(),
            repeats: 1,
        })
        .collect();
    SongForm {
        steps,
        loop_from: Some(1),
        origin: None,
    }
}

/// Re-time a section to `bars` by tiling its authored block. Every event keeps
/// its lane, voice, pitch and role; only its id and onset change, so the score
/// stays valid and every event id stays unique.
fn retime_section(section: &mut PortableSection, bars: u32, bar_ticks: u32) {
    let src_len = section.length_ticks;
    if src_len == 0 {
        return;
    }
    let target = bars * bar_ticks;
    let src_events = std::mem::take(&mut section.events);
    let mut out = Vec::new();
    let mut block = 0u32;
    while block * src_len < target {
        let offset = block * src_len;
        for event in &src_events {
            let start = event.start_tick() + offset;
            if start >= target {
                continue;
            }
            let dur = event.duration_ticks().min(target - start);
            let mut event = event.clone();
            match &mut event {
                MusicEvent::Note {
                    id,
                    start_tick,
                    duration_ticks,
                    ..
                }
                | MusicEvent::Percussion {
                    id,
                    start_tick,
                    duration_ticks,
                    ..
                } => {
                    *id = format!("{id}:t{block}");
                    *start_tick = start;
                    *duration_ticks = dur;
                }
            }
            out.push(event);
        }
        block += 1;
    }
    section.events = out;
    section.length_ticks = target;
}

/// The continuous magnitude response of the four Adventure traits (0..1), used
/// by the seeded path. Each trait deviates from the neutral 0.5; the
/// deviation drives concrete parameters so a knob move changes magnitude, never
/// a branch.
#[derive(Clone, Copy)]
struct AdventureTraits {
    wonder: f64,
    danger: f64,
    mystery: f64,
    motion: f64,
}

impl AdventureTraits {
    fn from_input(input: &AdventureInput) -> Self {
        Self {
            wonder: input.wonder.clamp(0.0, 1.0),
            danger: input.danger.clamp(0.0, 1.0),
            mystery: input.mystery.clamp(0.0, 1.0),
            motion: input.motion.clamp(0.0, 1.0),
        }
    }

    fn wonder_dev(self) -> f64 {
        self.wonder - 0.5
    }

    fn danger_dev(self) -> f64 {
        self.danger - 0.5
    }

    fn mystery_dev(self) -> f64 {
        self.mystery - 0.5
    }

    fn motion_dev(self) -> f64 {
        self.motion - 0.5
    }
}

/// Safe ceiling per style so default-path pitches never fold; only piercing
/// highs from biased traits get octave-folded (pc preserved).
fn adventure_register_ceiling(style: AdventureStyle) -> u8 {
    match style {
        AdventureStyle::Folk => 74,
        AdventureStyle::Dark => 72,
        AdventureStyle::Orchestral => 78,
    }
}

/// Adventure layer ranks for the development arc. Pedal (the bed) is rank 0 and
/// never masked; bass, harmony, percussion, melody provide the varying layers.
fn adventure_layer_rank(event: &MusicEvent) -> u8 {
    match event {
        MusicEvent::Note { lane, .. } => {
            if lane == "pedal" {
                0
            } else if lane == "bass" {
                1
            } else if lane == "harmony" {
                2
            } else if lane == "melody" {
                3
            } else {
                2
            }
        }
        MusicEvent::Percussion { .. } => 2,
    }
}

/// Role → arc shape mirroring the spirit of Racing/Suspense. Intro rises then
/// settles, grooves breathe, builds/peaks fill, break/outro release.
fn adventure_arc_for_role(role: AdventurePhaseRole) -> &'static [u8] {
    match role {
        AdventurePhaseRole::Intro => &[1, 2, 3, 2],
        AdventurePhaseRole::Groove => &[2, 3, 4, 3],
        AdventurePhaseRole::Build => &[1, 2, 3, 4],
        AdventurePhaseRole::Peak => &[2, 3, 4, 4],
        AdventurePhaseRole::Break => &[4, 3, 2, 1],
        AdventurePhaseRole::Outro => &[3, 2, 1, 2],
    }
}

/// Apply development arc to one section: mask layers by block schedule derived
/// from role arc (seeded density jitter). Pedal stays; last bar untouched.
fn apply_adventure_development_arc(section: &mut PortableSection, bar: u32, seed: u32) {
    let Some(spec) = adventure_phase_spec(&section.id) else {
        return;
    };
    let arc = adventure_arc_for_role(spec.role);
    develop_section(section, bar, arc, adventure_layer_rank, seed, true, 0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdventureSeamGesture {
    Fill,
    Riser,
    Lift,
    Both,
    Tail,
}

fn adventure_seam_pitch(tonic: i32, degree: i32, base: i32, intervals: &[i32]) -> u8 {
    let p = scale_pitch(base + tonic, degree, intervals);
    p.clamp(0, 127) as u8
}

fn section_scene(id: &str) -> Option<Scene> {
    SECTION_PLANS.iter().find(|plan| plan.id == id).map(|plan| plan.scene)
}

/// Plan deterministic gestures for each join (and loop back). Uses Adventure's
/// tonic for anchor pc and the target section's mode for any melodic carriers.
/// Gestures are perc or harp/vielle based to stay in Adventure idiom.
fn apply_adventure_transition_pass(score: &mut PortableScore, style: AdventureStyle, tonic: i32, seed: u32) {
    let bar = score.bar_ticks();
    if bar == 0 {
        return;
    }
    let Some(form) = score.form.clone() else {
        return;
    };
    let pairs = plan_joins(&form);
    clear_seams(score);
    let mut serial = 0usize;
    for (out_id, in_id) in pairs {
        let out_role = adventure_phase_spec(&out_id).map(|spec| spec.role);
        let in_role = adventure_phase_spec(&in_id).map(|spec| spec.role);
        let mut rng =
            DeterministicRandom::new(seed ^ hash_text(&format!("{out_id}>{in_id}:seam")));
        let choices: &[AdventureSeamGesture] = if in_role == Some(AdventurePhaseRole::Peak) {
            &[
                AdventureSeamGesture::Fill,
                AdventureSeamGesture::Riser,
                AdventureSeamGesture::Both,
            ]
        } else if out_role == Some(AdventurePhaseRole::Peak) {
            &[
                AdventureSeamGesture::Riser,
                AdventureSeamGesture::Tail,
                AdventureSeamGesture::Both,
                AdventureSeamGesture::Fill,
            ]
        } else {
            &[
                AdventureSeamGesture::Fill,
                AdventureSeamGesture::Riser,
                AdventureSeamGesture::Lift,
                AdventureSeamGesture::Both,
                AdventureSeamGesture::Tail,
            ]
        };
        let gesture = choices[rng.integer(choices.len() as u32) as usize];
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
        let intervals = section_scene(&in_id)
            .map(|sc| mode_intervals(mode_for(style, sc)))
            .unwrap_or_else(|| mode_intervals("ionian"));
        match gesture {
            AdventureSeamGesture::Fill => {
                for (index, step) in [8u32, 10, 12, 14, 15].iter().enumerate() {
                    let offset = (step * bar / 16).min(out_span.saturating_sub(bar / 16));
                    push_seam_perc(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 16,
                        0.12 + 0.015 * index as f64,
                        "frame-drum",
                    );
                }
            }
            AdventureSeamGesture::Riser => {
                push_seam_perc(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.12,
                    "tambourine",
                );
            }
            AdventureSeamGesture::Lift => {
                for index in 0..3usize {
                    let degree = [0, 2, 4][index % 3];
                    let offset = (index as u32 * out_span / 3)
                        .min(out_span.saturating_sub(bar / 4));
                    push_seam_note(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 4,
                        0.12,
                        adventure_seam_pitch(tonic, degree, 60, &intervals),
                        "harp",
                    );
                }
            }
            AdventureSeamGesture::Both => {
                push_seam_perc(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.12,
                    "tambourine",
                );
                for index in 0..3usize {
                    let degree = [0, 2, 4][index % 3];
                    let offset = (index as u32 * out_span / 3)
                        .min(out_span.saturating_sub(bar / 4));
                    push_seam_note(
                        &mut out_events,
                        &mut serial,
                        &out_id,
                        out_start + offset,
                        bar / 4,
                        0.12,
                        adventure_seam_pitch(tonic, degree, 60, &intervals),
                        "vielle",
                    );
                }
            }
            AdventureSeamGesture::Tail => {
                let degree = 0;
                push_seam_note(
                    &mut out_events,
                    &mut serial,
                    &out_id,
                    out_start,
                    out_span,
                    0.1,
                    adventure_seam_pitch(tonic, degree, 48, &intervals),
                    "harp",
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
                "tambourine",
            ),
            2 => push_seam_perc(&mut in_events, &mut serial, &in_id, 0, bar / 8, 0.22, "frame-drum"),
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

fn adventure_bar_has_pc(section: &PortableSection, start: u32, end: u32, pc: u8) -> bool {
    section.events.iter().any(|event| match event {
        MusicEvent::Note {
            start_tick,
            duration_ticks,
            pitch,
            ..
        } => *start_tick < end && start_tick + duration_ticks > start && pitch % 12 == pc % 12,
        _ => false,
    })
}

/// Ensure first and last bar of every section contain the tonic pitch class
/// (Adventure's shared key). Added anchors use a low harp so they sit under
/// the texture; register ceiling keeps them comfortable.
fn anchor_adventure_edges(section: &mut PortableSection, tonic: i32, bar: u32) {
    if section.length_ticks == 0 || bar == 0 {
        return;
    }
    let root_note = (48 + tonic).clamp(0, 127) as u8; // low harp register
    let head_end = bar.min(section.length_ticks);
    let last_start = section.length_ticks.saturating_sub(bar);
    let mut anchors: Vec<(u32, u32)> = Vec::new();
    if !adventure_bar_has_pc(section, 0, head_end, root_note % 12) {
        anchors.push((0, head_end));
    }
    if last_start > 0
        && !adventure_bar_has_pc(section, last_start, section.length_ticks, root_note % 12)
    {
        anchors.push((last_start, section.length_ticks - last_start));
    }
    if anchors.is_empty() {
        return;
    }
    let id = section.id.clone();
    for (index, (start, duration)) in anchors.into_iter().enumerate() {
        section.events.push(MusicEvent::Note {
            id: format!("{id}:anchor:{index}"),
            section: id.clone(),
            lane: format!("{id}-anchor"),
            start_tick: start,
            duration_ticks: duration.max(1),
            velocity: 0.10,
            pitch: root_note,
            voice: "harp".to_string(),
            role: None,
        });
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

/// Fold piercing highs by octaves (pc preserved). Ceiling chosen per style so
/// that default (non-biased) generation never moves a pitch.
fn apply_adventure_register_ceiling(section: &mut PortableSection, ceiling: u8) {
    fold_register_ceiling(section, ceiling);
}

/// The continuous trait response for a seeded Adventure section, applied on
/// top of the already-generated material (motion and wonder also answer in the
/// base generator: tempo/onsets/percussion and melody register/harmonic
/// brightness). Each effect here grows with its knob and folds back to identity
/// at the neutral 0.5.
///
/// - `danger` adds percussion density, darkens the register and pushes the bass;
/// - `mystery` adds a pedal drone and bell accents, thins the harmony voices and
///   pulls the tempo down (tempo lives on the score in [`generate_seeded`]);
/// - `wonder` brightens the harmony layer.
fn apply_adventure_trait_response(
    section: &mut PortableSection,
    bar: u32,
    tonic: i32,
    traits: &AdventureTraits,
) {
    if bar == 0 {
        return;
    }
    let pulse = bar / 8;
    let id = section.id.clone();
    let mut serial = 0usize;

    // wonder → harmonic brightness: the harmony layer sings brighter.
    let wonder_scale = (1.0 + traits.wonder_dev() * 0.6).clamp(0.5, 1.6);
    // danger → bass velocity.
    let danger_bass = (1.0 + traits.danger_dev() * 0.6).clamp(0.5, 1.6);

    for event in &mut section.events {
        let MusicEvent::Note {
            lane, velocity, pitch, ..
        } = event
        else {
            continue;
        };
        if lane == "harmony" || lane == "harp" {
            *velocity = (*velocity * wonder_scale).clamp(0.04, 0.95);
        }
        if lane == "bass" {
            *velocity = (*velocity * danger_bass).clamp(0.04, 0.95);
        }
        // danger → darker register: at high danger the melody and accompaniment
        // fold down an octave (pitch class, and hence the mode, preserved).
        if traits.danger > 0.66 && matches!(lane.as_str(), "melody" | "harmony" | "harp") {
            let folded = i32::from(*pitch) - 12;
            if folded >= 36 {
                *pitch = folded as u8;
            }
        }
    }

    // danger → percussion density: frame-drums and tambourines fill empty
    // eighths, the count growing with the knob.
    let danger_perc = (traits.danger * 6.0).round() as usize;
    if danger_perc > 0 {
        let mut placed = 0usize;
        for start in (0..section.length_ticks).step_by(pulse as usize) {
            if start + pulse / 2 > section.length_ticks {
                continue;
            }
            if section.events.iter().any(|e| e.start_tick() == start) {
                continue;
            }
            let voice = if placed.is_multiple_of(2) { "frame-drum" } else { "tambourine" };
            section.events.push(MusicEvent::Percussion {
                id: format!("{id}:trait:perc:{serial}"),
                section: id.clone(),
                lane: "percussion".to_string(),
                start_tick: start,
                duration_ticks: pulse / 2,
                velocity: 0.18,
                voice: voice.to_string(),
            });
            serial += 1;
            placed += 1;
            if placed >= danger_perc {
                break;
            }
        }
    }

    // mystery → pedal drone presence: a low held drone, more present as the
    // knob climbs.
    let mystery_pedal = (traits.mystery * 4.0).round() as usize;
    if mystery_pedal > 0 {
        let pedal_pitch = (48 + tonic).clamp(0, 127) as u8;
        let mut placed = 0usize;
        for start in (bar..section.length_ticks).step_by(bar as usize) {
            if start + bar > section.length_ticks {
                continue;
            }
            section.events.push(MusicEvent::Note {
                id: format!("{id}:trait:pedal:{serial}"),
                section: id.clone(),
                lane: "pedal".to_string(),
                start_tick: start,
                duration_ticks: bar,
                velocity: 0.1,
                pitch: pedal_pitch,
                voice: "vielle".to_string(),
                role: None,
            });
            serial += 1;
            placed += 1;
            if placed >= mystery_pedal {
                break;
            }
        }
    }

    // mystery → bell accents: a bright sparkle, more present as the knob climbs.
    let mystery_bell = (traits.mystery * 3.0).round() as usize;
    if mystery_bell > 0 {
        let bell_pitch = (72 + tonic).clamp(0, 127) as u8;
        let mut placed = 0usize;
        for start in (bar / 2..section.length_ticks).step_by(bar as usize) {
            if start + pulse > section.length_ticks {
                continue;
            }
            section.events.push(MusicEvent::Note {
                id: format!("{id}:trait:bell:{serial}"),
                section: id.clone(),
                lane: "bell".to_string(),
                start_tick: start,
                duration_ticks: pulse,
                velocity: 0.14,
                pitch: bell_pitch,
                voice: "bell".to_string(),
                role: None,
            });
            serial += 1;
            placed += 1;
            if placed >= mystery_bell {
                break;
            }
        }
    }

    // mystery → fewer voices: the highest harmony voice drops away as the knob
    // climbs, so a mysterious section is sparser (never the closing bar).
    let fewer = (traits.mystery * 4.0).round() as usize;
    if fewer > 0 {
        let last_bar = section.length_ticks.saturating_sub(bar);
        let mut indices: Vec<usize> = section
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                matches!(e, MusicEvent::Note { lane, start_tick, .. }
                    if lane == "harmony" && *start_tick < last_bar)
            })
            .map(|(i, _)| i)
            .collect();
        indices.sort_by_key(|&i| {
            let e = &section.events[i];
            let pitch = e.pitch().unwrap_or(0);
            (e.start_tick(), std::cmp::Reverse(pitch))
        });
        let remove = indices.into_iter().take(fewer).collect::<Vec<_>>();
        let mut kept = Vec::with_capacity(section.events.len());
        for (i, event) in section.events.drain(..).enumerate() {
            if !remove.contains(&i) {
                kept.push(event);
            }
        }
        section.events = kept;
    }

    section.events.sort_by_key(MusicEvent::start_tick);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::AdventureStyle;
    use crate::score::MusicEvent;
    use std::collections::HashSet;

    fn sample(secret: &str, style: AdventureStyle) -> AdventureInput {
        AdventureInput {
            secret: secret.into(),
            seed: "trail-01".into(),
            style,
            wonder: 0.6,
            danger: 0.5,
            mystery: 0.6,
            motion: 0.58,
        }
    }

    #[test]
    fn parse_as_str_default_contract() {
        assert_eq!(
            AdventureArrangement::parse("").unwrap(),
            AdventureArrangement::Original
        );
        assert_eq!(
            AdventureArrangement::parse("original").unwrap(),
            AdventureArrangement::Original
        );
        assert_eq!(
            AdventureArrangement::parse("seeded").unwrap(),
            AdventureArrangement::Seeded
        );
        assert_eq!(
            AdventureArrangement::parse("composed").unwrap(),
            AdventureArrangement::Seeded
        );
        assert_eq!(
            AdventureArrangement::parse("all-phases").unwrap(),
            AdventureArrangement::AllPhases
        );
        assert!(AdventureArrangement::parse("foo").is_err());
        assert!(AdventureArrangement::parse("Original").is_err());
        assert!(AdventureArrangement::parse("extended").is_err());
        assert_eq!(AdventureArrangement::Original.as_str(), "original");
        assert_eq!(AdventureArrangement::Seeded.as_str(), "seeded");
        assert_eq!(AdventureArrangement::AllPhases.as_str(), "all-phases");
        assert_eq!(
            AdventureArrangement::default(),
            AdventureArrangement::Original
        );
    }

    #[test]
    fn original_delegates_unchanged() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("orig-delegates", style);
            let via =
                generate_adventure_arrangement(&input, AdventureArrangement::Original).expect("via");
            let direct = generate_adventure(&input).expect("direct");
            assert_eq!(
                serde_json::to_vec(&via).expect("ser"),
                serde_json::to_vec(&direct).expect("ser"),
                "{style:?}"
            );
        }
    }

    #[test]
    fn seeded_produces_a_valid_score_with_a_real_form() {
        let input = sample("composed-shape", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        assert_eq!(score.sections.len(), 8);
        let form = score.form.as_ref().expect("seeded must carry a form");
        assert!(!form.steps.is_empty());
        for step in &form.steps {
            assert!(
                score.section(&step.section).is_some(),
                "unknown {}",
                step.section
            );
        }
        if let Some(loop_from) = form.loop_from {
            assert!(loop_from < form.steps.len() as u32);
        }
        assert!(score.id.ends_with("-seeded"));
        assert!(score.title.contains(" — Seeded"));
    }

    #[test]
    fn seeded_form_starts_camp_ends_victory_and_loops_to_a_groove() {
        let input = sample("composed-frame", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        let form = score.form.as_ref().expect("form");
        let loop_from = form.loop_from.expect("seeded form must have a loopFrom");
        assert!((loop_from as usize) < form.steps.len());
        assert_eq!(form.steps.first().unwrap().section, "camp");
        assert_eq!(form.steps.last().unwrap().section, "victory");
        let loop_section = &form.steps[loop_from as usize].section;
        assert!(
            loop_section == "explore" || loop_section == "town",
            "loop point must target a groove, got {loop_section}"
        );
    }

    #[test]
    fn seeded_role_aware_lengths_stretch_only_the_middle() {
        let input = sample("composed-lengths", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        let bar = score.bar_ticks();
        for id in [
            "camp",
            "explore",
            "town",
            "dungeon",
            "combat",
            "boss",
            "sanctuary",
            "victory",
        ] {
            let sec = score.section(id).expect(id);
            let bars = sec.length_ticks / bar;
            assert_eq!(bars % 16, 0, "{id} length not block-aligned");
            match id {
                "camp" | "sanctuary" => {
                    assert_eq!(bars, 16, "{id} should stay at 16");
                }
                "victory" => assert_eq!(bars, 32, "{id} should stay at its authored 32"),
                "explore" | "town" | "combat" => {
                    assert!((32..=48).contains(&bars), "{id} bars {bars}");
                }
                "dungeon" | "boss" => assert!((16..=32).contains(&bars), "{id} bars {bars}"),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn seeded_event_ids_unique_and_in_bounds() {
        let input = sample("composed-bounds", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        let mut seen = HashSet::new();
        for sec in &score.sections {
            for ev in &sec.events {
                let id = match ev {
                    MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
                };
                assert!(seen.insert(id.clone()), "duplicate event id {}", id);
                assert!(
                    ev.start_tick() + ev.duration_ticks() <= sec.length_ticks,
                    "oob {}",
                    id
                );
            }
        }
    }

    #[test]
    fn seeded_is_deterministic_and_varies_by_seed() {
        let i1 = sample("comp-det", AdventureStyle::Folk);
        let mut i2 = i1.clone();
        i2.seed = "trail-02".to_string();
        let s1 = generate_adventure_arrangement(&i1, AdventureArrangement::Seeded).expect("s1");
        let s1b = generate_adventure_arrangement(&i1, AdventureArrangement::Seeded).expect("s1b");
        let s2 = generate_adventure_arrangement(&i2, AdventureArrangement::Seeded).expect("s2");
        assert_eq!(
            serde_json::to_vec(&s1).expect("s1s"),
            serde_json::to_vec(&s1b).expect("s1bs")
        );
        assert_ne!(s1.id, s2.id);
        assert_ne!(
            serde_json::to_vec(&s1).expect("s1s"),
            serde_json::to_vec(&s2).expect("s2s"),
            "a seed must change the seeded score, not only the id"
        );
    }

    #[test]
    fn seeded_form_varies_across_seeds() {
        let mut forms = HashSet::new();
        for i in 0..16 {
            let mut input = sample("comp-var", AdventureStyle::Folk);
            input.seed = format!("trail-var-{i}");
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("ok");
            let steps: Vec<String> = score
                .form
                .expect("form")
                .steps
                .iter()
                .map(|step| step.section.clone())
                .collect();
            forms.insert(steps);
        }
        assert!(forms.len() > 1, "seeded form must vary with seed");
    }

    #[test]
    fn seeded_leaves_original_untouched() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("comp-iso", style);
            let original = generate_adventure(&input).expect("orig");
            let seeded =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
            assert_eq!(
                serde_json::to_vec(&generate_adventure(&input).expect("orig-again")).expect("o"),
                serde_json::to_vec(&original).expect("o2")
            );
            assert!(seeded.form.is_some());
            assert_ne!(seeded.id, original.id);
        }
    }

    #[test]
    fn seeded_development_arc_varied_layers_across_blocks() {
        let input = sample("arc-vary", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        let bar = score.bar_ticks();
        for section in &score.sections {
            if section.length_ticks < bar * 8 {
                continue;
            }
            // Collect per-block max rank present (after arc mask).
            let mut block_ranks: Vec<u8> = Vec::new();
            let blocks = (section.length_ticks / (bar * 4)).max(2) as usize; // use 4-bar blocks approx
            for b in 0..blocks {
                let start = (b as u32) * bar * 4;
                let end = start + bar * 4;
                let max_r = section
                    .events
                    .iter()
                    .filter(|e| {
                        let s = e.start_tick();
                        s >= start && s < end
                    })
                    .map(adventure_layer_rank)
                    .max()
                    .unwrap_or(0);
                block_ranks.push(max_r);
            }
            // Arc must produce at least one rank variation across blocks (not flat).
            let unique: std::collections::HashSet<_> = block_ranks.iter().copied().collect();
            assert!(
                unique.len() >= 2 || section.length_ticks < bar * 12,
                "arc should vary layers in {} (ranks {:?})",
                section.id,
                block_ranks
            );
        }
    }

    #[test]
    fn seeded_seams_share_a_pitch_class() {
        let input = sample("seam-pc", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("seeded");
        let tonic = adventure_tonic_pitch_class(&input.secret, &input.seed);
        let bar = score.bar_ticks();
        let form = score.form.as_ref().expect("form");
        let mut pairs: Vec<(String, String)> = form
            .steps
            .windows(2)
            .map(|w| (w[0].section.clone(), w[1].section.clone()))
            .collect();
        if let (Some(lf), Some(last)) = (form.loop_from, form.steps.last()) {
            if let Some(first) = form.steps.get(lf as usize) {
                pairs.push((last.section.clone(), first.section.clone()));
            }
        }
        for (out_id, in_id) in pairs {
            let out_sec = score.section(&out_id).expect("out");
            let in_sec = score.section(&in_id).expect("in");
            let out_last = out_sec.length_ticks.saturating_sub(bar);
            let has_out = adventure_bar_has_pc(out_sec, out_last, out_sec.length_ticks, (tonic % 12) as u8);
            let has_in = adventure_bar_has_pc(in_sec, 0, bar.min(in_sec.length_ticks), (tonic % 12) as u8);
            assert!(
                has_out && has_in,
                "seam {out_id}>{in_id} must share tonic pc {tonic}"
            );
        }
    }

    #[test]
    fn seeded_traits_change_output_and_defaults_stay_structural() {
        let mut def = sample("bias-def", AdventureStyle::Folk);
        def.danger = 0.50;
        def.mystery = 0.45;
        def.wonder = 0.68;
        def.motion = 0.50;
        let mut other = def.clone();
        other.danger = 0.90;

        let s_def = generate_adventure_arrangement(&def, AdventureArrangement::Seeded).expect("def");
        let s_other = generate_adventure_arrangement(&other, AdventureArrangement::Seeded).expect("other");
        assert_ne!(
            serde_json::to_vec(&s_def).expect("d"),
            serde_json::to_vec(&s_other).expect("o"),
            "traits must change the seeded surface"
        );
        assert!(s_def.form.is_some());
    }

    #[test]
    fn seeded_motion_moves_tempo_monotonically() {
        let mut bpm = Vec::new();
        for motion in [0.1, 0.5, 0.9] {
            let mut input = sample("mono-motion", AdventureStyle::Folk);
            input.motion = motion;
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("ok");
            bpm.push(score.bpm);
        }
        assert!(
            bpm[0] < bpm[1] && bpm[1] < bpm[2],
            "motion bpm must be strictly ordered, got {bpm:?}"
        );
    }

    #[test]
    fn seeded_danger_moves_percussion_density_monotonically() {
        let count = |danger| {
            let mut input = sample("mono-danger", AdventureStyle::Folk);
            input.danger = danger;
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("ok");
            score
                .sections
                .iter()
                .flat_map(|s| s.events.iter())
                .filter(|e| matches!(e, MusicEvent::Percussion { .. }))
                .count()
        };
        let low = count(0.1);
        let mid = count(0.5);
        let high = count(0.9);
        assert!(
            low < mid && mid < high,
            "danger percussion density must be strictly ordered, got {low}/{mid}/{high}"
        );
    }

    #[test]
    fn seeded_mystery_moves_tempo_down_and_bell_count_up() {
        let mut bpm = Vec::new();
        let mut bells = Vec::new();
        for mystery in [0.1, 0.5, 0.9] {
            let mut input = sample("mono-mystery", AdventureStyle::Folk);
            input.mystery = mystery;
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("ok");
            bpm.push(score.bpm);
            bells.push(
                score
                    .sections
                    .iter()
                    .flat_map(|s| s.events.iter())
                    .filter(|e| e.voice() == "bell")
                    .count(),
            );
        }
        assert!(
            bpm[0] > bpm[1] && bpm[1] > bpm[2],
            "mystery bpm must strictly fall, got {bpm:?}"
        );
        assert!(
            bells[0] <= bells[1] && bells[1] <= bells[2] && bells[0] < bells[2],
            "mystery bell accents must grow, got {bells:?}"
        );
    }

    #[test]
    fn seeded_wonder_moves_harmony_brightness_monotonically() {
        let brightness = |wonder| {
            let mut input = sample("mono-wonder", AdventureStyle::Folk);
            input.wonder = wonder;
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Seeded).expect("ok");
            let velocities: Vec<f64> = score
                .sections
                .iter()
                .flat_map(|s| s.events.iter())
                .filter(|e| matches!(e, MusicEvent::Note { lane, .. } if lane == "harmony" || lane == "harp"))
                .map(|e| e.velocity())
                .collect();
            velocities.iter().sum::<f64>() / velocities.len().max(1) as f64
        };
        let low = brightness(0.1);
        let mid = brightness(0.5);
        let high = brightness(0.9);
        assert!(
            low < mid && mid < high,
            "wonder harmony brightness must be strictly ordered, got {low}/{mid}/{high}"
        );
    }

    #[test]
    fn all_phases_produces_a_valid_eight_section_score_with_a_canonical_form() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("all-phases-shape", style);
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::AllPhases).expect("all-phases");
            assert_eq!(score.sections.len(), 8);
            assert_eq!(
                score.sections.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
                ADVENTURE_SECTION_IDS.to_vec()
            );
            let form = score.form.as_ref().expect("all-phases must carry a form");
            assert_eq!(
                form.steps.iter().map(|step| step.section.as_str()).collect::<Vec<_>>(),
                ADVENTURE_SECTION_IDS.to_vec()
            );
            assert_eq!(form.loop_from, Some(1));
            assert_eq!(form.steps[1].section, "explore");
            assert!(score.id.ends_with("-all-phases"));
            assert!(score.title.contains(" — All phases"));
            score.validate().expect("all-phases must validate");
        }
    }

    #[test]
    fn all_phases_keeps_the_default_sections_byte_identical() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("all-phases-iso", style);
            let all = generate_adventure_arrangement(&input, AdventureArrangement::AllPhases).expect("all");
            let original = generate_adventure(&input).expect("orig");
            for id in ADVENTURE_SECTION_IDS {
                let as_ = all.section(id).expect(id);
                let os = original.section(id).expect(id);
                assert_eq!(
                    serde_json::to_vec(as_).expect("as"),
                    serde_json::to_vec(os).expect("os"),
                    "section {} mutated for {style:?}",
                    id
                );
            }
        }
    }

    #[test]
    fn all_phases_is_deterministic_and_varies_by_seed() {
        let i1 = sample("all-det", AdventureStyle::Folk);
        let mut i2 = i1.clone();
        i2.seed = "trail-02".to_string();
        let s1 = generate_adventure_arrangement(&i1, AdventureArrangement::AllPhases).expect("s1");
        let s1b = generate_adventure_arrangement(&i1, AdventureArrangement::AllPhases).expect("s1b");
        let s2 = generate_adventure_arrangement(&i2, AdventureArrangement::AllPhases).expect("s2");
        assert_eq!(
            serde_json::to_vec(&s1).expect("s1s"),
            serde_json::to_vec(&s1b).expect("s1bs")
        );
        assert_ne!(s1.id, s2.id);
        assert_eq!(
            s1.sections.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            s2.sections.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn seeded_is_byte_identical_to_the_composed_alias() {
        let input = sample("seed-alias", AdventureStyle::Folk);
        let seeded = generate_adventure_arrangement(
            &input,
            AdventureArrangement::parse("seeded").expect("seeded parses"),
        )
        .expect("seeded");
        let composed = generate_adventure_arrangement(
            &input,
            AdventureArrangement::parse("composed").expect("composed parses as the seeded alias"),
        )
        .expect("composed");
        assert_eq!(
            serde_json::to_vec(&seeded).expect("s"),
            serde_json::to_vec(&composed).expect("c")
        );
        assert!(seeded.id.ends_with("-seeded"));
    }

    #[test]
    fn seam_lanes_survive_the_development_mask() {
        let bar = 4 * 960;
        let block_ticks = bar * 2;
        let schedule = [1u8]; // first block masked to rank 1
        let mut section = PortableSection {
            id: "probe".into(),
            label: "probe".into(),
            feeling: "probe".into(),
            color: "#000000".into(),
            length_ticks: bar * 4,
            events: vec![
                MusicEvent::Percussion {
                    id: "seam".into(),
                    section: "probe".into(),
                    lane: "probe-seam".into(),
                    start_tick: 0,
                    duration_ticks: bar / 8,
                    velocity: 0.1,
                    voice: "kick".into(),
                },
                MusicEvent::Note {
                    id: "melody".into(),
                    section: "probe".into(),
                    lane: "probe-melody".into(),
                    start_tick: 0,
                    duration_ticks: bar,
                    velocity: 0.2,
                    pitch: 60,
                    voice: "warm".into(),
                    role: None,
                },
            ],
        };
        mask_to_schedule(&mut section, bar, block_ticks, &schedule, adventure_layer_rank);
        let lanes: Vec<&str> = section
            .events
            .iter()
            .map(|event| match event {
                MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. } => {
                    lane.as_str()
                }
            })
            .collect();
        assert!(lanes.contains(&"probe-seam"), "the seam lane must survive the mask");
        assert!(!lanes.contains(&"probe-melody"), "the melody layer must be masked");
    }
}
