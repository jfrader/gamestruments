use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{AdaptiveCondition, AdaptiveRule, MusicEvent, PortableScore, PortableSection};

pub const GENERATOR_VERSION: &str = "1.10.1";
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

struct SectionPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    intensity: f64,
    development: i32,
    lift: i32,
    register_shift: i32,
    melody_gate_scale: f64,
    chord_gate_scale: f64,
    bass_gate_scale: f64,
    final_accent: &'static str,
}

const PLANS: [SectionPlan; 6] = [
    SectionPlan {
        id: "garage",
        label: "Garage",
        feeling: "polished / unhurried",
        color: "#dbc89a",
        intensity: 0.55,
        development: 0,
        lift: 0,
        register_shift: -7,
        melody_gate_scale: 1.65,
        chord_gate_scale: 1.18,
        bass_gate_scale: 1.22,
        final_accent: "hat",
    },
    SectionPlan {
        id: "grid",
        label: "Starting Grid",
        feeling: "focus / anticipation",
        color: "#f4b740",
        intensity: 0.7,
        development: 1,
        lift: 0,
        register_shift: -2,
        melody_gate_scale: 1.12,
        chord_gate_scale: 0.82,
        bass_gate_scale: 0.88,
        final_accent: "hat",
    },
    SectionPlan {
        id: "cruise",
        label: "Race Flow",
        feeling: "precision / forward motion",
        color: "#73d7c7",
        intensity: 0.82,
        development: 2,
        lift: 0,
        register_shift: 7,
        melody_gate_scale: 0.72,
        chord_gate_scale: 1.12,
        bass_gate_scale: 0.9,
        final_accent: "hat",
    },
    SectionPlan {
        id: "attack",
        label: "Position Fight",
        feeling: "pressure / resolve",
        color: "#f06c4f",
        intensity: 0.95,
        development: 3,
        lift: 1,
        register_shift: 7,
        melody_gate_scale: 0.52,
        chord_gate_scale: 1.05,
        bass_gate_scale: 0.7,
        final_accent: "hat",
    },
    SectionPlan {
        id: "final-lap",
        label: "Final Lap",
        feeling: "maximum commitment",
        color: "#ef3f50",
        intensity: 1.08,
        development: 4,
        lift: 0,
        register_shift: 12,
        melody_gate_scale: 0.55,
        chord_gate_scale: 1.08,
        bass_gate_scale: 0.72,
        final_accent: "tom",
    },
    SectionPlan {
        id: "victory",
        label: "Finish",
        feeling: "release / earned confidence",
        color: "#e7d46a",
        intensity: 0.78,
        development: 5,
        lift: 0,
        register_shift: 0,
        melody_gate_scale: 1.48,
        chord_gate_scale: 1.2,
        bass_gate_scale: 1.2,
        final_accent: "tom",
    },
];

const DRIVE_PHASES: [&str; 3] = ["grid", "attack", "final-lap"];

struct StyleKit {
    melody: &'static str,
    bright_melody: &'static str,
    harmony: &'static [&'static str],
    drive: &'static [&'static str],
    lift: &'static [&'static str],
    bass: &'static str,
}

fn style_kit(style: Style) -> StyleKit {
    match style {
        Style::Fusion => StyleKit {
            melody: "epiano",
            bright_melody: "glass",
            harmony: &["warm"],
            drive: &["organ"],
            lift: &["glass", "epiano"],
            bass: "bass",
        },
        Style::Neon => StyleKit {
            melody: "supersaw",
            bright_melody: "glass",
            harmony: &["pulse"],
            drive: &["pulse"],
            lift: &["glass"],
            bass: "bass",
        },
        Style::Funk => StyleKit {
            melody: "pluck",
            bright_melody: "epiano",
            harmony: &["warm", "pluck"],
            drive: &["pluck"],
            lift: &["pluck", "epiano"],
            bass: "bass",
        },
        Style::Chip => StyleKit {
            melody: "chip",
            bright_melody: "chip",
            harmony: &["chip"],
            drive: &["chip"],
            lift: &["chip"],
            bass: "triangle",
        },
    }
}

#[derive(Clone)]
struct NormalizedTraits {
    energy: f64,
    complexity: f64,
    brightness: f64,
    syncopation: f64,
}

fn normalize_traits(
    energy: f64,
    complexity: f64,
    brightness: f64,
    syncopation: f64,
) -> NormalizedTraits {
    NormalizedTraits {
        energy: energy.clamp(0.0, 1.0),
        complexity: complexity.clamp(0.0, 1.0),
        brightness: brightness.clamp(0.0, 1.0),
        syncopation: syncopation.clamp(0.0, 1.0),
    }
}

const NOTE_NAMES: [&str; 12] = [
    "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
];
const KEY_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const PROGRESSIONS: [&[i32; 4]; 4] = [&[0, 5, 3, 4], &[0, 3, 5, 4], &[0, 4, 5, 3], &[0, 2, 5, 4]];
const MOTIF_CONTOURS: [&[i32; 8]; 4] = [
    &[0, 2, 4, 3, 1, 5, 4, 0],
    &[0, 3, 2, 5, 4, 2, 1, 0],
    &[0, 1, 4, 2, 5, 3, 1, 0],
    &[0, 4, 3, 1, 2, 5, 4, 0],
];
const MAJOR_INTERVALS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

fn mode_intervals(mode: &str) -> Vec<i32> {
    match mode {
        "natural-minor" => vec![0, 2, 3, 5, 7, 8, 10],
        "dorian" => vec![0, 2, 3, 5, 7, 9, 10],
        "mixolydian" => vec![0, 2, 4, 5, 7, 9, 10],
        "lydian" => vec![0, 2, 4, 6, 7, 9, 11],
        _ => vec![0, 2, 3, 5, 7, 9, 10],
    }
}

fn select_mode(rng: &mut DeterministicRandom, brightness: f64) -> String {
    if brightness < 0.34 {
        rng.pick(&["natural-minor", "dorian"]).to_string()
    } else if brightness > 0.66 {
        rng.pick(&["mixolydian", "lydian"]).to_string()
    } else {
        rng.pick(&["dorian", "mixolydian"]).to_string()
    }
}

#[derive(Clone)]
struct HarmonyDna {
    root_pitch_class: i32,
    scale_intervals: Vec<i32>,
    progression_degrees: Vec<i32>,
    chord_size: u8,
    key: String,
}

