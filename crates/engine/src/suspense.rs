use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    AdaptiveCondition, AdaptiveRule, MusicEvent, PortableScore, PortableSection, SongForm,
    SongFormStep, TraceState, SCORE_SCHEMA_VERSION,
};

pub const GENERATOR_VERSION: &str = "2.1.1";
pub const DNA_SEED_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspenseStyle {
    Terminal,
    Cipher,
    Noir,
    /// The Suspense composition on club instruments.
    Techno,
    /// The Suspense composition on trance instruments.
    Trance,
}

impl SuspenseStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "terminal" => Ok(Self::Terminal),
            "cipher" => Ok(Self::Cipher),
            "noir" => Ok(Self::Noir),
            "techno" => Ok(Self::Techno),
            "trance" => Ok(Self::Trance),
            other => Err(format!("Unknown suspense style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Cipher => "cipher",
            Self::Noir => "noir",
            Self::Techno => "techno",
            Self::Trance => "trance",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::Cipher => "Cipher",
            Self::Noir => "Noir",
            Self::Techno => "Techno",
            Self::Trance => "Trance",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SuspenseInput {
    pub secret: String,
    pub seed: String,
    pub style: SuspenseStyle,
    pub tension: f64,
    pub heat: f64,
    pub mystery: f64,
    pub pulse: f64,
}

struct NormalizedTraits {
    tension: f64,
    heat: f64,
    mystery: f64,
    pulse: f64,
}

#[derive(Clone, Copy)]
struct SectionPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    bars: u32,
    role: SectionRole,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SectionRole {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    PostChorus,
    Interlude,
    Bridge,
    BridgeB,
    Break,
    Solo,
    Outro,
    Coda,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CueWeight {
    Idle,
    Drop,
    Work,
    Drive,
    Alarm,
}

const NOTE_NAMES: [&str; 12] = [
    "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
];
const KEY_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
const AEOLIAN: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
const CELLS: [[i32; 3]; 4] = [[0, 7, -2], [0, 7, -1], [0, 3, 7], [0, -5, 7]];

const PLANS: [SectionPlan; 14] = [
    SectionPlan {
        id: "intro",
        label: "Handshake",
        feeling: "drone and one interval",
        color: "#6b7c8a",
        bars: 8,
        role: SectionRole::Intro,
    },
    SectionPlan {
        id: "verse",
        label: "Scan",
        feeling: "the machine enters under the cell",
        color: "#7aa7b8",
        bars: 8,
        role: SectionRole::Verse,
    },
    SectionPlan {
        id: "pre-chorus",
        label: "Approach",
        feeling: "clock on, still one chord",
        color: "#c4a35a",
        bars: 8,
        role: SectionRole::PreChorus,
    },
    SectionPlan {
        id: "chorus",
        label: "Breach",
        feeling: "pulse plus the cell, no extra tune",
        color: "#e07a5f",
        bars: 8,
        role: SectionRole::Chorus,
    },
    SectionPlan {
        id: "break",
        label: "Break",
        feeling: "the floor drops out",
        color: "#4a5560",
        bars: 8,
        role: SectionRole::Break,
    },
    SectionPlan {
        id: "verse-b",
        label: "Second Pass",
        feeling: "pulse drops out for a breath",
        color: "#6f9eae",
        bars: 8,
        role: SectionRole::Verse,
    },
    SectionPlan {
        id: "post-chorus",
        label: "Echo",
        feeling: "cell over leftover pulse",
        color: "#d98973",
        bars: 8,
        role: SectionRole::PostChorus,
    },
    SectionPlan {
        id: "interlude",
        label: "Wait State",
        feeling: "only the machine",
        color: "#8a8f7a",
        bars: 8,
        role: SectionRole::Interlude,
    },
    SectionPlan {
        id: "bridge",
        label: "Complication",
        feeling: "tighter clock, still the same cell",
        color: "#c94f4f",
        bars: 8,
        role: SectionRole::Bridge,
    },
    SectionPlan {
        id: "bridge-b",
        label: "Other Hall",
        feeling: "the cell inverted, no arp",
        color: "#8b4a62",
        bars: 8,
        role: SectionRole::BridgeB,
    },
    SectionPlan {
        id: "solo",
        label: "Decrypt",
        feeling: "the cell alone, higher",
        color: "#f2d08b",
        bars: 8,
        role: SectionRole::Solo,
    },
    SectionPlan {
        id: "chorus-final",
        label: "Full Breach",
        feeling: "pulse and cell, one octave lift",
        color: "#ef9460",
        bars: 8,
        role: SectionRole::Chorus,
    },
    SectionPlan {
        id: "outro",
        label: "Disconnect",
        feeling: "layers peel",
        color: "#8b9aa6",
        bars: 8,
        role: SectionRole::Outro,
    },
    SectionPlan {
        id: "coda",
        label: "Closed Session",
        feeling: "drone, then nothing",
        color: "#d7c38a",
        bars: 8,
        role: SectionRole::Coda,
    },
];

struct HarmonyDna {
    key: String,
    root_pitch_class: i32,
}

struct MotifDna {
    cell: [i32; 3],
}

struct TimbreDna {
    drone_voice: String,
    cell_voice: String,
    pulse_voice: String,
    arp_voice: String,
}

struct ArrangementDna {
    bpm: f64,
}

fn clamp_unit(value: f64) -> f64 {
    if !value.is_finite() {
        0.5
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn normalize(input: &SuspenseInput) -> NormalizedTraits {
    NormalizedTraits {
        tension: clamp_unit(input.tension),
        heat: clamp_unit(input.heat),
        mystery: clamp_unit(input.mystery),
        pulse: clamp_unit(input.pulse),
    }
}

fn subseed(secret: &str, seed: &str, domain: &str) -> u32 {
    hash_text(&format!(
        "{DNA_SEED_VERSION}\0string:{seed}\0{domain}\0{secret}\0suspense"
    ))
}

fn create_harmony(seed: u32) -> HarmonyDna {
    let mut random = DeterministicRandom::new(seed);
    let root = *random.pick(&KEY_PITCH_CLASSES);
    HarmonyDna {
        key: NOTE_NAMES[root as usize].to_string(),
        root_pitch_class: root,
    }
}

fn create_motif(seed: u32) -> MotifDna {
    let mut random = DeterministicRandom::new(seed);
    MotifDna {
        cell: *random.pick(&CELLS),
    }
}

/// The one Suspense composition is written in Terminal's instruments; every
/// other style is a sound world applied afterwards ([`apply_sound_world`]).
fn reference_timbre() -> TimbreDna {
    TimbreDna {
        drone_voice: "warm".into(),
        cell_voice: "glass".into(),
        pulse_voice: "pulse".into(),
        arp_voice: "pulse".into(),
    }
}

/// The parts of the composition a sound world re-instruments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Drone,
    Cell,
    Pulse,
    Arp,
    Pad,
}

/// A style's instruments: one voice per part and its kick and snare. The notes,
/// rhythms, lengths and form never change between worlds.
struct SoundWorld {
    drone: &'static str,
    cell: &'static str,
    pulse: &'static str,
    arp: &'static str,
    pad: &'static str,
    kick: &'static str,
    snare: &'static str,
}

impl SoundWorld {
    fn voice(&self, part: Part) -> &'static str {
        match part {
            Part::Drone => self.drone,
            Part::Cell => self.cell,
            Part::Pulse => self.pulse,
            Part::Arp => self.arp,
            Part::Pad => self.pad,
        }
    }
}

impl SuspenseStyle {
    /// The style's sound world; Terminal is the reference the piece is written in.
    fn sound_world(self) -> Option<SoundWorld> {
        let club = |cell| SoundWorld {
            drone: "trance-pad",
            cell,
            pulse: "saw-bass",
            arp: "trance-lead",
            pad: "trance-pad",
            kick: "techno-kick",
            snare: "clap",
        };
        match self {
            Self::Terminal => None,
            Self::Cipher => Some(SoundWorld {
                drone: "warm",
                cell: "pluck",
                pulse: "bass",
                arp: "glass",
                pad: "dusk",
                kick: "kick",
                snare: "snare",
            }),
            Self::Noir => Some(SoundWorld {
                drone: "organ",
                cell: "epiano",
                pulse: "bass",
                arp: "warm",
                pad: "dusk",
                kick: "kick",
                snare: "snare",
            }),
            Self::Techno => Some(club("stab")),
            Self::Trance => Some(club("trance-lead")),
        }
    }
}

/// The part an event plays, from its lane (`{section}-{part}`), falling back
/// to its reference voice for lanes that carry no part name.
fn part_of(section: &str, lane: &str, voice: &str) -> Option<Part> {
    let name = lane
        .strip_prefix(section)
        .and_then(|rest| rest.strip_prefix('-'))
        .unwrap_or(lane);
    match name {
        "drone" | "anchor" | "drone-upper" | "extension-drone" => Some(Part::Drone),
        "cell" | "echo-cells" | "hook" => Some(Part::Cell),
        "pulse" | "bass" | "extension-pulse" | "fractured-pulse" | "response-pulse" => {
            Some(Part::Pulse)
        }
        "arp" => Some(Part::Arp),
        "pad" | "answer" | "extension-echo" | "response-echo" => Some(Part::Pad),
        _ => match voice {
            "warm" | "organ" => Some(Part::Drone),
            "glass" | "pluck" | "epiano" => Some(Part::Cell),
            "pulse" | "bass" | "felt" => Some(Part::Pulse),
            "dusk" => Some(Part::Pad),
            _ => None,
        },
    }
}

/// Re-instrument the whole score in the style's sound world: every part and
/// the kick and snare move to the world's instruments; nothing else changes.
pub(crate) fn apply_sound_world(score: &mut PortableScore, style: SuspenseStyle) {
    let Some(world) = style.sound_world() else {
        return;
    };
    for section in &mut score.sections {
        for event in &mut section.events {
            match event {
                MusicEvent::Note { lane, voice, .. } => {
                    if let Some(part) = part_of(&section.id, lane, voice) {
                        *voice = world.voice(part).into();
                    }
                }
                MusicEvent::Percussion { voice, .. } => match voice.as_str() {
                    "kick" => *voice = world.kick.into(),
                    "snare" => *voice = world.snare.into(),
                    _ => {}
                },
            }
        }
    }
}

fn create_arrangement(
    seed: u32,
    style: SuspenseStyle,
    traits: &NormalizedTraits,
) -> ArrangementDna {
    let mut random = DeterministicRandom::new(seed);
    let jitter = f64::from(random.integer(3)) - 1.0;
    let base = match style {
        SuspenseStyle::Noir => 64.0,
        SuspenseStyle::Terminal => 72.0,
        SuspenseStyle::Cipher => 78.0,
        SuspenseStyle::Techno => 84.0,
        SuspenseStyle::Trance => 88.0,
    };
    ArrangementDna {
        bpm: (base + traits.pulse * 8.0 + jitter).clamp(60.0, 88.0),
    }
}

fn midi(pitch: i32) -> u8 {
    pitch.clamp(28, 91) as u8
}

fn cue_weight(plan: &SectionPlan) -> CueWeight {
    match plan.role {
        SectionRole::Intro | SectionRole::Outro | SectionRole::Coda => CueWeight::Idle,
        SectionRole::Break => CueWeight::Drop,
        SectionRole::Verse
        | SectionRole::PostChorus
        | SectionRole::Interlude
        | SectionRole::Solo => CueWeight::Work,
        SectionRole::PreChorus | SectionRole::Chorus | SectionRole::BridgeB => CueWeight::Drive,
        SectionRole::Bridge => CueWeight::Alarm,
    }
}

fn event_id(section: &str, lane: &str, index: usize) -> String {
    format!("{section}:{lane}:{index}")
}

#[allow(clippy::too_many_arguments)]
fn push_note(
    events: &mut Vec<MusicEvent>,
    section: &str,
    lane: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    pitch: u8,
    voice: &str,
    melody: bool,
) {
    let index = events.len();
    events.push(MusicEvent::Note {
        id: event_id(section, lane, index),
        section: section.to_string(),
        lane: lane.to_string(),
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

fn push_perc(
    events: &mut Vec<MusicEvent>,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    voice: &str,
) {
    let index = events.len();
    events.push(MusicEvent::Percussion {
        id: event_id(section, "kit", index),
        section: section.to_string(),
        lane: format!("{section}-kit"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.55),
        voice: voice.to_string(),
    });
}

fn scale_degree(root: i32, degree: i32) -> i32 {
    let idx = degree.rem_euclid(7);
    let oct = degree.div_euclid(7);
    root + oct * 12 + AEOLIAN[idx as usize]
}

fn cell_hits(weight: CueWeight, solo: bool) -> &'static [(usize, u32)] {
    if solo {
        return &[(0, 0), (2, 0), (4, 4), (6, 0)];
    }
    match weight {
        CueWeight::Idle => &[(1, 4), (3, 0), (5, 4), (7, 0)],
        CueWeight::Drop => &[(2, 0), (6, 4)],
        CueWeight::Work => &[(0, 4), (2, 0), (4, 4), (6, 0)],
        CueWeight::Drive | CueWeight::Alarm => &[(0, 4), (2, 0), (3, 4), (5, 0), (6, 4)],
    }
}

fn pulse_from_bar(weight: CueWeight, plan: &SectionPlan) -> Option<usize> {
    match plan.role {
        SectionRole::Solo | SectionRole::Intro | SectionRole::Coda | SectionRole::Break => None,
        SectionRole::Verse if plan.id == "verse-b" => None,
        SectionRole::Interlude => Some(0),
        _ => match weight {
            CueWeight::Idle | CueWeight::Drop => None,
            CueWeight::Work => Some(2),
            CueWeight::Drive | CueWeight::Alarm => Some(0),
        },
    }
}

fn has_arp(plan: &SectionPlan) -> bool {
    matches!(plan.role, SectionRole::Chorus | SectionRole::Bridge)
}

fn has_drone(plan: &SectionPlan) -> bool {
    !matches!(plan.role, SectionRole::Bridge | SectionRole::Interlude)
}

fn hat_from_bar(weight: CueWeight) -> Option<usize> {
    match weight {
        CueWeight::Idle | CueWeight::Drop => None,
        CueWeight::Work => Some(4),
        CueWeight::Drive => Some(2),
        CueWeight::Alarm => Some(0),
    }
}

fn kick_from_bar(weight: CueWeight) -> Option<usize> {
    match weight {
        CueWeight::Idle => Some(4),
        CueWeight::Drop => None,
        CueWeight::Work => Some(4),
        CueWeight::Drive | CueWeight::Alarm => Some(0),
    }
}

fn build_section(
    plan: &SectionPlan,
    harmony: &HarmonyDna,
    motif: &MotifDna,
    timbre: &TimbreDna,
    traits: &NormalizedTraits,
    bar_ticks: u32,
    pulse: u32,
) -> PortableSection {
    let mut events = Vec::new();
    let weight = cue_weight(plan);
    let section_len = bar_ticks * plan.bars;
    let root = 48 + harmony.root_pitch_class;
    let drone_pitch = midi(36 + harmony.root_pitch_class);
    let cell_root = if plan.role == SectionRole::Solo {
        root + 12
    } else if plan.id == "chorus-final" {
        root + 7
    } else {
        root
    };
    let pulse_root = 36 + harmony.root_pitch_class;
    let fifth = midi(pulse_root + 7);
    let tonic = midi(pulse_root);
    let vel = 0.14 + traits.tension * 0.12;

    if has_drone(plan) {
        push_note(
            &mut events,
            plan.id,
            &format!("{}-drone", plan.id),
            0,
            section_len.saturating_sub(pulse * 2).max(bar_ticks),
            vel * 0.7,
            drone_pitch,
            &timbre.drone_voice,
            false,
        );
        if matches!(weight, CueWeight::Drive | CueWeight::Alarm) {
            push_note(
                &mut events,
                plan.id,
                &format!("{}-drone", plan.id),
                0,
                section_len.saturating_sub(pulse * 2).max(bar_ticks),
                vel * 0.45,
                midi(root + 7),
                &timbre.drone_voice,
                false,
            );
        }
    }

    if let Some(start_bar) = pulse_from_bar(weight, plan) {
        for bar in start_bar..plan.bars as usize {
            let bar_start = bar as u32 * bar_ticks;
            for step in 0..8u32 {
                let pitch = if step.is_multiple_of(2) { tonic } else { fifth };
                push_note(
                    &mut events,
                    plan.id,
                    &format!("{}-pulse", plan.id),
                    bar_start + step * pulse,
                    pulse,
                    0.22 + traits.heat * 0.08,
                    pitch,
                    &timbre.pulse_voice,
                    false,
                );
            }
        }
    }

    if has_arp(plan) {
        let tones = [
            midi(scale_degree(root, 0)),
            midi(scale_degree(root, 2)),
            midi(scale_degree(root, 4)),
            midi(scale_degree(root, 8)),
        ];
        for bar in 0..plan.bars as usize {
            let bar_start = bar as u32 * bar_ticks;
            for step in 0..8u32 {
                let pitch = tones[(step as usize) % tones.len()];
                push_note(
                    &mut events,
                    plan.id,
                    &format!("{}-arp", plan.id),
                    bar_start + step * pulse,
                    pulse,
                    0.18,
                    pitch,
                    &timbre.arp_voice,
                    false,
                );
            }
        }
    }

    let hits = cell_hits(weight, plan.role == SectionRole::Solo);
    for (index, &(bar, step)) in hits.iter().enumerate() {
        let interval = motif.cell[index % motif.cell.len()];
        let interval = if plan.role == SectionRole::BridgeB {
            -interval
        } else {
            interval
        };
        let pitch = midi(cell_root + interval);
        let hold = if matches!(weight, CueWeight::Idle | CueWeight::Drop) || traits.mystery >= 0.6 {
            pulse * 6
        } else {
            pulse * 3
        };
        let start = bar as u32 * bar_ticks + step * pulse;
        let duration = hold.min(section_len.saturating_sub(start));
        push_note(
            &mut events,
            plan.id,
            &format!("{}-cell", plan.id),
            start,
            duration.max(pulse * 2),
            vel + 0.12,
            pitch,
            &timbre.cell_voice,
            true,
        );
    }

    if let Some(start_bar) = kick_from_bar(weight) {
        for bar in start_bar..plan.bars as usize {
            let bar_start = bar as u32 * bar_ticks;
            push_perc(
                &mut events,
                plan.id,
                bar_start,
                pulse,
                if bar == 0 { 0.42 } else { 0.28 },
                "kick",
            );
            if !matches!(weight, CueWeight::Idle) {
                push_perc(
                    &mut events,
                    plan.id,
                    bar_start + pulse * 4,
                    pulse,
                    0.22,
                    "kick",
                );
            }
        }
    }

    if let Some(start_bar) = hat_from_bar(weight) {
        for bar in start_bar..plan.bars as usize {
            let bar_start = bar as u32 * bar_ticks;
            for step in [1u32, 3, 5, 7] {
                push_perc(
                    &mut events,
                    plan.id,
                    bar_start + step * pulse,
                    pulse / 3,
                    0.16,
                    "hat",
                );
            }
        }
    }

    if plan.role == SectionRole::Bridge {
        push_perc(
            &mut events,
            plan.id,
            7 * bar_ticks + 6 * pulse,
            pulse,
            0.3,
            "tom",
        );
    }

    if plan.id == "verse" {
        push_perc(&mut events, plan.id, 0, pulse, 0.42, "kick");
    }

    events.sort_by(|left, right| {
        left.start_tick()
            .cmp(&right.start_tick())
            .then_with(|| event_sort_id(left).cmp(event_sort_id(right)))
    });

    PortableSection {
        id: plan.id.to_string(),
        label: plan.label.to_string(),
        feeling: plan.feeling.to_string(),
        color: plan.color.to_string(),
        length_ticks: section_len,
        events,
    }
}

fn event_sort_id(event: &MusicEvent) -> &str {
    match event {
        MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
    }
}

pub(crate) struct TraceSpec {
    section: &'static str,
    hold: bool,
    priority: i32,
    heat_min: Option<f64>,
    focus_min: Option<f64>,
    progress_min: Option<f64>,
    phase: Option<&'static str>,
}

const TRACE_SPECS: &[TraceSpec] = &[
    TraceSpec {
        section: "coda",
        hold: true,
        priority: 100,
        heat_min: None,
        focus_min: None,
        progress_min: None,
        phase: Some("complete"),
    },
    TraceSpec {
        section: "coda",
        hold: true,
        priority: 95,
        heat_min: None,
        focus_min: None,
        progress_min: Some(0.95),
        phase: None,
    },
    TraceSpec {
        section: "outro",
        hold: true,
        priority: 90,
        heat_min: None,
        focus_min: None,
        progress_min: None,
        phase: Some("extract"),
    },
    TraceSpec {
        section: "bridge",
        hold: false,
        priority: 80,
        heat_min: Some(0.75),
        focus_min: None,
        progress_min: None,
        phase: None,
    },
    TraceSpec {
        section: "bridge",
        hold: false,
        priority: 70,
        heat_min: None,
        focus_min: None,
        progress_min: None,
        phase: Some("alert"),
    },
    TraceSpec {
        section: "chorus",
        hold: false,
        priority: 60,
        heat_min: None,
        focus_min: Some(0.7),
        progress_min: None,
        phase: Some("exploit"),
    },
];

/// Choose a suspense section by evaluating the score's own serialized adaptive
/// rules (the same rules the browser transport evaluates), so native transport
/// stays in lock-step with what was actually generated. Original/Theme scores
/// serialize only the base rules, so the extended `outro` progress cue never
/// leaks into them.
pub(crate) fn select_trace_section(
    rules: &[AdaptiveRule],
    state: &TraceState,
) -> Option<(String, bool)> {
    let mut matches: Vec<(usize, &AdaptiveRule)> = rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| trace_rule_matches(rule, state))
        .collect();
    matches.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left_index.cmp(right_index))
    });
    matches
        .into_iter()
        .next()
        .map(|(_, rule)| (rule.target.clone(), rule.hold != Some(false)))
}

fn trace_rule_matches(rule: &AdaptiveRule, state: &TraceState) -> bool {
    if let Some(numeric) = rule.when.numeric.as_object() {
        for (name, range) in numeric {
            let current = match name.as_str() {
                "heat" => state.heat,
                "focus" => state.focus,
                "progress" => state.progress,
                // Unknown numeric keys have no corresponding TraceState field, so
                // the browser would treat the value as missing and fail closed.
                _ => return false,
            };
            if !current.is_finite() {
                return false;
            }
            let Some(spec) = range.as_object() else {
                continue;
            };
            if let Some(min) = spec.get("min").and_then(serde_json::Value::as_f64) {
                if current < min {
                    return false;
                }
            }
            if let Some(max) = spec.get("max").and_then(serde_json::Value::as_f64) {
                if current > max {
                    return false;
                }
            }
        }
    }
    if let Some(categorical) = rule.when.categorical.as_object() {
        for (name, accepted) in categorical {
            // The only categorical key a TraceState carries is `tracePhase`;
            // anything else has no corresponding state value and fails closed.
            if name != "tracePhase" {
                return false;
            }
            let matched = match accepted {
                serde_json::Value::String(expected) => expected == &state.phase,
                serde_json::Value::Array(values) => values.iter().any(|value| {
                    value
                        .as_str()
                        .is_some_and(|expected| expected == state.phase)
                }),
                _ => false,
            };
            if !matched {
                return false;
            }
        }
    }
    true
}

/// The Suspense trace rules serialized into every generated score (base rules
/// only now that the Extended preset — and its `progress >= 0.8 -> outro` rule
/// — is retired).
fn default_rules() -> Vec<AdaptiveRule> {
    TRACE_SPECS
        .iter()
        .map(|spec| {
            let mut numeric = serde_json::Map::new();
            if let Some(min) = spec.heat_min {
                numeric.insert("heat".into(), serde_json::json!({"min": min}));
            }
            if let Some(min) = spec.focus_min {
                numeric.insert("focus".into(), serde_json::json!({"min": min}));
            }
            if let Some(min) = spec.progress_min {
                numeric.insert("progress".into(), serde_json::json!({"min": min}));
            }
            AdaptiveRule {
                target: spec.section.into(),
                priority: spec.priority,
                when: AdaptiveCondition {
                    numeric: serde_json::Value::Object(numeric),
                    categorical: match spec.phase {
                        Some(phase) => serde_json::json!({"tracePhase": phase}),
                        None => serde_json::json!({}),
                    },
                },
                hold: Some(spec.hold),
            }
        })
        .collect()
}

fn song_form() -> SongForm {
    SongForm {
        steps: [
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
        ]
        .into_iter()
        .map(|section| SongFormStep {
            section: section.to_string(),
            repeats: 1,
        })
        .collect(),
        loop_from: Some(1),
        origin: None,
    }
}

fn score_id(secret: &str, seed: &str, style: SuspenseStyle, traits: &NormalizedTraits) -> String {
    let identity = hash_text(&format!(
        "{GENERATOR_VERSION}\0{secret}\0string:{seed}\0{}\0{:.2}:{:.2}:{:.2}:{:.2}",
        style.as_str(),
        traits.tension,
        traits.heat,
        traits.mystery,
        traits.pulse
    ));
    format!(
        "suspense-generated-v{}-{:08x}",
        GENERATOR_VERSION.replace('.', "-"),
        identity
    )
}

/// Generate a Suspense score in the style's sound world.
pub fn generate_suspense(input: &SuspenseInput) -> Result<PortableScore, String> {
    let mut score = compose_suspense(input)?;
    apply_sound_world(&mut score, input.style);
    score.validate()?;
    Ok(score)
}

/// The Suspense composition in its reference instruments, before the style's
/// sound world is applied.
pub(crate) fn compose_suspense(input: &SuspenseInput) -> Result<PortableScore, String> {
    let traits = normalize(input);
    let harmony = create_harmony(subseed(&input.secret, &input.seed, "harmony"));
    let motif = create_motif(subseed(&input.secret, &input.seed, "motif"));
    let timbre = reference_timbre();
    let arrangement = create_arrangement(
        subseed(&input.secret, &input.seed, "arrangement"),
        input.style,
        &traits,
    );
    let bar_ticks = 4 * 960;
    let pulse = 480;
    let sections = PLANS
        .iter()
        .map(|plan| build_section(plan, &harmony, &motif, &timbre, &traits, bar_ticks, pulse))
        .collect();
    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: score_id(&input.secret, &input.seed, input.style, &traits),
        title: format!(
            "{} {} drone",
            input.style.display(),
            harmony.key.to_uppercase()
        ),
        bpm: arrangement.bpm,
        beats_per_bar: 4,
        ticks_per_beat: 960,
        crossfade_bars: 2.0,
        default_section: "intro".into(),
        sections,
        rules: default_rules(),
        form: Some(song_form()),
    };
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use super::{generate_suspense, select_trace_section, SuspenseInput, SuspenseStyle};
    use crate::score::{AdaptiveCondition, AdaptiveRule, MusicEvent, TraceState};
    use crate::transport::AdaptiveTransport;

    fn input(seed: &str, style: SuspenseStyle) -> SuspenseInput {
        SuspenseInput {
            secret: "arkhos-lab".into(),
            seed: seed.into(),
            style,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.7,
            pulse: 0.55,
        }
    }

    fn melody(events: &[MusicEvent]) -> Vec<&MusicEvent> {
        events.iter().filter(|event| event.is_melody()).collect()
    }

    #[test]
    fn generates_a_fourteen_section_song_form() {
        let score = generate_suspense(&input("session-7", SuspenseStyle::Terminal)).unwrap();
        assert_eq!(score.sections.len(), 14);
        let steps: Vec<_> = score
            .form
            .as_ref()
            .unwrap()
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert!(steps.contains(&"break"));
        assert!(steps.contains(&"bridge-b"));
        assert!(score.form.is_some());
        assert_eq!(score.default_section, "intro");
        for section in &score.sections {
            assert_eq!(section.length_ticks, 8 * 4 * 960);
            assert!(!section.events.is_empty(), "{}", section.id);
        }
        assert!(score.bpm >= 60.0 && score.bpm <= 88.0);
    }

    #[test]
    fn idle_is_a_drone_and_a_cell_not_a_tune() {
        let score = generate_suspense(&input("space", SuspenseStyle::Noir)).unwrap();
        let intro = score.section("intro").unwrap();
        let verse = score.section("verse").unwrap();
        let chorus = score.section("chorus").unwrap();
        assert!(melody(&intro.events).len() <= 4);
        assert!(melody(&intro.events).len() <= melody(&verse.events).len());
        assert!(
            intro.events.iter().any(|event| match event {
                MusicEvent::Note {
                    duration_ticks,
                    lane,
                    ..
                } => {
                    lane.contains("drone") && *duration_ticks >= 4 * 3840
                }
                _ => false,
            }),
            "intro needs a long drone"
        );
        let intro_has_arp = intro.events.iter().any(|event| match event {
            MusicEvent::Note { lane, .. } => lane.contains("arp"),
            _ => false,
        });
        let chorus_has_arp = chorus.events.iter().any(|event| match event {
            MusicEvent::Note { lane, .. } => lane.contains("arp"),
            _ => false,
        });
        assert!(!intro_has_arp);
        assert!(chorus_has_arp);
        assert!(chorus.events.len() > intro.events.len());
    }

    fn lane_named(events: &[MusicEvent], needle: &str) -> bool {
        events.iter().any(|event| match event {
            MusicEvent::Note { lane, .. } => lane.contains(needle),
            _ => false,
        })
    }

    #[test]
    fn break_drops_the_machine_and_second_bridge_has_no_arp() {
        let score = generate_suspense(&input("parts", SuspenseStyle::Terminal)).unwrap();
        let brk = score.section("break").unwrap();
        let bridge = score.section("bridge").unwrap();
        let other = score.section("bridge-b").unwrap();
        assert!(!lane_named(&brk.events, "pulse"));
        assert!(!lane_named(&brk.events, "arp"));
        assert!(melody(&brk.events).len() <= 2);
        assert!(lane_named(&bridge.events, "arp"));
        assert!(!lane_named(&other.events, "arp"));
        assert!(lane_named(&other.events, "pulse"));
    }

    #[test]
    fn approach_punches_in_on_a_downbeat_kick() {
        let score = generate_suspense(&input("punch", SuspenseStyle::Noir)).unwrap();
        let approach = score.section("pre-chorus").unwrap();
        let downbeat = approach.events.iter().any(|event| match event {
            MusicEvent::Percussion {
                start_tick, voice, ..
            } => voice == "kick" && *start_tick == 0,
            _ => false,
        });
        assert!(downbeat, "Approach should kick on bar 1 beat 1");
    }

    #[test]
    fn scan_has_one_entrance_kick_without_filling_its_quiet_opening() {
        let score = generate_suspense(&input("default-play", SuspenseStyle::Terminal)).unwrap();
        let scan = score.section("verse").unwrap();
        let kicks: Vec<_> = scan
            .events
            .iter()
            .filter_map(|event| match event {
                MusicEvent::Percussion {
                    voice,
                    start_tick,
                    velocity,
                    ..
                } if voice == "kick" => Some((*start_tick, *velocity)),
                _ => None,
            })
            .collect();
        let bar = score.bar_ticks();
        assert_eq!(kicks.first(), Some(&(0, 0.42)));
        assert_eq!(kicks.iter().filter(|(tick, _)| *tick < 4 * bar).count(), 1);
        let expected: Vec<_> = (4..8)
            .flat_map(|index| [(index * bar, 0.28), (index * bar + bar / 2, 0.22)])
            .collect();
        assert_eq!(&kicks[1..], expected.as_slice());
    }

    #[test]
    fn kit_never_uses_a_backbeat_snare() {
        let score = generate_suspense(&input("kit", SuspenseStyle::Cipher)).unwrap();
        for section in &score.sections {
            for event in &section.events {
                if let MusicEvent::Percussion { voice, .. } = event {
                    assert_ne!(voice, "snare", "{}", section.id);
                }
            }
        }
    }

    #[test]
    fn is_deterministic_and_style_sensitive() {
        let first = generate_suspense(&input("same", SuspenseStyle::Cipher)).unwrap();
        let second = generate_suspense(&input("same", SuspenseStyle::Cipher)).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(
            first.sections[0].events.len(),
            second.sections[0].events.len()
        );
        let noir = generate_suspense(&input("same", SuspenseStyle::Noir)).unwrap();
        assert_ne!(first.id, noir.id);
    }

    #[test]
    fn form_advances_without_game_state_and_alert_cues_once() {
        let score = generate_suspense(&input("form-drive", SuspenseStyle::Terminal)).unwrap();
        let bar = score.bar_ticks();
        let intro_len = score.section("intro").unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        assert_eq!(transport.current_section(), "intro");
        transport.advance(intro_len);
        transport.advance(intro_len + bar * 2);
        assert_eq!(transport.current_section(), "verse");

        transport.request_trace_state(
            &TraceState {
                phase: "alert".into(),
                heat: 0.8,
                focus: 0.2,
                progress: 0.1,
            },
            intro_len + bar * 3,
        );
        let after_cue = intro_len + bar * 5;
        transport.advance(after_cue);
        assert_eq!(transport.current_section(), "bridge");
        transport.request_trace_state(
            &TraceState {
                phase: "alert".into(),
                heat: 0.8,
                focus: 0.2,
                progress: 0.1,
            },
            after_cue,
        );
        assert_eq!(transport.current_section(), "bridge");
    }

    #[test]
    fn complete_holds_the_coda() {
        let score = generate_suspense(&input("ending", SuspenseStyle::Noir)).unwrap();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        transport.request_trace_state(
            &TraceState {
                phase: "complete".into(),
                heat: 0.1,
                focus: 0.1,
                progress: 1.0,
            },
            0,
        );
        transport.advance(8 * 4 * 960);
        transport.advance(16 * 4 * 960);
        assert_eq!(transport.current_section(), "coda");
    }

    fn trace_rule(
        target: &str,
        numeric: serde_json::Value,
        categorical: serde_json::Value,
    ) -> AdaptiveRule {
        AdaptiveRule {
            target: target.into(),
            priority: 80,
            when: AdaptiveCondition {
                numeric,
                categorical,
            },
            hold: Some(false),
        }
    }

    #[test]
    fn trace_selector_fails_closed_on_unknown_numeric_keys() {
        let rule = trace_rule(
            "bridge",
            serde_json::json!({ "temperature": { "min": 0.75 } }),
            serde_json::json!({}),
        );
        let state = TraceState {
            phase: "scan".into(),
            heat: 0.9,
            focus: 0.5,
            progress: 0.1,
        };
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state),
            None
        );
    }

    #[test]
    fn trace_selector_fails_closed_on_unknown_categorical_keys() {
        let rule = trace_rule(
            "bridge",
            serde_json::json!({}),
            serde_json::json!({ "mood": "alert" }),
        );
        let state = TraceState {
            phase: "alert".into(),
            heat: 0.8,
            focus: 0.2,
            progress: 0.1,
        };
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state),
            None
        );
    }

    #[test]
    fn trace_selector_fails_closed_on_non_finite_values() {
        let rule = trace_rule(
            "bridge",
            serde_json::json!({ "heat": { "min": 0.75 } }),
            serde_json::json!({}),
        );
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let state = TraceState {
                phase: "scan".into(),
                heat: bad,
                focus: 0.5,
                progress: 0.1,
            };
            assert_eq!(
                select_trace_section(std::slice::from_ref(&rule), &state),
                None,
                "heat={bad}"
            );
        }
    }

    #[test]
    fn trace_selector_applies_min_and_max_bounds() {
        let rule = trace_rule(
            "outro",
            serde_json::json!({ "progress": { "min": 0.5, "max": 0.9 } }),
            serde_json::json!({}),
        );
        let state = |progress: f64| TraceState {
            phase: "scan".into(),
            heat: 0.5,
            focus: 0.5,
            progress,
        };
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state(0.4)),
            None,
            "below min"
        );
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state(0.95)),
            None,
            "above max"
        );
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state(0.7)),
            Some(("outro".into(), false)),
            "inside range"
        );
    }

    #[test]
    fn trace_selector_accepts_an_array_phase_and_rejects_mismatches() {
        let rule = trace_rule(
            "outro",
            serde_json::json!({}),
            serde_json::json!({ "tracePhase": ["extract", "alert"] }),
        );
        let state = |phase: &str| TraceState {
            phase: phase.into(),
            heat: 0.1,
            focus: 0.1,
            progress: 0.1,
        };
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state("extract")),
            Some(("outro".into(), false))
        );
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state("alert")),
            Some(("outro".into(), false))
        );
        assert_eq!(
            select_trace_section(std::slice::from_ref(&rule), &state("complete")),
            None
        );
    }
}
