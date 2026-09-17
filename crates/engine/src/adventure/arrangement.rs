//! The Adventure arrangement selector: the default eight-section output, an
//! extended alias (currently the original for future expansion), and the
//! composed path that adds the shared development arc, seam gestures with
//! shared tonic pitch class, trait bias, and register ceiling.

use crate::development::{development_schedule, mask_to_schedule};
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{MusicEvent, PortableScore, PortableSection};
use crate::theory::{mode_intervals, scale_pitch};

use super::composition::{AdventurePhaseRole, mode_for, Scene, SECTION_PLANS};
use super::pool::{
    adventure_compose, adventure_compose_seed, adventure_phase_bars, adventure_phase_spec,
};
use super::{adventure_tonic_pitch_class, generate_adventure, AdventureInput, AdventureStyle};

/// Version of the composed-arrangement wrapper itself.
pub const COMPOSED_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdventureArrangement {
    #[default]
    Original,
    Extended,
    Composed,
}

impl AdventureArrangement {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" => Ok(Self::Original),
            "extended" => Ok(Self::Extended),
            "composed" => Ok(Self::Composed),
            other => Err(format!("Unknown adventure arrangement: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Extended => "extended",
            Self::Composed => "composed",
        }
    }
}

pub fn generate_adventure_arrangement(
    input: &AdventureInput,
    arrangement: AdventureArrangement,
) -> Result<PortableScore, String> {
    match arrangement {
        AdventureArrangement::Original | AdventureArrangement::Extended => generate_adventure(input),
        AdventureArrangement::Composed => generate_composed(input),
    }
}