fn create_harmony_dna(seed: u32, traits: &NormalizedTraits) -> HarmonyDna {
    let mut rng = DeterministicRandom::new(seed);
    let rpc = *rng.pick(&KEY_PITCH_CLASSES);
    let mode = select_mode(&mut rng, traits.brightness);
    let intervals = mode_intervals(&mode);
    let prog = rng.pick(&PROGRESSIONS).to_vec();
    let csize = if traits.complexity >= 0.62 { 4 } else { 3 };
    HarmonyDna {
        root_pitch_class: rpc,
        scale_intervals: intervals,
        progression_degrees: prog,
        chord_size: csize,
        key: NOTE_NAMES[rpc as usize].to_string(),
    }
}

fn create_motif_dna(seed: u32) -> Vec<i32> {
    let mut rng = DeterministicRandom::new(seed);
    let mut contour: Vec<i32> = rng.pick(&MOTIF_CONTOURS).to_vec();
    let ai = 4 + rng.integer(3) as usize;
    if let Some(ac) = contour.get_mut(ai) {
        *ac += *rng.pick(&[-1i32, 1]);
    }
    contour[0] = 0;
    contour[7] = 0;
    contour
}

#[derive(Clone)]
struct RhythmDna {
    melody_onsets: Vec<u32>,
    kick_onsets: Vec<u32>,
    #[allow(dead_code)]
    snare_onsets: Vec<u32>,
}

