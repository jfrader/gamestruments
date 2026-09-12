//! Medieval fantasy-RPG recipe.
//!
//! Seven adaptive sections (`explore`, `town`, `dungeon`, `combat`, `boss`,
//! `tavern`, `victory`) that a game selects by `scene`, each a stable modal
//! loop with bar-quantized crossfades. Sibling of `racing` and `suspense`:
//! same deterministic DNA pattern, same portable-score output, no samples.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    AdaptiveCondition, AdaptiveRule, MedievalState, MusicEvent, PortableScore, PortableSection,
    SCORE_SCHEMA_VERSION,
};
use crate::theory::{json_num, mode_intervals, phrase_gain, scale_pitch, NOTE_NAMES};

pub const GENERATOR_VERSION: &str = "1.0.0";
pub const DNA_SEED_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MedievalStyle {
    Court,
    Minstrel,
    Chapel,
}

impl MedievalStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "court" => Ok(Self::Court),
            "minstrel" => Ok(Self::Minstrel),
            "chapel" => Ok(Self::Chapel),
            other => Err(format!("Unknown medieval style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Court => "court",
            Self::Minstrel => "minstrel",
            Self::Chapel => "chapel",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Court => "Court",
            Self::Minstrel => "Minstrel",
            Self::Chapel => "Chapel",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MedievalInput {
    pub secret: String,
    pub seed: String,
    pub style: MedievalStyle,
    pub valor: f64,
    pub mystery: f64,
    pub warmth: f64,
    pub motion: f64,
}

#[derive(Clone, Copy)]
struct NormalizedTraits {
    valor: f64,
    mystery: f64,
    warmth: f64,
    motion: f64,
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn normalize(input: &MedievalInput) -> NormalizedTraits {
    NormalizedTraits {
        valor: clamp_unit(input.valor),
        mystery: clamp_unit(input.mystery),
        warmth: clamp_unit(input.warmth),
        motion: clamp_unit(input.motion),
    }
}

/// Instrument roles the style kit resolves to concrete synth voices.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Lead,
    Pluck,
    Chant,
    Fanfare,
    Strings,
    Organ,
    Low,
    Pad,
    Bell,
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
    chant: &'static str,
    fanfare: &'static str,
    strings: &'static str,
    organ: &'static str,
    low: &'static str,
    pad: &'static str,
    bell: &'static str,
    base_bpm: f64,
}

fn style_kit(style: MedievalStyle) -> StyleKit {
    match style {
        MedievalStyle::Court => StyleKit {
            lead: "vielle",
            pluck: "harp",
            chant: "organ",
            fanfare: "warm",
            strings: "vielle",
            organ: "organ",
            low: "dusk",
            pad: "felt",
            bell: "bell",
            base_bpm: 92.0,
        },
        MedievalStyle::Minstrel => StyleKit {
            lead: "recorder",
            pluck: "harp",
            chant: "recorder",
            fanfare: "warm",
            strings: "harp",
            organ: "organ",
            low: "dusk",
            pad: "felt",
            bell: "bell",
            base_bpm: 104.0,
        },
        MedievalStyle::Chapel => StyleKit {
            lead: "organ",
            pluck: "harp",
            chant: "organ",
            fanfare: "warm",
            strings: "vielle",
            organ: "organ",
            low: "dusk",
            pad: "felt",
            bell: "bell",
            base_bpm: 74.0,
        },
    }
}

fn role_voice(kit: &StyleKit, role: Role) -> &'static str {
    match role {
        Role::Lead => kit.lead,
        Role::Pluck => kit.pluck,
        Role::Chant => kit.chant,
        Role::Fanfare => kit.fanfare,
        Role::Strings => kit.strings,
        Role::Organ => kit.organ,
        Role::Low => kit.low,
        Role::Pad => kit.pad,
        Role::Bell => kit.bell,
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
    fanfare: bool,
    melody_role: Role,
    harmony_role: Role,
    drone_role: Role,
}

const PLANS: [SectionPlan; 7] = [
    SectionPlan {
        id: "explore",
        label: "Explore",
        feeling: "open road / unhurried wonder",
        color: "#8fbf6b",
        bars: 8,
        intensity: 0.6,
        mode: "dorian",
        register: 0,
        density: 0.5,
        perc: Perc::Soft,
        drone: true,
        fanfare: false,
        melody_role: Role::Lead,
        harmony_role: Role::Pluck,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "town",
        label: "Town",
        feeling: "warm hearth / welcome",
        color: "#e0b04e",
        bars: 8,
        intensity: 0.55,
        mode: "ionian",
        register: 0,
        density: 0.58,
        perc: Perc::Soft,
        drone: false,
        fanfare: false,
        melody_role: Role::Pluck,
        harmony_role: Role::Pluck,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "dungeon",
        label: "Dungeon",
        feeling: "cold stone / held breath",
        color: "#5b6b7a",
        bars: 8,
        intensity: 0.32,
        mode: "aeolian",
        register: -7,
        density: 0.26,
        perc: Perc::None,
        drone: true,
        fanfare: false,
        melody_role: Role::Chant,
        harmony_role: Role::Strings,
        drone_role: Role::Low,
    },
    SectionPlan {
        id: "combat",
        label: "Combat",
        feeling: "steel drawn / forward drive",
        color: "#c96a4a",
        bars: 8,
        intensity: 0.88,
        mode: "dorian",
        register: 0,
        density: 0.78,
        perc: Perc::Drive,
        drone: true,
        fanfare: false,
        melody_role: Role::Lead,
        harmony_role: Role::Strings,
        drone_role: Role::Low,
    },
    SectionPlan {
        id: "boss",
        label: "Boss",
        feeling: "dread / no retreat",
        color: "#8e3b4a",
        bars: 8,
        intensity: 1.0,
        mode: "phrygian",
        register: -7,
        density: 0.84,
        perc: Perc::Drive,
        drone: true,
        fanfare: false,
        melody_role: Role::Chant,
        harmony_role: Role::Organ,
        drone_role: Role::Low,
    },
    SectionPlan {
        id: "tavern",
        label: "Tavern",
        feeling: "dance / raised cup",
        color: "#d98f4e",
        bars: 8,
        intensity: 0.7,
        mode: "mixolydian",
        register: 7,
        density: 0.72,
        perc: Perc::Dance,
        drone: false,
        fanfare: false,
        melody_role: Role::Pluck,
        harmony_role: Role::Pluck,
        drone_role: Role::Pad,
    },
    SectionPlan {
        id: "victory",
        label: "Victory",
        feeling: "bright cadence / earned rest",
        color: "#f0d876",
        bars: 8,
        intensity: 0.82,
        mode: "ionian",
        register: 12,
        density: 0.62,
        perc: Perc::Soft,
        drone: false,
        fanfare: true,
        melody_role: Role::Fanfare,
        harmony_role: Role::Strings,
        drone_role: Role::Pad,
    },
];

const KEY_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const PROGRESSIONS: [&[i32; 4]; 4] = [&[0, 6, 3, 4], &[0, 3, 6, 0], &[0, 5, 3, 4], &[0, 4, 6, 3]];
const MOTIFS: [&[i32; 4]; 4] = [&[0, 2, 3, 1], &[0, 3, 4, 2], &[0, 1, 4, 2], &[0, 4, 3, 1]];
const ONSET_CELLS: [&[u32]; 6] = [
    &[0, 2, 4, 6],
    &[0, 3, 4, 6],
    &[0, 1, 4, 6],
    &[0, 4, 6],
    &[0, 2, 4],
    &[0, 3, 6],
];
const FANFARE_ONSETS: [u32; 4] = [0, 2, 4, 6];

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

fn score_id(secret: &str, seed: &str, style: MedievalStyle, traits: &NormalizedTraits) -> String {
    let trait_json = format!(
        r#"{{"valor":{},"mystery":{},"warmth":{},"motion":{}}}"#,
        json_num(traits.valor),
        json_num(traits.mystery),
        json_num(traits.warmth),
        json_num(traits.motion)
    );
    let identity = hash_text(&format!(
        "{secret}\0{seed}\0{}\0{trait_json}\0{GENERATOR_VERSION}",
        style.as_str()
    ));
    format!(
        "medieval-generated-v{}-{identity:08x}",
        GENERATOR_VERSION.replace('.', "-")
    )
}

/// Per-section register roots: bass (C2), drone, harmony, and melody lanes.
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
        0.11,
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
        0.1,
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
    let degrees: Vec<i32> = if traits.warmth > 0.55 {
        vec![0, 2, 4]
    } else {
        vec![0, 4]
    };
    for bar in 0..plan.bars {
        let degree = harmony.progression[bar as usize % harmony.progression.len()];
        let bar_start = bar * bar_ticks;
        for offset in &degrees {
            let pitch = scale_pitch(roots.harmony, degree + offset, intervals);
            let velocity = (0.12 + traits.warmth * 0.06) * phrase_gain(bar);
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
        "explore" => [0, 2, 4, 2, 4, 2, 1, 0],
        "town" => [0, 4, 2, 4, 2, 0, 0, 0],
        "dungeon" => [0, 0, 2, 0, 2, 0, 0, 0],
        "combat" => [0, 4, 2, 4, 6, 4, 2, 0],
        "boss" => [0, 4, 1, 4, 4, 2, 1, 0],
        "tavern" => [0, 4, 2, 4, 4, 6, 4, 0],
        "victory" => [0, 4, 4, 6, 7, 4, 4, 0],
        _ => [0, 2, 4, 2, 4, 2, 1, 0],
    }
}

/// Plucked styles get a broken-chord accompaniment; the held harmony is the pad.
fn wants_arp(section: &str) -> bool {
    matches!(section, "explore" | "town" | "tavern" | "victory")
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
    let chosen: &[u32] = if plan.fanfare {
        &FANFARE_ONSETS
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
        // Land the loop: drop the final onset so the last phrase holds.
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
            let velocity = (0.2 + traits.valor * 0.08 + plan.intensity * 0.06) * gain;
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
    let velocity = 0.16 + traits.valor * 0.12 + plan.intensity * 0.05;
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

    if plan.fanfare {
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
            role_voice(style, Role::Bell),
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
/// [`select_medieval_section`] exactly.
pub fn default_rules() -> Vec<AdaptiveRule> {
    fn rule(
        target: &str,
        priority: i32,
        scene: Option<&str>,
        danger_min: Option<f64>,
    ) -> AdaptiveRule {
        let mut numeric = serde_json::Map::new();
        if let Some(min) = danger_min {
            numeric.insert("danger".into(), serde_json::json!({ "min": min }));
        }
        let mut categorical = serde_json::Map::new();
        if let Some(scene) = scene {
            categorical.insert("scene".into(), serde_json::json!(scene));
        }
        AdaptiveRule {
            target: target.to_string(),
            priority,
            when: AdaptiveCondition {
                numeric: serde_json::Value::Object(numeric),
                categorical: serde_json::Value::Object(categorical),
            },
            hold: None,
        }
    }

    vec![
        rule("victory", 100, Some("victory"), None),
        rule("boss", 92, Some("combat"), Some(0.8)),
        rule("boss", 90, Some("boss"), None),
        rule("combat", 80, Some("combat"), None),
        rule("dungeon", 70, Some("dungeon"), None),
        rule("tavern", 60, Some("tavern"), None),
        rule("town", 50, Some("town"), None),
        rule("explore", 10, Some("explore"), None),
    ]
}

/// Pick the section for a scene state. Deterministic and total.
pub fn select_medieval_section(state: &MedievalState) -> &'static str {
    match state.scene.as_str() {
        "victory" => "victory",
        "boss" => "boss",
        "combat" if state.danger >= 0.8 => "boss",
        "combat" => "combat",
        "dungeon" => "dungeon",
        "tavern" => "tavern",
        "town" => "town",
        _ => "explore",
    }
}

pub fn generate_medieval(input: &MedievalInput) -> Result<PortableScore, String> {
    let traits = normalize(input);
    let style = input.style;
    let kit = style_kit(style);

    let harmony = create_harmony(subseed(&input.secret, &input.seed, "harmony"));
    let motif = create_motif(subseed(&input.secret, &input.seed, "motif"));

    let bpm = (kit.base_bpm + traits.valor * 22.0 - traits.mystery * 8.0)
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
        title: format!("{} {} Quest", style.display(), harmony.key.to_uppercase()),
        bpm,
        beats_per_bar,
        ticks_per_beat,
        crossfade_bars: 1.0,
        default_section: "explore".to_string(),
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
        generate_medieval, select_medieval_section, MedievalInput, MedievalStyle, GENERATOR_VERSION,
    };
    use crate::score::MedievalState;

    fn sample(secret: &str, style: MedievalStyle) -> MedievalInput {
        MedievalInput {
            secret: secret.into(),
            seed: "realm-01".into(),
            style,
            valor: 0.6,
            mystery: 0.55,
            warmth: 0.5,
            motion: 0.65,
        }
    }

    #[test]
    fn generates_seven_sections_with_events() {
        let score = generate_medieval(&sample("court-secret", MedievalStyle::Court))
            .expect("score must validate");
        assert_eq!(score.sections.len(), 7);
        assert_eq!(score.default_section, "explore");
        for expected in [
            "explore", "town", "dungeon", "combat", "boss", "tavern", "victory",
        ] {
            let section = score.section(expected).expect("section exists");
            assert!(section.events.len() > 8, "{expected} should carry events");
        }
        assert!(
            score.section("combat").unwrap().events.len()
                > score.section("dungeon").unwrap().events.len()
        );
    }

    #[test]
    fn secret_and_style_change_the_piece() {
        let first = generate_medieval(&sample("realm-a", MedievalStyle::Minstrel)).unwrap();
        let second = generate_medieval(&sample("realm-b", MedievalStyle::Minstrel)).unwrap();
        let chapel = generate_medieval(&sample("realm-a", MedievalStyle::Chapel)).unwrap();
        assert_ne!(first.id, second.id);
        assert_ne!(first.id, chapel.id);
        assert_ne!(first.bpm, chapel.bpm);
    }

    #[test]
    fn generation_is_deterministic() {
        let input = sample("deterministic", MedievalStyle::Court);
        let first = generate_medieval(&input).unwrap();
        let second = generate_medieval(&input).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
    }

    #[test]
    fn many_seeds_are_valid_and_unique() {
        assert_eq!(GENERATOR_VERSION, "1.0.0");
        let styles = [
            MedievalStyle::Court,
            MedievalStyle::Minstrel,
            MedievalStyle::Chapel,
        ];
        let mut ids = HashSet::new();
        for index in 0..192 {
            let input = MedievalInput {
                secret: "stress".into(),
                seed: format!("realm-{index}"),
                style: styles[index % styles.len()],
                valor: f64::from((index % 17) as u32) / 16.0,
                mystery: f64::from((index % 11) as u32) / 10.0,
                warmth: f64::from((index % 13) as u32) / 12.0,
                motion: f64::from((index % 19) as u32) / 18.0,
            };
            let score = generate_medieval(&input).expect("stress score must validate");
            assert!(ids.insert(score.id.clone()), "duplicate id {}", score.id);
        }
        assert_eq!(ids.len(), 192);
    }

    #[test]
    fn selector_and_serialized_rules_agree_across_states() {
        use super::default_rules;
        let scenes = [
            "explore", "town", "dungeon", "combat", "boss", "tavern", "victory", "unknown",
        ];
        for scene in scenes {
            for danger in [0.0, 0.5, 0.8, 1.0] {
                let state = MedievalState {
                    scene: scene.into(),
                    danger,
                };
                let selected = select_medieval_section(&state);
                let ruled = default_rules()
                    .into_iter()
                    .find(|rule| {
                        let scene_ok = rule
                            .when
                            .categorical
                            .get("scene")
                            .is_none_or(|expected| expected == &state.scene);
                        let danger_ok = rule
                            .when
                            .numeric
                            .get("danger")
                            .is_none_or(|spec| danger >= spec["min"].as_f64().unwrap());
                        scene_ok && danger_ok
                    })
                    .map(|rule| rule.target)
                    .unwrap_or_else(|| "explore".to_string());
                assert_eq!(selected, ruled, "{scene} danger={danger}");
            }
        }
    }

    #[test]
    fn hard_combat_escalates_to_boss() {
        assert_eq!(
            select_medieval_section(&MedievalState {
                scene: "combat".into(),
                danger: 0.85,
            }),
            "boss"
        );
        assert_eq!(
            select_medieval_section(&MedievalState {
                scene: "combat".into(),
                danger: 0.4,
            }),
            "combat"
        );
    }
}
