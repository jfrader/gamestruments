//! Adventure exploration recipe.
//!
//! Six adaptive sections (`camp`, `explore`, `clue`, `danger`, `sanctuary`,
//! `quest-complete`) that a game selects from area phase plus discovery,
//! threat, and quest progress. Ported from the Lantern Trail authoring
//! vocabulary into the shared deterministic engine: same DNA pattern and
//! portable-score output as `racing`, `suspense`, and `medieval`.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    AdaptiveCondition, AdaptiveRule, AdventureState, MusicEvent, PortableScore, PortableSection,
    SCORE_SCHEMA_VERSION,
};
use crate::theory::{json_num, mode_intervals, phrase_gain, scale_pitch, NOTE_NAMES};

pub const GENERATOR_VERSION: &str = "1.0.0";
pub const DNA_SEED_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdventureStyle {
    Campfire,
    Wilds,
    Ruins,
}

impl AdventureStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "campfire" => Ok(Self::Campfire),
            "wilds" => Ok(Self::Wilds),
            "ruins" => Ok(Self::Ruins),
            other => Err(format!("Unknown adventure style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Campfire => "campfire",
            Self::Wilds => "wilds",
            Self::Ruins => "ruins",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Campfire => "Campfire",
            Self::Wilds => "Wilds",
            Self::Ruins => "Ruins",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdventureInput {
    pub secret: String,
    pub seed: String,
    pub style: AdventureStyle,
    pub wonder: f64,
    pub danger: f64,
    pub mystery: f64,
    pub motion: f64,
}

#[derive(Clone, Copy)]
struct NormalizedTraits {
    wonder: f64,
    danger: f64,
    mystery: f64,
    motion: f64,
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn normalize(input: &AdventureInput) -> NormalizedTraits {
    NormalizedTraits {
        wonder: clamp_unit(input.wonder),
        danger: clamp_unit(input.danger),
        mystery: clamp_unit(input.mystery),
        motion: clamp_unit(input.motion),
    }
}

/// Instrument roles the style kit resolves to concrete synth voices.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Lead,
    Pluck,
    Glass,
    Warm,
    Pulse,
    Low,
    Pad,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Perc {
    None,
    Soft,
    Drive,
    Dance,
}

struct StyleKit {
    lead: &'static str,
    pluck: &'static str,
    glass: &'static str,
    warm: &'static str,
    pulse: &'static str,
    low: &'static str,
    pad: &'static str,
    base_bpm: f64,
}

fn style_kit(style: AdventureStyle) -> StyleKit {
    match style {
        AdventureStyle::Campfire => StyleKit {
            lead: "epiano",
            pluck: "pluck",
            glass: "glass",
            warm: "warm",
            pulse: "pulse",
            low: "dusk",
            pad: "felt",
            base_bpm: 104.0,
        },
        AdventureStyle::Wilds => StyleKit {
            lead: "glass",
            pluck: "pluck",
            glass: "glass",
            warm: "pulse",
            pulse: "pulse",
            low: "dusk",
            pad: "dusk",
            base_bpm: 92.0,
        },
        AdventureStyle::Ruins => StyleKit {
            lead: "epiano",
            pluck: "pluck",
            glass: "glass",
            warm: "warm",
            pulse: "pulse",
            low: "dusk",
            pad: "felt",
            base_bpm: 80.0,
        },
    }
}

fn role_voice(kit: &StyleKit, role: Role) -> &'static str {
    match role {
        Role::Lead => kit.lead,
        Role::Pluck => kit.pluck,
        Role::Glass => kit.glass,
        Role::Warm => kit.warm,
        Role::Pulse => kit.pulse,
        Role::Low => kit.low,
        Role::Pad => kit.pad,
    }
}

struct SectionPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    bars: u32,
    intensity: f64,
    mode: &'static str,
    register: i32,
    density: f64,
    perc: Perc,
    drone: bool,
    finale: bool,
    melody_role: Role,
    harmony_role: Role,
    drone_role: Role,
}

const PLANS: [SectionPlan; 6] = [
    SectionPlan {
        id: "camp",
        label: "Trailhead Camp",
        feeling: "warmth / still anticipation",
        color: "#e8c67a",
        bars: 8,
        intensity: 0.5,
        mode: "ionian",
        register: 0,
        density: 0.42,
        perc: Perc::Soft,
        drone: true,
        finale: false,
        melody_role: Role::Lead,
        harmony_role: Role::Warm,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "explore",
        label: "The Old Forest",
        feeling: "curiosity / forward motion",
        color: "#8fbf8f",
        bars: 8,
        intensity: 0.66,
        mode: "dorian",
        register: 0,
        density: 0.6,
        perc: Perc::Soft,
        drone: true,
        finale: false,
        melody_role: Role::Pluck,
        harmony_role: Role::Warm,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "clue",
        label: "A Lantern Found",
        feeling: "mystery / dawning discovery",
        color: "#7fd4d8",
        bars: 8,
        intensity: 0.72,
        mode: "mixolydian",
        register: 0,
        density: 0.66,
        perc: Perc::Soft,
        drone: true,
        finale: false,
        melody_role: Role::Glass,
        harmony_role: Role::Pulse,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "danger",
        label: "The Darkening Path",
        feeling: "menace / resolve",
        color: "#c94f4f",
        bars: 8,
        intensity: 0.9,
        mode: "aeolian",
        register: -7,
        density: 0.84,
        perc: Perc::Drive,
        drone: true,
        finale: false,
        melody_role: Role::Lead,
        harmony_role: Role::Pulse,
        drone_role: Role::Low,
    },
    SectionPlan {
        id: "sanctuary",
        label: "The Hidden Glade",
        feeling: "awe / radiant arrival",
        color: "#a6d7a0",
        bars: 8,
        intensity: 0.92,
        mode: "lydian",
        register: 0,
        density: 0.78,
        perc: Perc::Dance,
        drone: true,
        finale: false,
        melody_role: Role::Glass,
        harmony_role: Role::Warm,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "quest-complete",
        label: "Lantern Lit",
        feeling: "release / earned wonder",
        color: "#f2e39a",
        bars: 8,
        intensity: 0.72,
        mode: "ionian",
        register: 12,
        density: 0.6,
        perc: Perc::Dance,
        drone: false,
        finale: true,
        melody_role: Role::Lead,
        harmony_role: Role::Warm,
        drone_role: Role::Pad,
    },
];

const KEY_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const PROGRESSIONS: [&[i32; 4]; 4] = [&[0, 5, 3, 4], &[0, 3, 5, 4], &[0, 4, 5, 3], &[0, 2, 5, 4]];
const MOTIFS: [&[i32; 4]; 4] = [&[0, 2, 3, 1], &[0, 3, 4, 2], &[0, 1, 4, 2], &[0, 4, 3, 1]];
const ONSET_CELLS: [&[u32]; 6] = [
    &[0, 2, 4, 6],
    &[0, 3, 4, 6],
    &[0, 1, 4, 6],
    &[0, 4, 6],
    &[0, 2, 4],
    &[0, 3, 6],
];
const FINALE_ONSETS: [u32; 4] = [0, 2, 4, 6];

struct HarmonyDna {
    root_pitch_class: i32,
    progression: Vec<i32>,
    key: String,
}

fn create_harmony(seed: u32) -> HarmonyDna {
    let mut rng = DeterministicRandom::new(seed);
    let root_pitch_class = *rng.pick(&KEY_PITCH_CLASSES);
    let progression = rng.pick(&PROGRESSIONS).to_vec();
    HarmonyDna {
        root_pitch_class,
        progression,
        key: NOTE_NAMES[root_pitch_class as usize].to_string(),
    }
}

fn create_motif(seed: u32) -> [i32; 4] {
    let mut rng = DeterministicRandom::new(seed);
    let mut steps = **rng.pick(&MOTIFS);
    let index = 1 + rng.integer(3) as usize;
    steps[index] += *rng.pick(&[-1i32, 1]);
    steps[0] = 0;
    steps
}

fn subseed(secret: &str, seed: &str, domain: &str) -> u32 {
    let canonical = format!("string:{seed}");
    let mut value = format!("{DNA_SEED_VERSION}\0{canonical}\0{domain}");
    if !secret.is_empty() {
        value.push('\0');
        value.push_str(secret);
    }
    hash_text(&value)
}

fn score_id(secret: &str, seed: &str, style: AdventureStyle, traits: &NormalizedTraits) -> String {
    let trait_json = format!(
        r#"{{"wonder":{},"danger":{},"mystery":{},"motion":{}}}"#,
        json_num(traits.wonder),
        json_num(traits.danger),
        json_num(traits.mystery),
        json_num(traits.motion)
    );
    let identity = hash_text(&format!(
        "{secret}\0{seed}\0{}\0{trait_json}\0{GENERATOR_VERSION}",
        style.as_str()
    ));
    format!(
        "adventure-generated-v{}-{identity:08x}",
        GENERATOR_VERSION.replace('.', "-")
    )
}

#[derive(Clone, Copy)]
struct Roots {
    bass: i32,
    drone: i32,
    harmony: i32,
    melody: i32,
}

fn roots(plan: &SectionPlan, harmony: &HarmonyDna) -> Roots {
    let root = harmony.root_pitch_class;
    Roots {
        bass: 36 + root,
        drone: 48 + root,
        harmony: 55 + root,
        // Only the melody follows the section register; the bed stays put so
        // low sections do not collapse into a muddy octave.
        melody: 67 + root + plan.register,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_note(
    events: &mut Vec<MusicEvent>,
    section: &str,
    lane: &str,
    index: usize,
    start_tick: u32,
    duration_ticks: u32,
    velocity: f64,
    pitch: i32,
    voice: &str,
    role: Option<&str>,
) {
    events.push(MusicEvent::Note {
        id: format!("{section}:{lane}:{index}"),
        section: section.to_string(),
        lane: lane.to_string(),
        start_tick,
        duration_ticks: duration_ticks.max(1),
        velocity: velocity.clamp(0.05, 0.96),
        pitch: pitch.clamp(0, 127) as u8,
        voice: voice.to_string(),
        role: role.map(str::to_string),
    });
}

#[allow(clippy::too_many_arguments)]
fn push_perc(
    events: &mut Vec<MusicEvent>,
    section: &str,
    lane: &str,
    index: usize,
    start_tick: u32,
    duration_ticks: u32,
    velocity: f64,
    voice: &str,
) {
    events.push(MusicEvent::Percussion {
        id: format!("{section}:{lane}:{index}"),
        section: section.to_string(),
        lane: lane.to_string(),
        start_tick,
        duration_ticks: duration_ticks.max(1),
        velocity: velocity.clamp(0.05, 0.96),
        voice: voice.to_string(),
    });
}

fn event_id(event: &MusicEvent) -> &str {
    match event {
        MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
    }
}

fn drone_events(
    plan: &SectionPlan,
    roots: Roots,
    intervals: &[i32],
    bar_ticks: u32,
    voice: &str,
) -> Vec<MusicEvent> {
    let mut events = Vec::new();
    if !plan.drone {
        return events;
    }
    let length = bar_ticks * plan.bars;
    let root_pitch = scale_pitch(roots.drone, 0, intervals);
    push_note(
        &mut events,
        plan.id,
        "drone",
        0,
        0,
        length,
        0.1,
        root_pitch,
        voice,
        None,
    );
    // The fifth enters in the second phrase so the pedal has a little life.
    let phrase_b = bar_ticks * (plan.bars / 2);
    let fifth = scale_pitch(roots.drone, 4, intervals);
    push_note(
        &mut events,
        plan.id,
        "drone",
        1,
        phrase_b,
        length - phrase_b,
        0.09,
        fifth,
        voice,
        None,
    );
    events
}

#[allow(clippy::too_many_arguments)]
fn harmony_events(
    plan: &SectionPlan,
    harmony: &HarmonyDna,
    roots: Roots,
    intervals: &[i32],
    traits: &NormalizedTraits,
    bar_ticks: u32,
    voice: &str,
) -> Vec<MusicEvent> {
    let mut events = Vec::new();
    let mut index = 0;
    let degrees: Vec<i32> = if traits.wonder > 0.55 {
        vec![0, 2, 4]
    } else {
        vec![0, 4]
    };
    for bar in 0..plan.bars {
        let degree = harmony.progression[bar as usize % harmony.progression.len()];
        let bar_start = bar * bar_ticks;
        for offset in &degrees {
            let pitch = scale_pitch(roots.harmony, degree + offset, intervals);
            let velocity = (0.12 + traits.wonder * 0.06) * phrase_gain(bar);
            push_note(
                &mut events,
                plan.id,
                "harmony",
                index,
                bar_start,
                bar_ticks,
                velocity,
                pitch,
                voice,
                None,
            );
            index += 1;
        }
    }
    events
}

fn bass_events(
    plan: &SectionPlan,
    harmony: &HarmonyDna,
    roots: Roots,
    intervals: &[i32],
    bar_ticks: u32,
    pulse: u32,
) -> Vec<MusicEvent> {
    let mut events = Vec::new();
    let mut index = 0;
    let moving = matches!(plan.perc, Perc::Drive | Perc::Dance);
    for bar in 0..plan.bars {
        let degree = harmony.progression[bar as usize % harmony.progression.len()];
        let bar_start = bar * bar_ticks;
        let root = scale_pitch(roots.bass, degree, intervals);
        if moving {
            let fifth = scale_pitch(roots.bass, degree + 4, intervals);
            push_note(
                &mut events,
                plan.id,
                "bass",
                index,
                bar_start,
                pulse * 4,
                0.3,
                root,
                "bass",
                None,
            );
            index += 1;
            push_note(
                &mut events,
                plan.id,
                "bass",
                index,
                bar_start + pulse * 4,
                pulse * 4,
                0.26,
                fifth,
                "bass",
                None,
            );
            index += 1;
        } else {
            push_note(
                &mut events,
                plan.id,
                "bass",
                index,
                bar_start,
                bar_ticks,
                0.28,
                root,
                "bass",
                None,
            );
            index += 1;
        }
    }
    events
}

fn bar_targets(section: &str) -> [i32; 8] {
    match section {
        "camp" => [0, 2, 4, 2, 2, 4, 0, 0],
        "explore" => [0, 4, 2, 4, 4, 2, 1, 0],
        "clue" => [0, 2, 4, 6, 4, 2, 1, 0],
        "danger" => [0, 4, 2, 4, 6, 4, 2, 0],
        "sanctuary" => [0, 4, 6, 4, 7, 4, 2, 0],
        "quest-complete" => [0, 4, 4, 7, 7, 4, 4, 0],
        _ => [0, 2, 4, 2, 4, 2, 1, 0],
    }
}

/// Plucked styles get a broken-chord accompaniment; the held harmony is the pad.
fn wants_arp(section: &str) -> bool {
    matches!(
        section,
        "camp" | "explore" | "clue" | "sanctuary" | "quest-complete"
    )
}

#[allow(clippy::too_many_arguments)]
fn arp_events(
    plan: &SectionPlan,
    harmony: &HarmonyDna,
    roots: Roots,
    intervals: &[i32],
    bar_ticks: u32,
    pulse: u32,
    voice: &str,
) -> Vec<MusicEvent> {
    if !wants_arp(plan.id) {
        return Vec::new();
    }
    const PATTERN: [i32; 8] = [0, 4, 2, 4, 7, 4, 2, 4];
    let mut events = Vec::new();
    let mut index = 0;
    for bar in 0..plan.bars {
        let chord = harmony.progression[bar as usize % harmony.progression.len()];
        let bar_start = bar * bar_ticks;
        let gain = phrase_gain(bar);
        for (step, degree) in PATTERN.iter().enumerate() {
            let pitch = scale_pitch(roots.harmony, chord + degree, intervals);
            push_note(
                &mut events,
                plan.id,
                "arp",
                index,
                bar_start + step as u32 * pulse,
                pulse,
                (0.1 + plan.intensity * 0.03) * gain,
                pitch,
                voice,
                None,
            );
            index += 1;
        }
    }
    events
}

fn onsets_for_bar(
    rng: &mut DeterministicRandom,
    plan: &SectionPlan,
    traits: &NormalizedTraits,
    bar: u32,
) -> Vec<u32> {
    let chosen: &[u32] = if plan.finale {
        &FINALE_ONSETS
    } else {
        let target = (2.0 + plan.density * traits.motion * 4.0).round() as usize;
        let candidates: Vec<&[u32]> = ONSET_CELLS
            .iter()
            .copied()
            .filter(|cell| cell.len() <= target.max(2))
            .collect();
        if candidates.is_empty() {
            return vec![0, 4];
        }
        candidates[rng.integer(candidates.len() as u32) as usize]
    };
    if bar == plan.bars - 1 && chosen.len() > 1 {
        chosen[..chosen.len() - 1].to_vec()
    } else {
        chosen.to_vec()
    }
}

#[allow(clippy::too_many_arguments)]
fn melody_events(
    plan: &SectionPlan,
    harmony: &HarmonyDna,
    roots: Roots,
    intervals: &[i32],
    motif: &[i32; 4],
    traits: &NormalizedTraits,
    bar_ticks: u32,
    pulse: u32,
    rng: &mut DeterministicRandom,
    voice: &str,
) -> Vec<MusicEvent> {
    let mut events = Vec::new();
    let mut index = 0;
    let targets = bar_targets(plan.id);
    for bar in 0..plan.bars {
        let degree = harmony.progression[bar as usize % harmony.progression.len()];
        let target = targets[(bar % 8) as usize];
        let bar_start = bar * bar_ticks;
        let gain = phrase_gain(bar);
        let onsets = onsets_for_bar(rng, plan, traits, bar);
        for (position, step) in onsets.iter().enumerate() {
            // Anchor the downbeat on the bar's chord tone, then step through
            // light neighbour motion seeded by the motif.
            let offset = if position == 0 {
                target
            } else {
                target + 1 + motif[position % motif.len()].rem_euclid(2)
            };
            let pitch = scale_pitch(roots.melody, degree + offset, intervals);
            let next = onsets.get(position + 1).copied().unwrap_or(8);
            let raw = f64::from((next - step) * pulse);
            let duration = (raw * 0.85).max(f64::from(pulse) / 2.0) as u32;
            let velocity = (0.2 + traits.wonder * 0.06 + plan.intensity * 0.08) * gain;
            push_note(
                &mut events,
                plan.id,
                "melody",
                index,
                bar_start + step * pulse,
                duration,
                velocity,
                pitch,
                voice,
                Some("melody"),
            );
            index += 1;
        }
    }
    events
}

fn fill_hits(perc: Perc) -> &'static [(&'static str, u32)] {
    match perc {
        Perc::Soft => &[("hat", 3), ("tom", 6)],
        Perc::Drive => &[("tom", 5), ("snare", 7)],
        Perc::Dance => &[("tom", 5), ("tom", 7)],
        Perc::None => &[],
    }
}

fn percussion_events(
    plan: &SectionPlan,
    traits: &NormalizedTraits,
    bar_ticks: u32,
    pulse: u32,
) -> Vec<MusicEvent> {
    let mut events = Vec::new();
    if plan.perc == Perc::None {
        return events;
    }
    let hits: &[(&str, u32)] = match plan.perc {
        Perc::Soft => &[("kick", 0), ("hat", 2), ("hat", 6)],
        Perc::Drive => &[
            ("kick", 0),
            ("kick", 4),
            ("snare", 2),
            ("snare", 6),
            ("hat", 1),
            ("hat", 3),
            ("hat", 5),
            ("hat", 7),
        ],
        Perc::Dance => &[
            ("kick", 0),
            ("tom", 2),
            ("tom", 6),
            ("hat", 1),
            ("hat", 3),
            ("hat", 5),
            ("hat", 7),
        ],
        Perc::None => &[],
    };
    let velocity = 0.15 + traits.danger * 0.12 + plan.intensity * 0.06;
    let mut index = 0;
    for bar in 0..plan.bars {
        let bar_start = bar * bar_ticks;
        let mut bar_hits = hits.to_vec();
        if bar % 4 == 3 {
            bar_hits.extend_from_slice(fill_hits(plan.perc));
        }
        for (voice, step) in &bar_hits {
            push_perc(
                &mut events,
                plan.id,
                "percussion",
                index,
                bar_start + *step * pulse,
                pulse / 2,
                velocity,
                voice,
            );
            index += 1;
        }
    }
    events
}

#[allow(clippy::too_many_arguments)]
fn build_section(
    plan: &SectionPlan,
    style: &StyleKit,
    harmony: &HarmonyDna,
    motif: &[i32; 4],
    traits: &NormalizedTraits,
    bar_ticks: u32,
    pulse: u32,
    section_seed: u32,
) -> PortableSection {
    let intervals = mode_intervals(plan.mode);
    let roots = roots(plan, harmony);
    let mut rng = DeterministicRandom::new(section_seed);

    let mut events = Vec::new();
    events.extend(drone_events(
        plan,
        roots,
        &intervals,
        bar_ticks,
        role_voice(style, plan.drone_role),
    ));
    events.extend(harmony_events(
        plan,
        harmony,
        roots,
        &intervals,
        traits,
        bar_ticks,
        role_voice(style, plan.harmony_role),
    ));
    events.extend(arp_events(
        plan,
        harmony,
        roots,
        &intervals,
        bar_ticks,
        pulse,
        role_voice(style, Role::Pluck),
    ));
    events.extend(bass_events(
        plan, harmony, roots, &intervals, bar_ticks, pulse,
    ));
    events.extend(melody_events(
        plan,
        harmony,
        roots,
        &intervals,
        motif,
        traits,
        bar_ticks,
        pulse,
        &mut rng,
        role_voice(style, plan.melody_role),
    ));
    events.extend(percussion_events(plan, traits, bar_ticks, pulse));

    if plan.finale {
        let pitch = scale_pitch(roots.melody, 4, &intervals) + 12;
        push_note(
            &mut events,
            plan.id,
            "accent",
            0,
            0,
            bar_ticks,
            0.18,
            pitch,
            role_voice(style, Role::Glass),
            None,
        );
    }

    events.sort_by(|left, right| {
        left.start_tick()
            .cmp(&right.start_tick())
            .then_with(|| event_id(left).cmp(event_id(right)))
    });

    PortableSection {
        id: plan.id.to_string(),
        label: plan.label.to_string(),
        feeling: plan.feeling.to_string(),
        color: plan.color.to_string(),
        length_ticks: bar_ticks * plan.bars,
        events,
    }
}

/// Serialized selection rules for the portable score, mirroring
/// [`select_adventure_section`] exactly.
pub fn default_rules() -> Vec<AdaptiveRule> {
    fn rule(
        target: &str,
        priority: i32,
        phase: Option<&str>,
        numeric: Option<(&str, f64)>,
        quest: bool,
    ) -> AdaptiveRule {
        let mut numeric_map = serde_json::Map::new();
        if let Some((name, min)) = numeric {
            numeric_map.insert(name.into(), serde_json::json!({ "min": min }));
        }
        let mut categorical = serde_json::Map::new();
        if let Some(phase) = phase {
            categorical.insert("areaPhase".into(), serde_json::json!(phase));
        }
        if quest {
            numeric_map.insert("questComplete".into(), serde_json::json!({ "min": 1 }));
        }
        AdaptiveRule {
            target: target.to_string(),
            priority,
            when: AdaptiveCondition {
                numeric: serde_json::Value::Object(numeric_map),
                categorical: serde_json::Value::Object(categorical),
            },
            hold: None,
        }
    }

    vec![
        rule("quest-complete", 100, Some("complete"), None, false),
        rule("quest-complete", 95, None, None, true),
        rule("sanctuary", 90, None, Some(("discovery", 0.85)), false),
        rule("danger", 80, None, Some(("threat", 0.7)), false),
        rule("danger", 70, Some("danger"), None, false),
        rule("clue", 60, None, Some(("discovery", 0.3)), false),
        rule("clue", 50, Some("clue"), None, false),
        rule("explore", 30, Some("explore"), None, false),
        rule("camp", 10, Some("camp"), None, false),
    ]
}

/// Pick the section for an adventure state. Deterministic and total; the
/// default is `camp`.
pub fn select_adventure_section(state: &AdventureState) -> &'static str {
    if state.quest_complete || state.area_phase == "complete" {
        "quest-complete"
    } else if state.discovery >= 0.85 {
        "sanctuary"
    } else if state.threat >= 0.7 || state.area_phase == "danger" {
        "danger"
    } else if state.discovery >= 0.3 || state.area_phase == "clue" {
        "clue"
    } else if state.area_phase == "explore" {
        "explore"
    } else {
        "camp"
    }
}