fn clamp_i(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

fn create_rhythm_dna(seed: u32, traits: &NormalizedTraits) -> RhythmDna {
    let mut rng = DeterministicRandom::new(seed);
    let note_count = 3 + (traits.complexity * 4.0).round() as usize;
    let odd_target = clamp_i(
        (note_count as f64 * traits.syncopation).round() as i32,
        0,
        4,
    ) as usize;
    let even_target = clamp_i((note_count - odd_target) as i32, 0, 4) as usize;
    let remaining = note_count - odd_target - even_target;
    let odd_count = odd_target + remaining.min(4 - odd_target);
    let even_count = note_count - odd_count;
    let mut odd: Vec<u32> = rng.shuffle(&[1u32, 3, 5, 7]);
    odd.truncate(odd_count);
    let mut even: Vec<u32> = rng.shuffle(&[0u32, 2, 4, 6]);
    even.truncate(even_count);
    let mut melody_onsets: Vec<u32> = odd.into_iter().chain(even).collect();
    melody_onsets.sort_unstable();
    let k1 = rng.pick(&[3u32, 4, 6]);
    let kick_onsets = vec![0, *k1];
    let mut snare_choices: Vec<u32> = rng.shuffle(&[2u32, 6, 3, 7, 1, 5]);
    snare_choices.retain(|s| !kick_onsets.contains(s));
    let s0 = *snare_choices.first().unwrap_or(&2);
    let s1 = *snare_choices.get(1).unwrap_or(&6);
    let mut snare_onsets = vec![s0, s1];
    snare_onsets.sort_unstable();
    RhythmDna {
        melody_onsets,
        kick_onsets,
        snare_onsets,
    }
}

#[derive(Clone)]
struct TimbreDna {
    harmony_voice: String,
    drive_harmony_voice: String,
    melody_voice: String,
    lift_voice: String,
    bass_voice: String,
}

fn create_timbre_dna(seed: u32, traits: &NormalizedTraits, style: Style) -> TimbreDna {
    let mut rng = DeterministicRandom::new(seed);
    let kit = style_kit(style);
    let use_alt = match style {
        Style::Fusion => traits.brightness > 0.72,
        Style::Neon => traits.brightness < 0.35,
        Style::Funk => traits.energy < 0.35,
        Style::Chip => false,
    };
    TimbreDna {
        harmony_voice: rng.pick(kit.harmony).to_string(),
        drive_harmony_voice: rng.pick(kit.drive).to_string(),
        melody_voice: if use_alt {
            kit.bright_melody.to_string()
        } else {
            kit.melody.to_string()
        },
        lift_voice: rng.pick(kit.lift).to_string(),
        bass_voice: kit.bass.to_string(),
    }
}

fn arrangement_groove(style: Style, plan: &SectionPlan) -> &SectionPlan {
    if style != Style::Funk {
        return plan;
    }
    if plan.id == "grid" {
        PLANS.iter().find(|p| p.id == "cruise").unwrap_or(plan)
    } else if plan.id == "cruise" {
        PLANS.iter().find(|p| p.id == "grid").unwrap_or(plan)
    } else {
        plan
    }
}

#[derive(Clone)]
struct ArrangementDna {
    bpm: f64,
    bass_approach: String,
    chord_gate: f64,
    melody_gate: f64,
}

fn create_arrangement_dna(seed: u32, traits: &NormalizedTraits) -> ArrangementDna {
    let mut rng = DeterministicRandom::new(seed);
    let jitter = rng.integer(5) as f64 - 2.0;
    ArrangementDna {
        bpm: (112.0 + (traits.energy * 36.0).round() + jitter).clamp(112.0, 150.0),
        bass_approach: rng
            .pick(&["root-fifth", "root-octave", "fifth-root"])
            .to_string(),
        chord_gate: 0.68 + traits.complexity * 0.22,
        melody_gate: 0.42 + (1.0 - traits.syncopation) * 0.3,
    }
}

#[derive(Clone)]
struct OrnamentDna {
    bar_rotations: [u32; 4],
    passing_offsets: [i32; 4],
    turnaround_step: u32,
}

fn create_ornament_dna(seed: u32, traits: &NormalizedTraits) -> OrnamentDna {
    let mut rng = DeterministicRandom::new(seed);
    let strength = if traits.complexity >= 0.5 { 1 } else { 0 };
    OrnamentDna {
        bar_rotations: [
            0,
            1 + rng.integer(3),
            3 + rng.integer(3),
            1 + rng.integer(5),
        ],
        passing_offsets: [
            0,
            rng.pick(&[-1i32, 1]) * strength,
            rng.pick(&[-1i32, 1]) * strength,
            0,
        ],
        turnaround_step: rng.integer(7),
    }
}

fn json_num(v: f64) -> String {
    let mut s = format!("{:.2}", v);
    while s.ends_with('0') && s.contains('.') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s.is_empty() {
        "0".to_string()
    } else {
        s
    }
}

fn compute_score_id(
    secret: &str,
    seed: &str,
    style_str: &str,
    palette_key: &str,
    palette_empty: bool,
    traits: &NormalizedTraits,
    gen_ver: &str,
) -> String {
    let empty_pal = secret.is_empty() && palette_empty;
    let ver = if empty_pal { "1.9.0" } else { gen_ver };
    let vstr = ver.replace('.', "-");
    let identity = if empty_pal {
        let canon = format!("string:{}", seed);
        let tjson = format!(
            r#"{{"energy":{},"complexity":{},"brightness":{},"syncopation":{}}}"#,
            json_num(traits.energy),
            json_num(traits.complexity),
            json_num(traits.brightness),
            json_num(traits.syncopation)
        );
        hash_text(&format!("{}:{}:{}", canon, style_str, tjson))
    } else {
        hash_text(&format!(
            "{}\0{}\0{}\0{}\0{}",
            secret, seed, style_str, palette_key, gen_ver
        ))
    };
    format!("pocket-circuit-generated-v{}-{:08x}", vstr, identity)
}

fn subseed(secret: &str, seed: &str, domain: &str, palette: &str) -> u32 {
    let canonical = format!("string:{}", seed);
    let mut s = format!("{}\0{}\0{}", DNA_SEED_VERSION, canonical, domain);
    if !secret.is_empty() || !palette.is_empty() {
        s.push('\0');
        s.push_str(secret);
        s.push('\0');
        s.push_str(palette);
    }
    hash_text(&s)
}

pub fn generate_pocket_circuit(input: &GenerateInput) -> Result<PortableScore, String> {
    let traits = normalize_traits(
        input.energy,
        input.complexity,
        input.brightness,
        input.syncopation,
    );
    let style = input.style;
    let style_str = style.as_str();
    let palette_key = input.palette.fingerprint();
    let palette_empty = input.palette.melody.is_empty()
        && input.palette.harmony.is_empty()
        && input.palette.drive.is_empty()
        && input.palette.bass.is_empty();
    let pal_for_seed = if palette_empty {
        "".to_string()
    } else {
        palette_key.clone()
    };
    let domain_h = subseed(&input.secret, &input.seed, "harmony", &pal_for_seed);
    let domain_m = subseed(&input.secret, &input.seed, "motif", &pal_for_seed);
    let domain_r = subseed(&input.secret, &input.seed, "rhythm", &pal_for_seed);
    let domain_t = subseed(&input.secret, &input.seed, "timbre", &pal_for_seed);
    let domain_a = subseed(&input.secret, &input.seed, "arrangement", &pal_for_seed);
    let domain_o = subseed(&input.secret, &input.seed, "ornaments", &pal_for_seed);

    let harmony = create_harmony_dna(domain_h, &traits);
    let motif = create_motif_dna(domain_m);
    let rhythm = create_rhythm_dna(domain_r, &traits);
    let mut timbre = create_timbre_dna(domain_t, &traits, style);
    // palette overrides (after generation defaults from kit)
    if !input.palette.harmony.is_empty() {
        timbre.harmony_voice = input.palette.harmony.clone();
    }
    if !input.palette.drive.is_empty() {
        timbre.drive_harmony_voice = input.palette.drive.clone();
    }
    if !input.palette.melody.is_empty() {
        timbre.melody_voice = input.palette.melody.clone();
    }
    if !input.palette.bass.is_empty() {
        timbre.bass_voice = input.palette.bass.clone();
    }
    let mut arrangement = create_arrangement_dna(domain_a, &traits);
    if style == Style::Fusion {
        arrangement.bpm = (arrangement.bpm + 10.0).clamp(112.0, 150.0);
    }
    let ornaments = create_ornament_dna(domain_o, &traits);

    let ticks_per_beat = 960u32;
    let beats_per_bar = 4u32;
    let bar_ticks = ticks_per_beat * beats_per_bar;
    let pulse = 480u32;

    let sections: Vec<PortableSection> = PLANS
        .iter()
        .map(|plan| {
            build_section_faithful(
                plan,
                style,
                &harmony,
                &motif,
                &rhythm,
                &timbre,
                &arrangement,
                &ornaments,
                &traits,
                bar_ticks,
                pulse,
                &input.palette,
            )
        })
        .collect();

    let id = compute_score_id(
        &input.secret,
        &input.seed,
        style_str,
        &palette_key,
        palette_empty,
        &traits,
        GENERATOR_VERSION,
    );
    let key_upper = harmony.key.to_uppercase();
    let title = format!("{} {} Run", style_display(style), key_upper);

    let score = PortableScore {
        schema_version: 1,
        id,
        title,
        bpm: arrangement.bpm,
        beats_per_bar,
        ticks_per_beat,
        crossfade_bars: 2.0,
        default_section: "garage".into(),
        sections,
        rules: default_rules(),
    };
    score.validate()?;
    Ok(score)
}

fn style_display(style: Style) -> &'static str {
    match style {
        Style::Fusion => "Fusion",
        Style::Neon => "Neon",
        Style::Funk => "Pocket Funk",
        Style::Chip => "Micro Motor",
    }
}