/// Compose an Adventure song form over the eight-section pool, mirroring
/// Racing's pool + composer. Role/energy metadata feeds the seeded composer;
/// the composed surface then applies: development arc (layers enter/leave over
/// blocks, pedal continuous), seam gestures (deterministic per seed/pair with
/// shared tonic pc at joins via Adventure's tonic/mode), XOR-neutral trait bias,
/// and register ceiling (folds highs, default path pitches untouched).
fn generate_composed(input: &AdventureInput) -> Result<PortableScore, String> {
    let mut score = generate_adventure(input)?;
    let bar = score.bar_ticks();
    let form_seed = adventure_compose_seed(&input.secret, &input.seed);
    let bias = adventure_trait_bias(input);
    let tonic = adventure_tonic_pitch_class(&input.secret, &input.seed);
    let ceiling = adventure_register_ceiling(input.style);

    // Re-time by role before composing the form so each step references the
    // final length. Sections not chosen by the composer still re-time; they are
    // carried in the score but simply not toured.
    for section in &mut score.sections {
        let Some(spec) = adventure_phase_spec(&section.id) else {
            continue;
        };
        let bars = adventure_phase_bars(&spec, form_seed ^ bias);
        if bars * bar != section.length_ticks {
            retime_section(section, bars, bar);
        }
    }

    score.form = Some(adventure_compose(form_seed));

    // The composed surface passes (only for composed Adventure).
    let pass_seed = form_seed ^ bias;
    for section in &mut score.sections {
        apply_adventure_development_arc(section, bar, pass_seed);
    }
    apply_adventure_transition_pass(&mut score, input.style, tonic, pass_seed);
    for section in &mut score.sections {
        anchor_adventure_edges(section, tonic, bar);
        apply_adventure_register_ceiling(section, ceiling);
    }

    score.id.push_str(&format!("-composed-v{COMPOSED_VERSION}"));
    score.title.push_str(" — Composed");
    score.validate()?;
    Ok(score)
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

/// XOR-neutral bias so default Lab preset (folk 0.50/0.45/0.68/0.50) produces
/// pass_seed == form_seed (no effective bias). Other traits XOR a perturbation.
fn adventure_trait_bias(input: &AdventureInput) -> u32 {
    let d = format!("{:.2}", input.danger.clamp(0.0, 1.0));
    let my = format!("{:.2}", input.mystery.clamp(0.0, 1.0));
    let w = format!("{:.2}", input.wonder.clamp(0.0, 1.0));
    let mo = format!("{:.2}", input.motion.clamp(0.0, 1.0));
    let this = hash_text(&format!("adventure-traits-v1\0{d}\0{my}\0{w}\0{mo}"));
    let neutral = hash_text("adventure-traits-v1\x000.50\x000.45\x000.68\x000.50");
    this ^ neutral
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
    if bar == 0 {
        return;
    }
    let Some(spec) = adventure_phase_spec(&section.id) else {
        return;
    };
    let bars = section.length_ticks / bar;
    let arc = adventure_arc_for_role(spec.role);
    let Some((block_bars, schedule)) = development_schedule(arc, bars) else {
        return;
    };
    let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:arc", section.id)));
    let mut schedule = schedule;
    for rank in &mut schedule {
        match rng.integer(3) {
            0 => *rank = rank.saturating_sub(1).max(1),
            2 => *rank = (*rank + 1).min(4),
            _ => {}
        }
    }
    let block_ticks = bar * block_bars;
    mask_to_schedule(section, bar, block_ticks, &schedule, adventure_layer_rank);
    section.events.sort_by_key(MusicEvent::start_tick);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdventureSeamGesture {
    Fill,
    Riser,
    Lift,
    Both,
    Tail,
}

fn adventure_seam_lane(event: &MusicEvent) -> &str {
    match event {
        MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. } => lane,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_adventure_seam_note(
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

fn push_adventure_seam_perc(
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
    for section in &mut score.sections {
        section
            .events
            .retain(|event| !adventure_seam_lane(event).ends_with("-seam"));
    }
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
                    push_adventure_seam_perc(
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
                push_adventure_seam_perc(
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
                    push_adventure_seam_note(
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
                push_adventure_seam_perc(
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
                    push_adventure_seam_note(
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
                push_adventure_seam_note(
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
            1 => push_adventure_seam_perc(
                &mut in_events,
                &mut serial,
                &in_id,
                0,
                bar / 8,
                0.12,
                "tambourine",
            ),
            2 => push_adventure_seam_perc(&mut in_events, &mut serial, &in_id, 0, bar / 8, 0.22, "frame-drum"),
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
    for event in &mut section.events {
        if let MusicEvent::Note { pitch, .. } = event {
            let mut value = i32::from(*pitch);
            while value > i32::from(ceiling) {
                value -= 12;
            }
            *pitch = u8::try_from(value.max(0)).unwrap_or(*pitch);
        }
    }
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
            AdventureArrangement::parse("extended").unwrap(),
            AdventureArrangement::Extended
        );
        assert_eq!(
            AdventureArrangement::parse("composed").unwrap(),
            AdventureArrangement::Composed
        );
        assert!(AdventureArrangement::parse("foo").is_err());
        assert!(AdventureArrangement::parse("Original").is_err());
        assert_eq!(AdventureArrangement::Original.as_str(), "original");
        assert_eq!(AdventureArrangement::Extended.as_str(), "extended");
        assert_eq!(AdventureArrangement::Composed.as_str(), "composed");
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
            let ext = generate_adventure_arrangement(&input, AdventureArrangement::Extended).expect("ext");
            assert_eq!(
                serde_json::to_vec(&ext).expect("ser"),
                serde_json::to_vec(&direct).expect("ser"),
                "extended acts as original for {style:?}"
            );
        }
    }

    #[test]
    fn composed_produces_a_valid_score_with_a_real_form() {
        let input = sample("composed-shape", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
        assert_eq!(score.sections.len(), 8);
        let form = score.form.as_ref().expect("composed must carry a form");
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
        assert!(score.id.ends_with(&format!("-composed-v{COMPOSED_VERSION}")));
        assert!(score.title.contains(" — Composed"));
    }

    #[test]
    fn composed_form_starts_camp_ends_victory_and_loops_to_a_groove() {
        let input = sample("composed-frame", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
        let form = score.form.as_ref().expect("form");
        let loop_from = form.loop_from.expect("composed form must have a loopFrom");
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
    fn composed_role_aware_lengths_stretch_only_the_middle() {
        let input = sample("composed-lengths", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
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
    fn composed_event_ids_unique_and_in_bounds() {
        let input = sample("composed-bounds", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
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
    fn composed_is_deterministic_and_varies_by_seed() {
        let i1 = sample("comp-det", AdventureStyle::Folk);
        let mut i2 = i1.clone();
        i2.seed = "trail-02".to_string();
        let s1 = generate_adventure_arrangement(&i1, AdventureArrangement::Composed).expect("s1");
        let s1b = generate_adventure_arrangement(&i1, AdventureArrangement::Composed).expect("s1b");
        let s2 = generate_adventure_arrangement(&i2, AdventureArrangement::Composed).expect("s2");
        assert_eq!(
            serde_json::to_vec(&s1).expect("s1s"),
            serde_json::to_vec(&s1b).expect("s1bs")
        );
        assert_ne!(s1.id, s2.id);
        assert_ne!(
            serde_json::to_vec(&s1).expect("s1s"),
            serde_json::to_vec(&s2).expect("s2s"),
            "a seed must change the composed score, not only the id"
        );
    }

    #[test]
    fn composed_form_varies_across_seeds() {
        let mut forms = HashSet::new();
        for i in 0..16 {
            let mut input = sample("comp-var", AdventureStyle::Folk);
            input.seed = format!("trail-var-{i}");
            let score =
                generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("ok");
            let steps: Vec<String> = score
                .form
                .expect("form")
                .steps
                .iter()
                .map(|step| step.section.clone())
                .collect();
            forms.insert(steps);
        }
        assert!(forms.len() > 1, "composed form must vary with seed");
    }

    #[test]
    fn composed_leaves_original_untouched() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("comp-iso", style);
            let original = generate_adventure(&input).expect("orig");
            let composed =
                generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("comp");
            assert_eq!(
                serde_json::to_vec(&generate_adventure(&input).expect("orig-again")).expect("o"),
                serde_json::to_vec(&original).expect("o2")
            );
            assert!(composed.form.is_some());
            assert_ne!(composed.id, original.id);
            let ext = generate_adventure_arrangement(&input, AdventureArrangement::Extended).expect("ext");
            assert_eq!(
                serde_json::to_vec(&ext).expect("e"),
                serde_json::to_vec(&original).expect("e2")
            );
        }
    }

    #[test]
    fn composed_development_arc_varied_layers_across_blocks() {
        let input = sample("arc-vary", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
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
    fn composed_seams_share_a_pitch_class() {
        let input = sample("seam-pc", AdventureStyle::Folk);
        let score =
            generate_adventure_arrangement(&input, AdventureArrangement::Composed).expect("composed");
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
    fn composed_trait_bias_changes_output_while_defaults_stay_neutral() {
        let mut def = sample("bias-def", AdventureStyle::Folk);
        def.danger = 0.50;
        def.mystery = 0.45;
        def.wonder = 0.68;
        def.motion = 0.50;
        let mut other = def.clone();
        other.danger = 0.90;

        let b_def = adventure_trait_bias(&def);
        let b_other = adventure_trait_bias(&other);
        assert_eq!(b_def, 0, "default traits must be XOR-neutral (bias 0)");
        assert_ne!(b_other, 0, "non-default must produce non-zero bias");

        let s_def = generate_adventure_arrangement(&def, AdventureArrangement::Composed).expect("def");
        let s_other = generate_adventure_arrangement(&other, AdventureArrangement::Composed).expect("other");
        assert_ne!(
            serde_json::to_vec(&s_def).expect("d"),
            serde_json::to_vec(&s_other).expect("o"),
            "traits must change the composed surface"
        );
        // defaults neutral also means lengths use unperturbed seed for the form itself
        assert!(s_def.form.is_some());
    }
}