pub fn generate_adventure(input: &AdventureInput) -> Result<PortableScore, String> {
    let traits = normalize(input);
    let style = input.style;
    let kit = style_kit(style);

    let harmony = create_harmony(subseed(&input.secret, &input.seed, "harmony"));
    let motif = create_motif(subseed(&input.secret, &input.seed, "motif"));

    let bpm = (kit.base_bpm + traits.motion * 20.0 - traits.mystery * 8.0)
        .clamp(64.0, 132.0)
        .round();

    let ticks_per_beat = 960u32;
    let beats_per_bar = 4u32;
    let bar_ticks = ticks_per_beat * beats_per_bar;
    let pulse = ticks_per_beat / 2;

    let sections: Vec<PortableSection> = PLANS
        .iter()
        .map(|plan| {
            build_section(
                plan,
                &kit,
                &harmony,
                &motif,
                &traits,
                bar_ticks,
                pulse,
                subseed(&input.secret, &input.seed, plan.id),
            )
        })
        .collect();

    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: score_id(&input.secret, &input.seed, style, &traits),
        title: format!("{} {} Trail", style.display(), harmony.key.to_uppercase()),
        bpm,
        beats_per_bar,
        ticks_per_beat,
        crossfade_bars: 2.0,
        default_section: "camp".to_string(),
        sections,
        rules: default_rules(),
        form: None,
    };
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        generate_adventure, select_adventure_section, AdventureInput, AdventureStyle,
        GENERATOR_VERSION,
    };
    use crate::score::{AdventureState, MusicEvent};

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
    fn generates_six_sections_with_events() {
        let score = generate_adventure(&sample("camp-secret", AdventureStyle::Campfire))
            .expect("score must validate");
        assert_eq!(score.sections.len(), 6);
        assert_eq!(score.default_section, "camp");
        for expected in [
            "camp",
            "explore",
            "clue",
            "danger",
            "sanctuary",
            "quest-complete",
        ] {
            let section = score.section(expected).expect("section exists");
            assert!(section.events.len() > 8, "{expected} should carry events");
        }
        let percussion = |id: &str| {
            score
                .section(id)
                .unwrap()
                .events
                .iter()
                .filter(|event| matches!(event, MusicEvent::Percussion { .. }))
                .count()
        };
        assert!(percussion("danger") > percussion("camp"));
    }

    #[test]
    fn secret_and_style_change_the_piece() {
        let first = generate_adventure(&sample("trail-a", AdventureStyle::Wilds)).unwrap();
        let second = generate_adventure(&sample("trail-b", AdventureStyle::Wilds)).unwrap();
        let ruins = generate_adventure(&sample("trail-a", AdventureStyle::Ruins)).unwrap();
        assert_ne!(first.id, second.id);
        assert_ne!(first.id, ruins.id);
        assert_ne!(first.bpm, ruins.bpm);
    }

    #[test]
    fn generation_is_deterministic() {
        let input = sample("deterministic", AdventureStyle::Campfire);
        let first = generate_adventure(&input).unwrap();
        let second = generate_adventure(&input).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
    }

    #[test]
    fn many_seeds_are_valid_and_unique() {
        assert_eq!(GENERATOR_VERSION, "1.0.0");
        let styles = [
            AdventureStyle::Campfire,
            AdventureStyle::Wilds,
            AdventureStyle::Ruins,
        ];
        let mut ids = HashSet::new();
        for index in 0..192 {
            let input = AdventureInput {
                secret: "stress".into(),
                seed: format!("trail-{index}"),
                style: styles[index % styles.len()],
                wonder: f64::from((index % 17) as u32) / 16.0,
                danger: f64::from((index % 11) as u32) / 10.0,
                mystery: f64::from((index % 13) as u32) / 12.0,
                motion: f64::from((index % 19) as u32) / 18.0,
            };
            let score = generate_adventure(&input).expect("stress score must validate");
            assert!(ids.insert(score.id.clone()), "duplicate id {}", score.id);
        }
        assert_eq!(ids.len(), 192);
    }

    #[test]
    fn selector_and_serialized_rules_agree_across_states() {
        use super::default_rules;
        let phases = ["camp", "explore", "clue", "danger", "complete", "unknown"];
        for phase in phases {
            for (discovery, threat) in [(0.0, 0.0), (0.4, 0.2), (0.9, 0.2), (0.9, 0.9), (0.2, 0.8)]
            {
                for quest_complete in [false, true] {
                    let state = AdventureState {
                        area_phase: phase.into(),
                        discovery,
                        threat,
                        quest_complete,
                    };
                    let selected = select_adventure_section(&state);
                    let ruled = default_rules()
                        .into_iter()
                        .find(|rule| {
                            let phase_ok = rule
                                .when
                                .categorical
                                .get("areaPhase")
                                .is_none_or(|expected| expected == &state.area_phase);
                            let numeric_ok =
                                rule.when.numeric.as_object().is_none_or(|conditions| {
                                    conditions.iter().all(|(name, spec)| {
                                        let Some(min) =
                                            spec.get("min").and_then(serde_json::Value::as_f64)
                                        else {
                                            return true;
                                        };
                                        match name.as_str() {
                                            "discovery" => discovery >= min,
                                            "threat" => threat >= min,
                                            "questComplete" => {
                                                (if state.quest_complete { 1.0 } else { 0.0 })
                                                    >= min
                                            }
                                            _ => true,
                                        }
                                    })
                                });
                            phase_ok && numeric_ok
                        })
                        .map(|rule| rule.target)
                        .unwrap_or_else(|| "camp".to_string());
                    assert_eq!(
                        selected, ruled,
                        "{phase} d={discovery} t={threat} q={quest_complete}"
                    );
                }
            }
        }
    }

    #[test]
    fn discovery_and_threat_drive_the_section() {
        let state = |phase: &str, discovery: f64, threat: f64| AdventureState {
            area_phase: phase.into(),
            discovery,
            threat,
            quest_complete: false,
        };
        assert_eq!(select_adventure_section(&state("camp", 0.0, 0.0)), "camp");
        assert_eq!(
            select_adventure_section(&state("explore", 0.0, 0.0)),
            "explore"
        );
        assert_eq!(
            select_adventure_section(&state("explore", 0.4, 0.0)),
            "clue"
        );
        assert_eq!(
            select_adventure_section(&state("explore", 0.0, 0.9)),
            "danger"
        );
        assert_eq!(
            select_adventure_section(&state("explore", 0.9, 0.0)),
            "sanctuary"
        );
        assert_eq!(
            select_adventure_section(&AdventureState {
                area_phase: "danger".into(),
                discovery: 0.2,
                threat: 0.1,
                quest_complete: true,
            }),
            "quest-complete"
        );
    }
}