#[allow(clippy::too_many_arguments)]
fn build_section_faithful(
    plan: &SectionPlan,
    style: Style,
    harmony: &HarmonyDna,
    motif: &[i32],
    rhythm: &RhythmDna,
    timbre: &TimbreDna,
    arrangement: &ArrangementDna,
    ornaments: &OrnamentDna,
    traits: &NormalizedTraits,
    bar_ticks: u32,
    pulse: u32,
    _palette: &InstrumentPalette,
) -> PortableSection {
    let groove = arrangement_groove(style, plan);
    let resolved = resolve_section_harmony(harmony, groove.id);
    let drive = DRIVE_PHASES.contains(&groove.id);
    let harm_voice = if drive {
        &timbre.drive_harmony_voice
    } else {
        &timbre.harmony_voice
    };
    let mel_voice = &timbre.melody_voice;
    let bass_voice = &timbre.bass_voice;
    let lift_voice = &timbre.lift_voice;

    let pid = plan.id;
    let lane_h = format!("{}-harmony", pid);
    let lane_m = format!("{}-melody", pid);
    let lane_b = format!("{}-bass", pid);
    let lane_k = format!("{}-kit", pid);

    // collect per-lane raw (start, end, token, pitch_or_0) then assign ids after sort per lane
    let mut harmony_raw: Vec<(u32, u32, String, u8)> = Vec::new();
    let mut melody_raw: Vec<(u32, u32, String, u8)> = Vec::new();
    let mut bass_raw: Vec<(u32, u32, String, u8)> = Vec::new();
    let mut kit_raw: Vec<(u32, u32, String, u8)> = Vec::new(); // token is voice, pitch unused
    let mut lift_raw: Vec<(u32, u32, String, u8)> = Vec::new();
    let mut sparkle_raw: Vec<(u32, u32, String, u8)> = Vec::new();

    for bar in 0..4 {
        // harmony chords
        let use_seventh = resolved.chord_size == 4
            || groove.development >= 3
            || (style == Style::Fusion && groove.development >= 1);
        let offs: &[i32] = if use_seventh {
            &[0, 2, 4, 6]
        } else {
            &[0, 2, 4]
        };
        let chord_pitches: Vec<u8> = offs
            .iter()
            .map(|off| {
                scale_pitch(
                    48 + harmony.root_pitch_class,
                    resolved.progression_degrees[bar] + off,
                    &resolved.scale_intervals,
                ) as u8
            })
            .collect();
        let mask = transform_style_mask(chord_mask(plan.id, bar), style, plan, "harmony", true);
        let (slots, slot_dur) = match &mask {
            Mask::Whole => (vec![bar as u32 * bar_ticks], bar_ticks),
            Mask::Steps(sts) => (
                sts.iter()
                    .map(|&st| bar as u32 * bar_ticks + st * pulse)
                    .collect(),
                pulse,
            ),
        };
        for s in slots {
            for &pit in &chord_pitches {
                let tok = midi_to_note(pit as i32);
                harmony_raw.push((s, s + slot_dur, tok, pit));
            }
        }

        // bass
        let root_m = scale_pitch(
            36 + harmony.root_pitch_class,
            resolved.progression_degrees[bar],
            &resolved.scale_intervals,
        );
        let fifth_m = scale_pitch(
            36 + harmony.root_pitch_class,
            resolved.progression_degrees[bar] + 4,
            &resolved.scale_intervals,
        );
        let alt_m = match arrangement.bass_approach.as_str() {
            "root-octave" => root_m + 12,
            "fifth-root" => scale_pitch(
                36 + harmony.root_pitch_class,
                resolved.progression_degrees[bar] - 3,
                &resolved.scale_intervals,
            ),
            _ => fifth_m,
        };
        let bmask = transform_style_mask(bass_mask(plan.id, bar), style, plan, "bass", true);
        if let Mask::Whole = &bmask {
            let pit = root_m as u8;
            bass_raw.push((
                bar as u32 * bar_ticks,
                (bar as u32 + 1) * bar_ticks,
                midi_to_note(root_m),
                pit,
            ));
        } else if let Mask::Steps(sts) = &bmask {
            let seq = [root_m, alt_m, root_m + 12, fifth_m];
            for (ni, &st) in sts.iter().enumerate() {
                let pit = seq[ni % seq.len()] as u8;
                let s = bar as u32 * bar_ticks + st * pulse;
                bass_raw.push((s, s + pulse, midi_to_note(pit as i32), pit));
            }
        }

        // melody
        let onsets =
            phase_melody_onsets(&rhythm.melody_onsets, style, plan, bar, traits.complexity);
        let degrees = melody_degrees_for_bar(motif, ornaments, plan, bar, &onsets);
        let bright_oct = if traits.brightness >= 0.67 { 12 } else { 0 };
        let root_mel = 60 + harmony.root_pitch_class + bright_oct;
        let reg = plan.register_shift
            + if style == Style::Fusion && pid == "victory" {
                7
            } else {
                0
            };
        for (ni, &st) in onsets.iter().enumerate() {
            let deg = degrees[ni];
            let pit = scale_pitch(root_mel, deg + reg, &resolved.scale_intervals) as u8;
            let s = bar as u32 * bar_ticks + st * pulse;
            melody_raw.push((s, s + pulse, midi_to_note(pit as i32), pit));
        }

        // percussion
        let pons = percussion_onsets(rhythm, plan, bar, style);
        let mut step_tok: Vec<Option<&'static str>> = vec![None; 8];
        for &st in &pons.kick {
            step_tok[st as usize] = Some("kick");
        }
        for &st in &pons.snare {
            step_tok[st as usize] = Some("snare");
        }
        for &st in &pons.hat {
            step_tok[st as usize] = Some("hat");
        }
        if bar == 3 && plan.final_accent == "tom" {
            step_tok[7] = Some("tom");
        }
        for (st, opt) in step_tok.iter().enumerate() {
            if let Some(t) = opt {
                let s = bar as u32 * bar_ticks + (st as u32) * pulse;
                kit_raw.push((s, s + pulse, t.to_string(), 0));
            }
        }
    }

    // lift (only final-lap, uses plan for some, but onsets from groove? lift pattern uses resolved prog
    if pid == "final-lap" {
        let root_l = 72 + harmony.root_pitch_class;
        let extra = match style {
            Style::Fusion => 4,
            Style::Neon => 7,
            Style::Funk => 2,
            Style::Chip => 0,
        };
        for (bi, &deg) in resolved.progression_degrees.iter().enumerate() {
            let pit = scale_pitch(root_l, deg + extra, &resolved.scale_intervals) as u8;
            let s = bi as u32 * bar_ticks;
            lift_raw.push((s, s + bar_ticks, midi_to_note(pit as i32), pit));
        }
    }

    // sparkle
    if style == Style::Fusion && pid == "victory" {
        let root_sp = 72 + harmony.root_pitch_class;
        let offs = [0, 2, 4, 7];
        let onsets = [0u32, 2, 4, 6];
        for bar in 0..4 {
            for (oi, &st) in onsets.iter().enumerate() {
                let pit = scale_pitch(root_sp, offs[oi], &resolved.scale_intervals) as u8;
                let s = bar as u32 * bar_ticks + st * pulse;
                sparkle_raw.push((s, s + pulse, midi_to_note(pit as i32), pit));
            }
        }
    }

    // now per lane: sort by start, end, token; build events with per-lane index ids
    #[allow(clippy::ptr_arg)]
    fn sort_raw(raw: &mut Vec<(u32, u32, String, u8)>) {
        raw.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    }
    sort_raw(&mut harmony_raw);
    sort_raw(&mut melody_raw);
    sort_raw(&mut bass_raw);
    sort_raw(&mut kit_raw);
    sort_raw(&mut lift_raw);
    sort_raw(&mut sparkle_raw);

    let mut events: Vec<MusicEvent> = Vec::new();

    // harmony lane
    let hgate = phase_gate(arrangement.chord_gate, plan.chord_gate_scale, style);
    let hvel = velocity(if style == Style::Chip { 0.1 } else { 0.18 }, traits, plan);
    for (idx, (s, e, _tok, pit)) in harmony_raw.iter().enumerate() {
        let dur = 1u32.max((((*e as f64 - *s as f64) * hgate).round()) as u32);
        events.push(MusicEvent::Note {
            id: format!("{}:{}:{}", pid, lane_h, idx),
            section: pid.to_string(),
            lane: lane_h.clone(),
            start_tick: *s,
            duration_ticks: dur,
            velocity: hvel.clamp(0.1, 0.96),
            pitch: *pit,
            voice: harm_voice.clone(),
            role: None,
        });
    }

    // melody
    let mgate = phase_gate(arrangement.melody_gate, plan.melody_gate_scale, style);
    let mvel = velocity(0.24, traits, plan);
    for (idx, (s, e, _tok, pit)) in melody_raw.iter().enumerate() {
        let dur = 1u32.max((((*e as f64 - *s as f64) * mgate).round()) as u32);
        events.push(MusicEvent::Note {
            id: format!("{}:{}:{}", pid, lane_m, idx),
            section: pid.to_string(),
            lane: lane_m.clone(),
            start_tick: *s,
            duration_ticks: dur,
            velocity: mvel.clamp(0.1, 0.96),
            pitch: *pit,
            voice: mel_voice.clone(),
            role: Some("melody".to_string()),
        });
    }

    // bass
    let bbase = 0.58 + traits.energy * 0.24;
    let bgate = phase_gate(bbase, plan.bass_gate_scale, style);
    let bvel = velocity(0.3, traits, plan);
    for (idx, (s, e, _tok, pit)) in bass_raw.iter().enumerate() {
        let dur = 1u32.max((((*e as f64 - *s as f64) * bgate).round()) as u32);
        events.push(MusicEvent::Note {
            id: format!("{}:{}:{}", pid, lane_b, idx),
            section: pid.to_string(),
            lane: lane_b.clone(),
            start_tick: *s,
            duration_ticks: dur,
            velocity: bvel.clamp(0.1, 0.96),
            pitch: *pit,
            voice: bass_voice.clone(),
            role: None,
        });
    }

    // kit perc , gate fixed 0.45
    let kvel = velocity(0.2, traits, plan);
    for (idx, (s, e, tok, _)) in kit_raw.iter().enumerate() {
        let dur = 1u32.max((((*e as f64 - *s as f64) * 0.45).round()) as u32);
        events.push(MusicEvent::Percussion {
            id: format!("{}:{}:{}", pid, lane_k, idx),
            section: pid.to_string(),
            lane: lane_k.clone(),
            start_tick: *s,
            duration_ticks: dur,
            velocity: kvel.clamp(0.1, 0.96),
            voice: tok.clone(),
        });
    }

    // lift
    if !lift_raw.is_empty() {
        let lane_l = format!("{}-lift", pid);
        let lgate = lift_gate(style);
        let lvel = velocity(0.28, traits, plan); // note original plan
        for (idx, (s, e, _tok, pit)) in lift_raw.iter().enumerate() {
            let dur = 1u32.max((((*e as f64 - *s as f64) * lgate).round()) as u32);
            events.push(MusicEvent::Note {
                id: format!("{}:{}:{}", pid, lane_l, idx),
                section: pid.to_string(),
                lane: lane_l.clone(),
                start_tick: *s,
                duration_ticks: dur,
                velocity: lvel.clamp(0.1, 0.96),
                pitch: *pit,
                voice: lift_voice.clone(),
                role: None,
            });
        }
    }

    // sparkle
    if !sparkle_raw.is_empty() {
        let lane_sp = format!("{}-sparkle", pid);
        let svel = velocity(0.2, traits, plan);
        for (idx, (s, e, _tok, pit)) in sparkle_raw.iter().enumerate() {
            let dur = 1u32.max((((*e as f64 - *s as f64) * 0.5).round()) as u32);
            events.push(MusicEvent::Note {
                id: format!("{}:{}:{}", pid, lane_sp, idx),
                section: pid.to_string(),
                lane: lane_sp.clone(),
                start_tick: *s,
                duration_ticks: dur,
                velocity: svel.clamp(0.1, 0.96),
                pitch: *pit,
                voice: "glass".to_string(),
                role: None,
            });
        }
    }

    // final sort across lanes: by start, then by id
    events.sort_by(|a, b| {
        let sa = a.start_tick();
        let sb = b.start_tick();
        if sa != sb {
            return sa.cmp(&sb);
        }
        a_id(a).cmp(&b_id(b))
    });

    PortableSection {
        id: pid.to_string(),
        label: plan.label.to_string(),
        feeling: plan.feeling.to_string(),
        color: plan.color.to_string(),
        length_ticks: bar_ticks * 4,
        events,
    }
}

