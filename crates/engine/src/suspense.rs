use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    AdaptiveCondition, AdaptiveRule, MusicEvent, PortableScore, PortableSection, SongForm,
    SongFormStep, SCORE_SCHEMA_VERSION,
};

pub const GENERATOR_VERSION: &str = "2.0.0";
pub const DNA_SEED_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspenseStyle {
    Terminal,
    Cipher,
    Noir,
}

impl SuspenseStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "terminal" => Ok(Self::Terminal),
            "cipher" => Ok(Self::Cipher),
            "noir" => Ok(Self::Noir),
            other => Err(format!("Unknown suspense style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Cipher => "cipher",
            Self::Noir => "noir",
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
    Solo,
    Outro,
    Coda,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CueWeight {
    Idle,
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

const PLANS: [SectionPlan; 12] = [
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

fn create_timbre(seed: u32, style: SuspenseStyle) -> TimbreDna {
    let mut random = DeterministicRandom::new(seed);
    let _ = random.next();
    match style {
        SuspenseStyle::Terminal => TimbreDna {
            drone_voice: "warm".into(),
            cell_voice: "glass".into(),
            pulse_voice: "pulse".into(),
            arp_voice: "pulse".into(),
        },
        SuspenseStyle::Cipher => TimbreDna {
            drone_voice: "warm".into(),
            cell_voice: "pluck".into(),
            pulse_voice: "bass".into(),
            arp_voice: "glass".into(),
        },
        SuspenseStyle::Noir => TimbreDna {
            drone_voice: "organ".into(),
            cell_voice: "epiano".into(),
            pulse_voice: "bass".into(),
            arp_voice: "warm".into(),
        },
    }
}

fn create_arrangement(seed: u32, style: SuspenseStyle, traits: &NormalizedTraits) -> ArrangementDna {
    let mut random = DeterministicRandom::new(seed);
    let jitter = f64::from(random.integer(3)) - 1.0;
    let base = match style {
        SuspenseStyle::Noir => 64.0,
        SuspenseStyle::Terminal => 72.0,
        SuspenseStyle::Cipher => 78.0,
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
        SectionRole::Verse | SectionRole::PostChorus | SectionRole::Interlude | SectionRole::Solo => {
            CueWeight::Work
        }
        SectionRole::PreChorus | SectionRole::Chorus => CueWeight::Drive,
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
        CueWeight::Work => &[(0, 4), (2, 0), (4, 4), (6, 0)],
        CueWeight::Drive | CueWeight::Alarm => &[
            (0, 4),
            (2, 0),
            (3, 4),
            (5, 0),
            (6, 4),
        ],
    }
}

fn pulse_from_bar(weight: CueWeight, plan: &SectionPlan) -> Option<usize> {
    match plan.role {
        SectionRole::Solo | SectionRole::Intro | SectionRole::Coda => None,
        SectionRole::Verse if plan.id == "verse-b" => None,
        SectionRole::Interlude => Some(0),
        _ => match weight {
            CueWeight::Idle => None,
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
        CueWeight::Idle => None,
        CueWeight::Work => Some(4),
        CueWeight::Drive => Some(2),
        CueWeight::Alarm => Some(0),
    }
}

fn kick_from_bar(weight: CueWeight) -> Option<usize> {
    match weight {
        CueWeight::Idle => Some(4),
        CueWeight::Work => Some(4),
        CueWeight::Drive => Some(2),
        CueWeight::Alarm => Some(0),
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
        let pitch = midi(cell_root + interval);
        let hold = if matches!(weight, CueWeight::Idle) || traits.mystery >= 0.6 {
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
                0.28,
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

fn default_rules() -> Vec<AdaptiveRule> {
    vec![
        AdaptiveRule {
            target: "coda".into(),
            priority: 100,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"tracePhase":"complete"}),
            },
            hold: Some(true),
        },
        AdaptiveRule {
            target: "coda".into(),
            priority: 95,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"progress":{"min":0.95}}),
                categorical: serde_json::json!({}),
            },
            hold: Some(true),
        },
        AdaptiveRule {
            target: "outro".into(),
            priority: 90,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"tracePhase":"extract"}),
            },
            hold: Some(true),
        },
        AdaptiveRule {
            target: "bridge".into(),
            priority: 80,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"heat":{"min":0.75}}),
                categorical: serde_json::json!({}),
            },
            hold: Some(false),
        },
        AdaptiveRule {
            target: "bridge".into(),
            priority: 70,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"tracePhase":"alert"}),
            },
            hold: Some(false),
        },
        AdaptiveRule {
            target: "chorus".into(),
            priority: 60,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"focus":{"min":0.7}}),
                categorical: serde_json::json!({"tracePhase":"exploit"}),
            },
            hold: Some(false),
        },
    ]
}

fn song_form() -> SongForm {
    SongForm {
        steps: [
            "intro",
            "verse",
            "pre-chorus",
            "chorus",
            "verse-b",
            "post-chorus",
            "interlude",
            "bridge",
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

pub fn generate_suspense(input: &SuspenseInput) -> Result<PortableScore, String> {
    let traits = normalize(input);
    let harmony = create_harmony(subseed(&input.secret, &input.seed, "harmony"));
    let motif = create_motif(subseed(&input.secret, &input.seed, "motif"));
    let _rhythm = subseed(&input.secret, &input.seed, "rhythm");
    let timbre = create_timbre(subseed(&input.secret, &input.seed, "timbre"), input.style);
    let arrangement = create_arrangement(
        subseed(&input.secret, &input.seed, "arrangement"),
        input.style,
        &traits,
    );
    let _ornaments = subseed(&input.secret, &input.seed, "ornaments");
    let bar_ticks = 4 * 960;
    let pulse = 480;
    let sections = PLANS
        .iter()
        .map(|plan| {
            build_section(
                plan,
                &harmony,
                &motif,
                &timbre,
                &traits,
                bar_ticks,
                pulse,
            )
        })
        .collect();
    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: score_id(&input.secret, &input.seed, input.style, &traits),
        title: format!(
            "{} {} drone",
            match input.style {
                SuspenseStyle::Terminal => "Terminal",
                SuspenseStyle::Cipher => "Cipher",
                SuspenseStyle::Noir => "Noir",
            },
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
    use super::{generate_suspense, SuspenseInput, SuspenseStyle};
    use crate::score::{MusicEvent, TraceState};
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
    fn generates_a_twelve_section_song_form() {
        let score = generate_suspense(&input("session-7", SuspenseStyle::Terminal)).unwrap();
        assert_eq!(score.sections.len(), 12);
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
                MusicEvent::Note { duration_ticks, lane, .. } => {
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
        assert_eq!(first.sections[0].events.len(), second.sections[0].events.len());
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
}
