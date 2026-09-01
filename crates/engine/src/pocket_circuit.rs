use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{AdaptiveCondition, AdaptiveRule, MusicEvent, PortableScore, PortableSection};

pub const GENERATOR_VERSION: &str = "1.10.0";
pub const DNA_SEED_VERSION: &str = "1.1.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Fusion,
    Neon,
    Funk,
    Chip,
}

impl Style {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "fusion" => Ok(Self::Fusion),
            "neon" => Ok(Self::Neon),
            "funk" => Ok(Self::Funk),
            "chip" => Ok(Self::Chip),
            other => Err(format!("Unknown style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fusion => "fusion",
            Self::Neon => "neon",
            Self::Funk => "funk",
            Self::Chip => "chip",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstrumentPalette {
    pub melody: String,
    pub harmony: String,
    pub drive: String,
    pub bass: String,
}

impl InstrumentPalette {
    pub fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.melody, self.harmony, self.drive, self.bass
        )
    }
}

#[derive(Clone, Debug)]
pub struct GenerateInput {
    pub secret: String,
    pub seed: String,
    pub style: Style,
    pub palette: InstrumentPalette,
    pub energy: f64,
    pub complexity: f64,
    pub brightness: f64,
    pub syncopation: f64,
}

struct Kit {
    melody: String,
    harmony: String,
    drive: String,
    bass: String,
}

fn kit(style: Style, palette: &InstrumentPalette) -> Kit {
    let defaults = match style {
        Style::Fusion => ("epiano", "warm", "organ", "bass"),
        Style::Neon => ("supersaw", "pulse", "pulse", "bass"),
        Style::Funk => ("pluck", "warm", "pluck", "bass"),
        Style::Chip => ("chip", "chip", "chip", "triangle"),
    };
    Kit {
        melody: choose_voice(&palette.melody, defaults.0),
        harmony: choose_voice(&palette.harmony, defaults.1),
        drive: choose_voice(&palette.drive, defaults.2),
        bass: choose_voice(&palette.bass, defaults.3),
    }
}

fn choose_voice(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

struct Plan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    intensity: f64,
    register: i32,
}

const PLANS: [Plan; 6] = [
    Plan {
        id: "garage",
        label: "Garage",
        feeling: "polished / unhurried",
        color: "#dbc89a",
        intensity: 0.55,
        register: -7,
    },
    Plan {
        id: "grid",
        label: "Starting Grid",
        feeling: "focus / anticipation",
        color: "#f4b740",
        intensity: 0.7,
        register: -2,
    },
    Plan {
        id: "cruise",
        label: "Race Flow",
        feeling: "precision / forward motion",
        color: "#73d7c7",
        intensity: 0.82,
        register: 7,
    },
    Plan {
        id: "attack",
        label: "Position Fight",
        feeling: "pressure / resolve",
        color: "#f06c4f",
        intensity: 0.95,
        register: 7,
    },
    Plan {
        id: "final-lap",
        label: "Final Lap",
        feeling: "maximum commitment",
        color: "#ef3f50",
        intensity: 1.08,
        register: 12,
    },
    Plan {
        id: "victory",
        label: "Finish",
        feeling: "release / earned confidence",
        color: "#e7d46a",
        intensity: 0.78,
        register: 0,
    },
];

pub fn generate_pocket_circuit(input: &GenerateInput) -> PortableScore {
    let energy = input.energy.clamp(0.0, 1.0);
    let complexity = input.complexity.clamp(0.0, 1.0);
    let brightness = input.brightness.clamp(0.0, 1.0);
    let syncopation = input.syncopation.clamp(0.0, 1.0);
    let kit = kit(input.style, &input.palette);
    let palette_key = input.palette.fingerprint();
    let bpm = (112.0
        + (energy * 36.0).round()
        + if input.style == Style::Fusion {
            10.0
        } else {
            0.0
        })
    .clamp(112.0, 160.0);
    let identity = hash_text(&format!(
        "{}\0{}\0{}\0{}\0{}",
        input.secret,
        input.seed,
        input.style.as_str(),
        palette_key,
        GENERATOR_VERSION
    ));
    let mut harmony_rng =
        DeterministicRandom::new(subseed(&input.secret, &input.seed, "harmony", &palette_key));
    let roots = [0, 2, 3, 5, 7, 9, 10];
    let root = roots[harmony_rng.integer(7) as usize];
    let scale: [i32; 7] = if brightness > 0.66 {
        [0, 2, 4, 6, 7, 9, 11]
    } else if brightness < 0.34 {
        [0, 2, 3, 5, 7, 8, 10]
    } else {
        [0, 2, 3, 5, 7, 9, 10]
    };
    let mut motif_rng =
        DeterministicRandom::new(subseed(&input.secret, &input.seed, "motif", &palette_key));
    let motif = [
        0,
        motif_rng.integer(6) as i32,
        motif_rng.integer(6) as i32,
        motif_rng.integer(6) as i32,
        motif_rng.integer(6) as i32,
        motif_rng.integer(6) as i32,
        motif_rng.integer(6) as i32,
        0,
    ];
    let mut rhythm_rng =
        DeterministicRandom::new(subseed(&input.secret, &input.seed, "rhythm", &palette_key));
    let melody_count = 3 + (complexity * 4.0).round() as usize;
    let mut melody_steps = rhythm_rng.shuffle(&[0_u32, 1, 2, 3, 4, 5, 6, 7]);
    melody_steps.truncate(melody_count.max(2));
    melody_steps.sort_unstable();
    if syncopation > 0.5 {
        melody_steps = melody_steps
            .into_iter()
            .map(|step| if step % 2 == 0 { (step + 1) % 8 } else { step })
            .collect();
        melody_steps.sort_unstable();
        melody_steps.dedup();
    }
    let ticks_per_beat = 960_u32;
    let beats_per_bar = 4_u32;
    let bar_ticks = ticks_per_beat * beats_per_bar;
    let pulse = 480_u32;
    let sections = PLANS
        .iter()
        .map(|plan| {
            build_section(
                plan,
                input.style,
                &kit,
                root,
                &scale,
                &motif,
                &melody_steps,
                energy,
                brightness,
                bar_ticks,
                pulse,
            )
        })
        .collect();
    PortableScore {
        schema_version: 1,
        id: format!(
            "pocket-circuit-generated-v{}-{:08x}",
            GENERATOR_VERSION.replace('.', "-"),
            identity
        ),
        title: format!("{} run", input.style.as_str()),
        bpm,
        beats_per_bar,
        ticks_per_beat,
        crossfade_bars: 2.0,
        default_section: "garage".into(),
        sections,
        rules: default_rules(),
    }
}

fn subseed(secret: &str, seed: &str, domain: &str, palette: &str) -> u32 {
    hash_text(&format!(
        "{DNA_SEED_VERSION}\0{secret}\0string:{seed}\0{domain}\0{palette}"
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_section(
    plan: &Plan,
    style: Style,
    kit: &Kit,
    root: i32,
    scale: &[i32; 7],
    motif: &[i32; 8],
    melody_steps: &[u32],
    energy: f64,
    brightness: f64,
    bar_ticks: u32,
    pulse: u32,
) -> PortableSection {
    let groove_id = if style == Style::Funk {
        match plan.id {
            "grid" => "cruise",
            "cruise" => "grid",
            other => other,
        }
    } else {
        plan.id
    };
    let drive = matches!(groove_id, "grid" | "attack" | "final-lap");
    let harmony_voice = if drive {
        kit.drive.as_str()
    } else {
        kit.harmony.as_str()
    };
    let melody_onsets: Vec<u32> = match groove_id {
        "garage" | "victory" => melody_steps.iter().copied().take(2).collect(),
        "grid" => vec![0, 2, 4, 6],
        "cruise" | "attack" => vec![2, 5],
        "final-lap" => vec![2, 5],
        _ => vec![2, 5],
    };
    let mut events = Vec::new();
    let octave = if brightness >= 0.67 { 12 } else { 0 };
    for bar in 0..4 {
        let degree = [0, 3, 5, 4][bar];
        let chord_start = bar as u32 * bar_ticks;
        let chord_held = groove_id != "grid";
        if chord_held {
            for offset in [0, 2, 4] {
                events.push(note_event(
                    plan.id,
                    "harmony",
                    events.len(),
                    chord_start,
                    bar_ticks.saturating_sub(80),
                    0.22 + plan.intensity * 0.2,
                    midi(
                        root,
                        degree + offset + plan.register / 12,
                        scale,
                        48 + octave,
                    ),
                    harmony_voice,
                    None,
                ));
            }
        } else {
            for step in [0_u32, 4] {
                for offset in [0, 2, 4] {
                    events.push(note_event(
                        plan.id,
                        "harmony",
                        events.len(),
                        chord_start + step * pulse,
                        pulse / 2,
                        0.28 + energy * 0.2,
                        midi(root, degree + offset, scale, 48 + octave),
                        harmony_voice,
                        None,
                    ));
                }
            }
        }
        events.push(note_event(
            plan.id,
            "bass",
            events.len(),
            chord_start,
            pulse * 3,
            0.35 + energy * 0.25,
            midi(root, degree, scale, 36),
            kit.bass.as_str(),
            None,
        ));
        events.push(note_event(
            plan.id,
            "bass",
            events.len(),
            chord_start + pulse * 4,
            pulse * 3,
            0.32 + energy * 0.2,
            midi(root, degree + 4, scale, 36),
            kit.bass.as_str(),
            None,
        ));
        for (note_index, step) in melody_onsets.iter().enumerate() {
            let motif_degree = motif[(note_index + bar) % motif.len()];
            events.push(note_event(
                plan.id,
                "melody",
                events.len(),
                chord_start + *step * pulse,
                pulse,
                0.3 + plan.intensity * 0.25,
                midi(root, motif_degree + plan.register / 2, scale, 60 + octave),
                kit.melody.as_str(),
                Some("melody"),
            ));
        }
        events.push(perc_event(
            plan.id,
            events.len(),
            chord_start,
            pulse,
            0.4,
            "kick",
        ));
        events.push(perc_event(
            plan.id,
            events.len(),
            chord_start + pulse * 4,
            pulse,
            0.38,
            "kick",
        ));
        if groove_id != "garage" {
            events.push(perc_event(
                plan.id,
                events.len(),
                chord_start + pulse * 2,
                pulse / 2,
                0.34,
                "snare",
            ));
            events.push(perc_event(
                plan.id,
                events.len(),
                chord_start + pulse * 6,
                pulse / 2,
                0.34,
                "snare",
            ));
        }
        for hat_step in [1_u32, 3, 5, 7] {
            if groove_id == "garage" && hat_step != 4 {
                continue;
            }
            events.push(perc_event(
                plan.id,
                events.len(),
                chord_start + hat_step * pulse,
                pulse / 3,
                0.2,
                "hat",
            ));
        }
    }
    PortableSection {
        id: plan.id.into(),
        label: plan.label.into(),
        feeling: plan.feeling.into(),
        color: plan.color.into(),
        length_ticks: bar_ticks * 4,
        events,
    }
}

fn midi(root: i32, degree: i32, scale: &[i32; 7], base: i32) -> u8 {
    let index = degree.rem_euclid(7);
    let octaves = degree.div_euclid(7);
    (base + root + scale[index as usize] + octaves * 12).clamp(0, 127) as u8
}

#[allow(clippy::too_many_arguments)]
fn note_event(
    section: &str,
    lane: &str,
    index: usize,
    start: u32,
    duration: u32,
    velocity: f64,
    pitch: u8,
    voice: &str,
    role: Option<&str>,
) -> MusicEvent {
    MusicEvent::Note {
        id: format!("{section}:{lane}:{index}"),
        section: section.into(),
        lane: format!("{section}-{lane}"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.05, 0.96),
        pitch,
        voice: voice.into(),
        role: role.map(str::to_string),
    }
}

fn perc_event(
    section: &str,
    index: usize,
    start: u32,
    duration: u32,
    velocity: f64,
    voice: &str,
) -> MusicEvent {
    MusicEvent::Percussion {
        id: format!("{section}:kit:{index}"),
        section: section.into(),
        lane: format!("{section}-kit"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.05, 0.96),
        voice: voice.into(),
    }
}

fn default_rules() -> Vec<AdaptiveRule> {
    vec![
        AdaptiveRule {
            target: "victory".into(),
            priority: 100,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"racePhase":"finish","finishResult":"win"}),
            },
        },
        AdaptiveRule {
            target: "final-lap".into(),
            priority: 90,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"finalLap":{"min":1}}),
                categorical: serde_json::json!({}),
            },
        },
        AdaptiveRule {
            target: "cruise".into(),
            priority: 30,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"racePhase":"race"}),
            },
        },
        AdaptiveRule {
            target: "grid".into(),
            priority: 20,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"racePhase":"grid"}),
            },
        },
        AdaptiveRule {
            target: "garage".into(),
            priority: 10,
            when: AdaptiveCondition {
                numeric: serde_json::json!({}),
                categorical: serde_json::json!({"racePhase":"garage"}),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};

    fn sample(secret: &str, palette: InstrumentPalette) -> GenerateInput {
        GenerateInput {
            secret: secret.into(),
            seed: "level-004".into(),
            style: Style::Funk,
            palette,
            energy: 0.58,
            complexity: 0.75,
            brightness: 0.55,
            syncopation: 0.9,
        }
    }

    fn pitches(score: &super::PortableScore) -> Vec<u8> {
        score.sections[1]
            .events
            .iter()
            .filter_map(|event| event.pitch())
            .collect()
    }

    #[test]
    fn secret_changes_the_piece() {
        let pocket =
            generate_pocket_circuit(&sample("pocket-secret", InstrumentPalette::default()));
        let other = generate_pocket_circuit(&sample("other-secret", InstrumentPalette::default()));
        assert_ne!(pocket.id, other.id);
        assert_ne!(pitches(&pocket), pitches(&other));
    }

    #[test]
    fn palette_changes_the_piece() {
        let default_palette =
            generate_pocket_circuit(&sample("pocket-secret", InstrumentPalette::default()));
        let custom = generate_pocket_circuit(&sample(
            "pocket-secret",
            InstrumentPalette {
                melody: "chip".into(),
                harmony: "organ".into(),
                drive: "pulse".into(),
                bass: "triangle".into(),
            },
        ));
        assert_ne!(default_palette.id, custom.id);
        assert_ne!(pitches(&default_palette), pitches(&custom));
        let custom_voices: Vec<_> = custom.sections[2]
            .events
            .iter()
            .filter(|event| event.is_melody())
            .map(|event| event.voice().to_string())
            .collect();
        assert!(custom_voices.iter().all(|voice| voice == "chip"));
    }

    #[test]
    fn generates_six_sections() {
        let score = generate_pocket_circuit(&GenerateInput {
            secret: "pocket-secret".into(),
            seed: "race-12".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.7,
            complexity: 0.6,
            brightness: 0.5,
            syncopation: 0.7,
        });
        assert_eq!(score.sections.len(), 6);
        assert!(score.section("cruise").unwrap().events.len() > 8);
    }
}