fn a_id(ev: &MusicEvent) -> String {
    match ev {
        MusicEvent::Note { id, .. } => id.clone(),
        MusicEvent::Percussion { id, .. } => id.clone(),
    }
}
fn b_id(ev: &MusicEvent) -> String {
    a_id(ev)
}

#[derive(Clone)]
struct ResolvedHarmony {
    scale_intervals: Vec<i32>,
    progression_degrees: Vec<i32>,
    chord_size: u8,
}

fn resolve_section_harmony(h: &HarmonyDna, section_id: &str) -> ResolvedHarmony {
    let treatment = match section_id {
        "garage" | "grid" | "cruise" => "shared",
        "attack" => "pressure",
        "final-lap" => "dominant",
        "victory" => "parallel-major",
        _ => "shared",
    };
    match treatment {
        "shared" => ResolvedHarmony {
            scale_intervals: h.scale_intervals.clone(),
            progression_degrees: h.progression_degrees.clone(),
            chord_size: h.chord_size,
        },
        "pressure" => {
            let mut sc = h.scale_intervals.clone();
            raise_seventh(&mut sc);
            ResolvedHarmony {
                scale_intervals: sc,
                progression_degrees: h.progression_degrees.clone(),
                chord_size: h.chord_size,
            }
        }
        "dominant" => {
            let mut p = h.progression_degrees.clone();
            p[3] = 4;
            let mut sc = h.scale_intervals.clone();
            raise_seventh(&mut sc);
            ResolvedHarmony {
                scale_intervals: sc,
                progression_degrees: p,
                chord_size: h.chord_size,
            }
        }
        "parallel-major" => {
            let mut p = h.progression_degrees.clone();
            p[3] = 0;
            ResolvedHarmony {
                scale_intervals: MAJOR_INTERVALS.to_vec(),
                progression_degrees: p,
                chord_size: h.chord_size,
            }
        }
        _ => ResolvedHarmony {
            scale_intervals: h.scale_intervals.clone(),
            progression_degrees: h.progression_degrees.clone(),
            chord_size: h.chord_size,
        },
    }
}

fn raise_seventh(iv: &mut [i32]) {
    if let Some(sev) = iv.get_mut(6) {
        *sev = (*sev + 1).min(11);
    }
}

#[derive(Clone)]
enum Mask {
    Whole,
    Steps(Vec<u32>),
}

fn chord_mask(section_id: &str, bar: usize) -> Mask {
    match (section_id, bar) {
        ("garage", _) | ("cruise", _) | ("attack", _) | ("final-lap", _) | ("victory", _) => {
            Mask::Whole
        }
        ("grid", 0) => Mask::Steps(vec![0, 4]),
        ("grid", 1) => Mask::Steps(vec![0, 3, 5]),
        ("grid", 2) => Mask::Steps(vec![0, 2, 4, 6]),
        ("grid", 3) => Mask::Steps(vec![0, 1, 3, 5, 7]),
        _ => Mask::Whole,
    }
}

fn bass_mask(section_id: &str, bar: usize) -> Mask {
    match (section_id, bar) {
        ("garage", _) | ("victory", _) => Mask::Whole,
        ("grid", 0) => Mask::Steps(vec![0, 4]),
        ("grid", 1) => Mask::Steps(vec![0, 3, 6]),
        ("grid", 2) => Mask::Steps(vec![0, 2, 4, 6]),
        ("grid", 3) => Mask::Steps(vec![0, 1, 3, 5, 7]),
        ("cruise", _) | ("attack", _) | ("final-lap", _) => Mask::Steps(vec![0, 4]),
        _ => Mask::Whole,
    }
}

fn transform_style_mask(
    mask: Mask,
    style: Style,
    plan: &SectionPlan,
    _lane: &str,
    preserve_down: bool,
) -> Mask {
    if matches!(mask, Mask::Whole)
        || style == Style::Fusion
        || plan.id == "cruise"
        || plan.id == "attack"
        || plan.id == "final-lap"
    {
        return mask;
    }
    let steps = match mask {
        Mask::Whole => return Mask::Whole,
        Mask::Steps(s) => s,
    };
    let downbeat = if preserve_down && steps.contains(&0) {
        vec![0]
    } else {
        vec![]
    };
    let movable: Vec<u32> = steps
        .iter()
        .filter(|s| !downbeat.contains(s))
        .copied()
        .collect();
    if style == Style::Neon {
        let shift = if plan.id == "grid" { 1 } else { 2 };
        let mut combined = downbeat.clone();
        combined.extend(movable.iter().map(|s| s + shift));
        return unique_steps(combined);
    }
    if style == Style::Funk {
        let mut mapped: Vec<u32> = downbeat.clone();
        mapped.extend(movable.iter().map(|&s| if s % 2 == 0 { s + 1 } else { s }));
        return unique_steps(mapped);
    }
    // chip echo
    let echo = if plan.id == "attack" { 2 } else { 1 };
    let mut mapped: Vec<u32> = steps.clone();
    mapped.extend(movable.iter().map(|&s| s + echo));
    unique_steps(mapped)
}

fn unique_steps(mut steps: Vec<u32>) -> Mask {
    steps = steps.into_iter().map(|s| s.rem_euclid(8)).collect();
    steps.sort_unstable();
    steps.dedup();
    if steps.is_empty() {
        Mask::Whole
    } else {
        Mask::Steps(steps)
    }
}

fn fill_steps(initial: &[u32], target: usize, candidates: &[u32]) -> Vec<u32> {
    let mut res = unique_vec(initial);
    for &c in candidates {
        if res.len() >= target {
            break;
        }
        if !res.contains(&c) {
            res.push(c);
        }
    }
    unique_vec(&res)[..target.min(res.len())].to_vec()
}

fn distributed_steps(steps: &[u32], count: usize) -> Vec<u32> {
    if count >= steps.len() {
        return steps.to_vec();
    }
    (0..count)
        .map(|i| {
            let si = (i * steps.len()) / count;
            steps[si]
        })
        .collect()
}

fn unique_vec(v: &[u32]) -> Vec<u32> {
    let mut r: Vec<u32> = v.iter().map(|step| step % 8).collect();
    r.sort_unstable();
    r.dedup();
    r
}

fn break_consecutive_runs(mut steps: Vec<u32>) -> Vec<u32> {
    loop {
        let pos = steps
            .windows(3)
            .position(|w| w[1] == w[0] + 1 && w[2] == w[0] + 2);
        if let Some(p) = pos {
            steps.remove(p + 1);
        } else {
            break;
        }
    }
    steps
}

fn phase_melody_onsets(
    base: &[u32],
    style: Style,
    plan: &SectionPlan,
    bar: usize,
    comp: f64,
) -> Vec<u32> {
    let onsets: Vec<u32> = if plan.id == "garage" {
        distributed_steps(base, base.len().min(2))
    } else if plan.id == "grid" {
        fill_steps(
            &[],
            2 + (comp * 3.0).round() as usize + bar,
            &[0, 4, 6, 2, 5, 7, 1, 3],
        )
    } else if plan.id == "cruise" {
        vec![2, 5]
    } else if plan.id == "attack" {
        vec![2, 6]
    } else if plan.id == "final-lap" {
        if bar == 3 {
            vec![2, 5, 7]
        } else {
            vec![2, 5]
        }
    } else {
        distributed_steps(base, base.len().min(2))
    };
    if plan.id == "cruise" || plan.id == "attack" || plan.id == "final-lap" {
        return unique_vec(&onsets);
    }
    if style == Style::Neon {
        let shift = if plan.id == "grid" { 1 } else { 2 };
        let sh: Vec<u32> = onsets.iter().map(|s| s + shift).collect();
        let u = unique_vec(&sh);
        return if plan.id == "grid" {
            u
        } else {
            break_consecutive_runs(u)
        };
    }
    if style == Style::Funk {
        if plan.id == "final-lap" {
            return fill_steps(&onsets, onsets.len().min(7), &[1, 3, 5, 7, 0, 2, 4, 6]);
        }
        let tgt = onsets.len().min(4);
        let mapped: Vec<u32> = onsets
            .iter()
            .map(|&s| if s % 2 == 0 { s + 1 } else { s })
            .collect();
        return break_consecutive_runs(fill_steps(&mapped, tgt, &[1, 3, 5, 7]));
    }
    if style == Style::Chip && plan.id != "victory" && plan.id != "garage" {
        if plan.id == "grid" {
            let mut ext = onsets.clone();
            ext.extend(onsets.iter().map(|s| s + 1));
            return unique_vec(&ext);
        }
        let maxv = if plan.id == "garage" {
            3
        } else if plan.id == "cruise" {
            7
        } else {
            8
        };
        let tgt = if plan.id == "final-lap" {
            8
        } else {
            onsets.len().min(maxv) + 1
        };
        let echo = if plan.id == "attack" { 2 } else { 1 };
        let mut src = onsets.clone();
        src.extend(onsets.iter().map(|s| s + echo));
        return fill_steps(
            &src,
            tgt,
            if plan.id == "attack" {
                &[1, 3, 5, 7]
            } else {
                &[0, 2, 4, 6, 1, 3, 5, 7]
            },
        );
    }
    unique_vec(&onsets)
}

fn melody_degrees_for_bar(
    motif: &[i32],
    ornaments: &OrnamentDna,
    plan: &SectionPlan,
    bar: usize,
    onsets: &[u32],
) -> Vec<i32> {
    let rot = ornaments.bar_rotations[bar] as usize;
    let pass = ornaments.passing_offsets[bar];
    let mut degs = Vec::new();
    for (ni, &st) in onsets.iter().enumerate() {
        let mi = (ni
            + rot
            + if bar == 0 {
                0
            } else {
                plan.development as usize
            })
            % motif.len();
        let md = motif[mi];
        let is_turn = bar == 3 && st == ornaments.turnaround_step;
        let dev = if is_turn {
            0
        } else {
            md + if bar == 0 { 0 } else { plan.lift + pass }
        };
        if let Some(&prev) = degs.last() {
            if prev == dev {
                degs.push(dev + 2);
                continue;
            }
        }
        degs.push(dev);
    }
    degs
}

fn percussion_onsets(
    rhythm: &RhythmDna,
    plan: &SectionPlan,
    bar: usize,
    style: Style,
) -> PercOnsets {
    #[allow(unused_mut)]
    let (mut kick, mut snare, mut hat) = if plan.id == "garage" {
        (
            vec![rhythm.kick_onsets.first().copied().unwrap_or(0)],
            vec![],
            vec![4],
        )
    } else if plan.id == "grid" {
        let ek = rhythm.kick_onsets.get(1).copied().unwrap_or(6);
        let k = if bar >= 2 {
            vec![0, if ek == 6 { 4 } else { ek }]
        } else {
            vec![0]
        };
        let sn = if bar >= 1 { vec![6, 2] } else { vec![6] };
        let mut h = vec![2, 4];
        if bar >= 2 {
            h.push(1);
        }
        if bar == 3 {
            h.push(7);
        }
        (k, sn, h)
    } else if plan.id == "cruise" {
        (vec![0, 4], vec![2, 6], vec![1, 3, 5, 7])
    } else if plan.id == "attack" {
        (vec![0, 4, 5], vec![2, 6], vec![1, 3, 7])
    } else if plan.id == "final-lap" {
        (vec![0, 4, 5], vec![2, 6, 7], vec![1, 3])
    } else {
        let h = if bar == 3 { vec![] } else { vec![6] };
        (vec![0], vec![4], h)
    };
    if plan.id == "garage" || plan.id == "grid" || plan.id == "victory" {
        if style == Style::Neon {
            hat = hat.into_iter().map(|s| (s + 2) % 8).collect();
        } else if style == Style::Funk {
            kick = kick
                .into_iter()
                .enumerate()
                .map(|(i, s)| {
                    if i == 0 {
                        s
                    } else if s % 2 == 0 {
                        s + 1
                    } else {
                        s
                    }
                })
                .collect();
            let mut nh: Vec<u32> = hat.into_iter().filter(|s| s % 2 == 1).collect();
            if plan.id != "garage" {
                nh.push(7);
            }
            hat = nh;
        } else if style == Style::Chip {
            let ech: Vec<u32> = kick.iter().map(|s| (s + 1) % 8).collect();
            hat.extend(ech);
        }
    }
    PercOnsets {
        kick: unique_vec(&kick),
        snare: unique_vec(&snare),
        hat: unique_vec(&hat),
    }
}
struct PercOnsets {
    kick: Vec<u32>,
    snare: Vec<u32>,
    hat: Vec<u32>,
}

fn gate_scale(style: Style) -> f64 {
    match style {
        Style::Fusion => 1.0,
        Style::Neon => 0.88,
        Style::Funk => 0.72,
        Style::Chip => 0.58,
    }
}
fn lift_gate(style: Style) -> f64 {
    match style {
        Style::Fusion => 0.92,
        Style::Neon => 0.88,
        Style::Funk => 0.8,
        Style::Chip => 0.7,
    }
}

fn phase_gate(base: f64, phase_scale: f64, style: Style) -> f64 {
    (base * phase_scale * gate_scale(style)).clamp(0.16, 0.98)
}

fn velocity(base: f64, traits: &NormalizedTraits, plan: &SectionPlan) -> f64 {
    (base + traits.energy * 0.22 + plan.intensity * 0.24).clamp(0.1, 0.96)
}

fn scale_pitch(root: i32, degree: i32, intervals: &[i32]) -> i32 {
    let len = intervals.len() as i32;
    let idx = degree.rem_euclid(len);
    let oct = (degree as f64 / len as f64).floor() as i32;
    root + oct * 12 + intervals[idx as usize]
}

fn midi_to_note(midi: i32) -> String {
    let pc = midi.rem_euclid(12);
    let name = NOTE_NAMES[pc as usize];
    let oct = (midi / 12) - 1;
    format!("{}{}", name, oct)
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
            target: "attack".into(),
            priority: 80,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"positionPressure":{"min":0.68}}),
                categorical: serde_json::json!({}),
            },
        },
        AdaptiveRule {
            target: "attack".into(),
            priority: 70,
            when: AdaptiveCondition {
                numeric: serde_json::json!({"intensity":{"min":0.72}}),
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
    use std::collections::HashSet;

    use super::{
        generate_pocket_circuit, GenerateInput, InstrumentPalette, Style, GENERATOR_VERSION,
    };

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
            generate_pocket_circuit(&sample("pocket-secret", InstrumentPalette::default()))
                .expect("pocket score must validate");
        let other = generate_pocket_circuit(&sample("other-secret", InstrumentPalette::default()))
            .expect("other score must validate");
        assert_ne!(pocket.id, other.id);
        assert_ne!(pitches(&pocket), pitches(&other));
    }

    #[test]
    fn palette_changes_the_piece() {
        let default_palette =
            generate_pocket_circuit(&sample("pocket-secret", InstrumentPalette::default()))
                .expect("default palette score must validate");
        let custom = generate_pocket_circuit(&sample(
            "pocket-secret",
            InstrumentPalette {
                melody: "chip".into(),
                harmony: "organ".into(),
                drive: "pulse".into(),
                bass: "triangle".into(),
            },
        ))
        .expect("custom palette score must validate");
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
        })
        .expect("six-section score must validate");
        assert_eq!(score.sections.len(), 6);
        assert!(score.section("cruise").unwrap().events.len() > 8);
    }

    #[test]
    fn many_seed_generation_is_valid_unique_and_deterministic() {
        assert_eq!(GENERATOR_VERSION, "1.10.1");
        let styles = [Style::Fusion, Style::Neon, Style::Funk, Style::Chip];
        let mut ids = HashSet::new();
        for index in 0..256 {
            let style = styles[index % styles.len()];
            let input = GenerateInput {
                secret: "stress-secret".into(),
                seed: format!("stress-{index}"),
                style,
                palette: InstrumentPalette::default(),
                energy: f64::from((index % 17) as u32) / 16.0,
                complexity: f64::from((index % 11) as u32) / 10.0,
                brightness: f64::from((index % 13) as u32) / 12.0,
                syncopation: f64::from((index % 19) as u32) / 18.0,
            };
            let score = generate_pocket_circuit(&input).expect("stress score must validate");
            assert!(
                ids.insert(score.id.clone()),
                "duplicate score id {}",
                score.id
            );
            if index % 31 == 0 {
                let repeated =
                    generate_pocket_circuit(&input).expect("repeated stress score must validate");
                assert_eq!(
                    serde_json::to_vec(&score).expect("stress score serializes"),
                    serde_json::to_vec(&repeated).expect("repeated stress score serializes")
                );
            }
        }
        assert_eq!(ids.len(), 256);
    }

    #[test]
    fn reserved_take_reproduces_catalog() {
        let input = GenerateInput {
            secret: "".into(),
            seed: "level-004".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.58,
            complexity: 0.75,
            brightness: 0.55,
            syncopation: 0.9,
        };
        let generated = generate_pocket_circuit(&input).expect("reserved score must validate");
        let catalog_str =
            include_str!("../../../catalog/pocket-circuit/tiny-torque-level-004/score.json");
        let catalog: super::PortableScore =
            serde_json::from_str(catalog_str).expect("catalog parses");
        assert_eq!(generated.bpm, catalog.bpm);
        assert_eq!(generated.title, catalog.title);
        assert_eq!(generated.id, "pocket-circuit-generated-v1-9-0-7864ec71");
        assert_eq!(generated.sections.len(), catalog.sections.len());
        let mut mismatches = 0usize;
        let mut first_diff = None;
        for (gsec, csec) in generated.sections.iter().zip(catalog.sections.iter()) {
            assert_eq!(gsec.id, csec.id);
            assert_eq!(
                gsec.events.len(),
                csec.events.len(),
                "len mismatch {}",
                gsec.id
            );
            for (i, (ge, ce)) in gsec.events.iter().zip(csec.events.iter()).enumerate() {
                let mut bad = false;
                if ge.start_tick() != ce.start_tick() {
                    bad = true;
                }
                if ge.duration_ticks() != ce.duration_ticks() {
                    bad = true;
                }
                let vdiff = (ge.velocity() - ce.velocity()).abs();
                if vdiff > 1e-6 {
                    bad = true;
                }
                if ge.voice() != ce.voice() {
                    bad = true;
                }
                match (ge, ce) {
                    (
                        super::MusicEvent::Note {
                            pitch: gp,
                            role: gr,
                            ..
                        },
                        super::MusicEvent::Note {
                            pitch: cp,
                            role: cr,
                            ..
                        },
                    ) => {
                        if gp != cp || gr != cr {
                            bad = true;
                        }
                    }
                    (
                        super::MusicEvent::Percussion { .. },
                        super::MusicEvent::Percussion { .. },
                    ) => {}
                    _ => {
                        bad = true;
                    }
                }
                let (gid, gsecid, glane) = match ge {
                    super::MusicEvent::Note {
                        id, section, lane, ..
                    } => (id, section, lane),
                    super::MusicEvent::Percussion {
                        id, section, lane, ..
                    } => (id, section, lane),
                };
                let (cid, csecid, clane) = match ce {
                    super::MusicEvent::Note {
                        id, section, lane, ..
                    } => (id, section, lane),
                    super::MusicEvent::Percussion {
                        id, section, lane, ..
                    } => (id, section, lane),
                };
                if gid != cid || gsecid != csecid || glane != clane {
                    bad = true;
                }
                if bad {
                    mismatches += 1;
                    if first_diff.is_none() {
                        first_diff = Some(format!("{}[{}] g:{:?} c:{:?}", gsec.id, i, ge, ce));
                    }
                }
            }
        }
        assert_eq!(mismatches, 0, "event mismatches: {}", mismatches);
    }
}
