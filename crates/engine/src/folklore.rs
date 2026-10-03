//! Seeded Chacarera and Carnavalito scores.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    FormOrigin, MusicEvent, PortableScore, PortableSection, SongForm, SongFormStep,
    SCORE_SCHEMA_VERSION,
};
use crate::theory::{mode_intervals, scale_pitch, NOTE_NAMES};

pub const GENERATOR_VERSION: &str = "0.3.0-prototype";
const DNA_SEED_VERSION: &str = "0.2.0-prototype";
pub const CARNAVALITO_VERSION: &str = "0.1.0-carnavalito";

const EIGHTH: u32 = 480;
const BEAT: u32 = 960;
/// Chacarera bar: 3/4 with the interlocking 6/8 subdivision.
const BAR: u32 = 3 * BEAT;
/// Carnavalito bar: 2/4, two beats of binary (two-eighth) feet.
const CARNAVALITO_BAR: u32 = 2 * BEAT;
const LOWEST_GUITAR_PITCH: i32 = 40;

#[derive(Clone, Debug)]
pub struct FolkloreInput {
    pub secret: String,
    pub seed: String,
    pub energy: f64,
    pub complexity: f64,
    pub brightness: f64,
    pub syncopation: f64,
}

/// The folklore style the generator composes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolkloreStyle {
    Chacarera,
    Carnavalito,
}

impl FolkloreStyle {
    /// Accepts `""` and `"chacarera"` as the Chacarera default, plus
    /// `"carnavalito"`. Anything else is rejected.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "chacarera" => Ok(Self::Chacarera),
            "carnavalito" => Ok(Self::Carnavalito),
            other => Err(format!("Unknown folklore style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chacarera => "chacarera",
            Self::Carnavalito => "carnavalito",
        }
    }
}

#[derive(Clone, Copy)]
struct Traits {
    energy: f64,
    complexity: f64,
    brightness: f64,
    syncopation: f64,
}

impl From<&FolkloreInput> for Traits {
    fn from(input: &FolkloreInput) -> Self {
        fn normal(value: f64) -> f64 {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.5
            }
        }
        Self {
            energy: normal(input.energy),
            complexity: normal(input.complexity),
            brightness: normal(input.brightness),
            syncopation: normal(input.syncopation),
        }
    }
}

/// Keep the original Chacarera seed namespace when changing score identity.
fn subseed(input: &FolkloreInput, style: FolkloreStyle, domain: &str) -> u32 {
    let version = match style {
        FolkloreStyle::Chacarera => DNA_SEED_VERSION,
        FolkloreStyle::Carnavalito => CARNAVALITO_VERSION,
    };
    hash_text(&format!(
        "folklore/{version}\0{}\0{}\0{domain}",
        input.secret, input.seed
    ))
}

const ROOTS: [i32; 5] = [7, 9, 4, 2, 0];

/// The root pitch class the Chacarera score sounds in (the default style).
pub fn folklore_root_pitch_class(input: &FolkloreInput) -> i32 {
    folklore_root_pitch_class_with_style(input, FolkloreStyle::Chacarera)
}

pub fn folklore_root_pitch_class_with_style(input: &FolkloreInput, style: FolkloreStyle) -> i32 {
    ROOTS[subseed(input, style, "root") as usize % ROOTS.len()]
}

/// Chacarera diatonic-minor harmony.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Chord {
    Tonic,
    Subdominant,
    Dominant,
    Relative,
    Sixth,
}

impl Chord {
    fn notes(self) -> &'static [i32] {
        match self {
            Self::Tonic => &[0, 3, 7],
            Self::Subdominant => &[5, 8, 12],
            Self::Dominant => &[7, 11, 14, 17],
            Self::Relative => &[3, 7, 10],
            Self::Sixth => &[8, 12, 15],
        }
    }

    fn bass(self) -> i32 {
        self.notes()[0]
    }
}

use Chord::{Dominant as V7, Relative as III, Sixth as VI, Subdominant as Iv, Tonic as I};

/// How a Chacarera section textures its material. `Standard` reproduces the
/// original six sections byte-for-byte; the other three are the new phases.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Texture {
    Standard,
    /// Punteo: a guitar question/answer in the lead.
    CallAnswer,
    /// Respiro: quiet, low-register guitar space that keeps the dance pulse.
    Quiet,
    /// Peña: fuller celebration with a break and a decisive return.
    Celebration,
}

struct SectionPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    harmony: &'static [Chord],
    lead: bool,
    texture: Texture,
    /// Bar (within `harmony`) that drops to a dry break.
    break_bar: Option<usize>,
}

const PLANS: &[SectionPlan] = &[
    SectionPlan {
        id: "introduccion",
        label: "Introducción",
        feeling: "invitation",
        color: "#6B4423",
        harmony: &[I, Iv, V7, I],
        lead: false,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "primera",
        label: "Primera",
        feeling: "verse",
        color: "#8B5A2B",
        harmony: &[I, Iv, V7, I, III, VI, V7, I],
        lead: true,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "interludio",
        label: "Interludio",
        feeling: "reflection",
        color: "#556B2F",
        harmony: &[III, VI, V7, I],
        lead: false,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "segunda",
        label: "Segunda",
        feeling: "verse",
        color: "#8B5A2B",
        harmony: &[I, Iv, V7, I, III, VI, V7, I],
        lead: true,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "estribillo",
        label: "Estribillo",
        feeling: "refrain",
        color: "#CD853F",
        harmony: &[III, VI, Iv, V7, III, VI, V7, I],
        lead: true,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "cierre",
        label: "Cierre",
        feeling: "resolve",
        color: "#5C4033",
        harmony: &[Iv, V7, V7, I],
        lead: true,
        texture: Texture::Standard,
        break_bar: None,
    },
    SectionPlan {
        id: "punteo",
        label: "Punteo",
        feeling: "question/answer",
        color: "#A0522D",
        harmony: &[I, Iv, V7, I],
        lead: true,
        texture: Texture::CallAnswer,
        break_bar: None,
    },
    SectionPlan {
        id: "respiro",
        label: "Respiro",
        feeling: "breath",
        color: "#4A5D3A",
        harmony: &[III, VI, I, I],
        lead: false,
        texture: Texture::Quiet,
        break_bar: None,
    },
    SectionPlan {
        id: "pena",
        label: "Peña",
        feeling: "celebration",
        color: "#B87333",
        harmony: &[I, Iv, V7, I, Iv, V7, V7, I],
        lead: true,
        texture: Texture::Celebration,
        break_bar: Some(5),
    },
];

struct Composer {
    tonic: i32,
    traits: Traits,
    hook: [usize; 4],
}

/// A section's running event buffer shared by both styles.
struct Part<'a> {
    id: &'a str,
    events: Vec<MusicEvent>,
    next_id: usize,
}

impl Part<'_> {
    fn note(&mut self, lane: &str, tick: u32, duration: u32, velocity: f64, pitch: i32) {
        self.note_voice(lane, tick, duration, velocity, pitch, "nylon-guitar");
    }

    fn note_voice(
        &mut self,
        lane: &str,
        tick: u32,
        duration: u32,
        velocity: f64,
        pitch: i32,
        voice: &str,
    ) {
        self.push(MusicEvent::Note {
            id: String::new(),
            section: self.id.into(),
            lane: lane.into(),
            start_tick: tick,
            duration_ticks: duration,
            velocity: velocity.clamp(0.0, 1.0),
            pitch: pitch as u8,
            voice: voice.into(),
            role: (lane == "melody").then(|| "melody".into()),
        });
    }

    fn drum(&mut self, tick: u32, voice: &str, velocity: f64) {
        self.push(MusicEvent::Percussion {
            id: String::new(),
            section: self.id.into(),
            lane: "bombo".into(),
            start_tick: tick,
            duration_ticks: 240,
            velocity: velocity.clamp(0.0, 1.0),
            voice: voice.into(),
        });
    }

    fn push(&mut self, mut event: MusicEvent) {
        let id = format!("{}-{}", self.id, self.next_id);
        match &mut event {
            MusicEvent::Note { id: event_id, .. } | MusicEvent::Percussion { id: event_id, .. } => {
                *event_id = id;
            }
        }
        self.next_id += 1;
        self.events.push(event);
    }
}

impl Composer {
    fn section(&self, plan: &SectionPlan, input: &FolkloreInput) -> PortableSection {
        let mut part = Part {
            id: plan.id,
            events: Vec::new(),
            next_id: 0,
        };
        let mut rng = DeterministicRandom::new(subseed(input, FolkloreStyle::Chacarera, plan.id));
        let count = plan.harmony.len();
        let texture = plan.texture;
        let gain_scale = match texture {
            Texture::Standard | Texture::CallAnswer => 1.0,
            Texture::Quiet => 0.62,
            Texture::Celebration => 1.16,
        };
        for (bar, &chord) in plan.harmony.iter().enumerate() {
            let start = bar as u32 * BAR;
            let closing = bar + 1 == count;
            let break_bar = texture == Texture::Celebration && plan.break_bar == Some(bar);
            let gain = (0.78 + 0.21 * self.traits.energy)
                * if closing && texture != Texture::Celebration {
                    0.86
                } else {
                    1.0
                }
                * gain_scale;
            if break_bar {
                // The break: one dry bombo accent and a single dry hit; the
                // guitar and bass drop out before the decisive return.
                part.drum(start, "bombo", 0.9 * gain);
            } else if !closing || plan.id != "cierre" {
                // Bombo skin in 3/4 against dry rim and guitar accents in 6/8.
                part.drum(start, "bombo-rim", 0.35 * gain);
                part.drum(start + BEAT, "bombo", 0.71 * gain);
                part.drum(start + 3 * EIGHTH, "bombo-rim", 0.48 * gain);
                part.drum(start + 2 * BEAT, "bombo", 0.82 * gain);
                if self.traits.complexity > 0.68 && bar % 4 == 2 {
                    part.drum(start + 5 * EIGHTH, "bombo-rim", 0.27 * gain);
                }
            } else {
                part.drum(start, "bombo", 0.73 * gain);
            }

            if !break_bar {
                // Three walking bass beats; sparse rasgueado at the two compound accents.
                for beat in 0..3 {
                    if closing && plan.id == "cierre" && beat != 0 {
                        break;
                    }
                    let bass = self.tonic - 12
                        + if beat == 1 {
                            chord.notes()[2] - 12
                        } else {
                            chord.bass()
                        };
                    part.note(
                        "bass",
                        start + beat * BEAT,
                        580,
                        (0.47 + 0.06 * rng.next()) * gain,
                        LOWEST_GUITAR_PITCH + (bass - LOWEST_GUITAR_PITCH).rem_euclid(12),
                    );
                }
            }
            let strums: &[u32] = if break_bar {
                &[]
            } else if closing && plan.id == "cierre" {
                &[0]
            } else if texture == Texture::Quiet {
                // A single low strum keeps the space without the full rasgueado.
                &[0]
            } else if texture == Texture::Celebration {
                &[0, 3 * EIGHTH, 5 * EIGHTH]
            } else {
                &[0, 3 * EIGHTH]
            };
            for (index, &offset) in strums.iter().enumerate() {
                for (finger, &interval) in chord.notes().iter().enumerate() {
                    let spread = (finger as u32) * (12 + (self.traits.syncopation * 16.0) as u32);
                    part.note(
                        "rasgueado",
                        start + offset + spread,
                        370,
                        (0.27 + 0.06 * self.traits.brightness + 0.03 * rng.next())
                            * gain
                            * if index == 0 { 0.86 } else { 1.0 },
                        self.tonic + interval + if texture == Texture::Quiet { -12 } else { 0 },
                    );
                }
            }

            if break_bar {
                part.note(
                    "melody",
                    start,
                    180,
                    0.5 * gain,
                    self.tonic + 12 + chord.notes()[2],
                );
            } else if texture == Texture::Quiet {
                // A held low-guitar note on alternating bars; the bombo and
                // walking bass keep the dance pulse underneath.
                if bar.is_multiple_of(2) {
                    part.note(
                        "melody",
                        start + 3 * EIGHTH,
                        900,
                        0.42 * gain,
                        self.tonic - 12 + chord.notes()[0],
                    );
                }
            } else if texture == Texture::CallAnswer {
                self.melody_call_answer(&mut part, &mut rng, bar, chord);
            } else if plan.lead || (bar >= 2 && plan.id != "interludio") {
                self.melody(&mut part, &mut rng, bar, chord, plan);
            } else if plan.id == "interludio" && bar % 2 == 0 {
                part.note(
                    "melody",
                    start + 3 * EIGHTH,
                    900,
                    0.56 * gain,
                    self.tonic + 12 + chord.notes()[1],
                );
            }
        }
        part.events.sort_by_key(MusicEvent::start_tick);
        PortableSection {
            id: plan.id.into(),
            label: plan.label.into(),
            feeling: plan.feeling.into(),
            color: plan.color.into(),
            length_ticks: count as u32 * BAR,
            events: part.events,
        }
    }

    fn melody(
        &self,
        part: &mut Part<'_>,
        rng: &mut DeterministicRandom,
        bar: usize,
        chord: Chord,
        plan: &SectionPlan,
    ) {
        let start = bar as u32 * BAR;
        let final_bar = bar + 1 == plan.harmony.len();
        let answer = bar % 4 >= 2;
        let shift = if plan.id == "estribillo" { 1 } else { 0 };
        let tones = chord.notes();
        let first = tones[(self.hook[bar % 4] + shift) % 3];
        let next = tones[(self.hook[(bar + 1) % 4] + shift) % 3];
        let strength = (0.72 + 0.15 * self.traits.energy) * (0.96 + 0.06 * rng.next());
        let octave = self.tonic + 12;
        // A recurring call, an answering contour, and a small silence before each resolution.
        part.note(
            "melody",
            start,
            if final_bar { 1250 } else { 710 },
            strength,
            octave + first,
        );
        if final_bar {
            part.note(
                "melody",
                start + 3 * EIGHTH,
                3 * EIGHTH,
                strength * 0.88,
                octave,
            );
            return;
        }
        if self.traits.complexity > 0.25 && bar % 4 != 3 {
            let intervals = mode_intervals("aeolian");
            let degree = ((self.hook[bar % 4] + bar) % 7) as i32;
            let passing = scale_pitch(octave, degree, &intervals);
            let offset = if self.traits.syncopation > 0.55 {
                EIGHTH
            } else {
                BEAT
            };
            part.note("melody", start + offset, 340, strength * 0.7, passing);
        }
        part.note(
            "melody",
            start + 3 * EIGHTH,
            if bar % 4 == 3 { 700 } else { 570 },
            strength * if answer { 0.94 } else { 0.84 },
            octave + next,
        );
        if chord == V7 && (bar + 1) % 4 == 3 {
            // Harmonic minor's raised seventh anticipates the tonic in the next bar.
            part.note(
                "melody",
                start + 5 * EIGHTH,
                380,
                strength * 0.78,
                octave + 11,
            );
        } else if self.traits.complexity > 0.7 && bar % 4 == 1 {
            part.note(
                "melody",
                start + 5 * EIGHTH,
                330,
                strength * 0.62,
                octave + tones[1],
            );
        }
    }

    /// Punteo: a rising guitar question on even bars, a falling answer on odd
    /// bars, with a breath of silence at the top of each answer.
    fn melody_call_answer(
        &self,
        part: &mut Part<'_>,
        rng: &mut DeterministicRandom,
        bar: usize,
        chord: Chord,
    ) {
        let start = bar as u32 * BAR;
        let tones = chord.notes();
        let strength = (0.72 + 0.15 * self.traits.energy) * (0.96 + 0.06 * rng.next());
        let octave = self.tonic + 12;
        if bar.is_multiple_of(2) {
            // The question climbs and hovers on the fifth.
            part.note("melody", start, 480, strength, octave + tones[0]);
            part.note(
                "melody",
                start + 3 * EIGHTH,
                480,
                strength * 0.85,
                octave + tones[1],
            );
            part.note(
                "melody",
                start + 2 * BEAT,
                600,
                strength * 0.9,
                octave + tones[2],
            );
        } else {
            // The answer falls back and resolves onto the root.
            part.note(
                "melody",
                start + EIGHTH,
                480,
                strength * 0.9,
                octave + tones[2],
            );
            part.note(
                "melody",
                start + 3 * EIGHTH,
                480,
                strength * 0.85,
                octave + tones[1],
            );
            part.note("melody", start + 2 * BEAT, 900, strength, octave + tones[0]);
        }
    }
}

/// The Andean minor pentatonic (la-based) the Carnavalito draws its original
/// cells from: root, minor third, fourth, fifth, minor seventh.
const PENTATONIC: [i32; 5] = [0, 3, 5, 7, 10];
/// An open tonic drone (root, fourth, fifth) for the charango pedal.
const TONIC_CHORD: [i32; 3] = [0, 5, 7];
/// A lift color (minor third, fifth, minor seventh) for answers and the refrain.
const LIFT_CHORD: [i32; 3] = [3, 7, 10];

struct CarnavalitoPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    bars: u32,
    lead: bool,
    /// Lower-register call/answer between quena and charango.
    answer: bool,
    /// 0..1 fullness of the charango and bombo texture.
    build: f64,
}

const CARNAVALITO_PLANS: &[CarnavalitoPlan] = &[
    CarnavalitoPlan {
        id: "preludio",
        label: "Preludio",
        feeling: "opening",
        color: "#7D6B4A",
        bars: 4,
        lead: true,
        answer: false,
        build: 0.5,
    },
    CarnavalitoPlan {
        id: "copla",
        label: "Copla",
        feeling: "verse",
        color: "#A8865A",
        bars: 8,
        lead: true,
        answer: false,
        build: 0.68,
    },
    CarnavalitoPlan {
        id: "respuesta",
        label: "Respuesta",
        feeling: "answer",
        color: "#6B7D4F",
        bars: 4,
        lead: true,
        answer: true,
        build: 0.58,
    },
    CarnavalitoPlan {
        id: "estribillo",
        label: "Estribillo",
        feeling: "refrain",
        color: "#C79A5B",
        bars: 8,
        lead: true,
        answer: false,
        build: 0.9,
    },
    CarnavalitoPlan {
        id: "cierre",
        label: "Cierre",
        feeling: "resolve",
        color: "#5C4A35",
        bars: 4,
        lead: true,
        answer: false,
        build: 0.8,
    },
];

struct CarnavalitoComposer {
    tonic: i32,
    traits: Traits,
    hook: [usize; 5],
}

impl CarnavalitoComposer {
    fn section(&self, plan: &CarnavalitoPlan, input: &FolkloreInput) -> PortableSection {
        let mut part = Part {
            id: plan.id,
            events: Vec::new(),
            next_id: 0,
        };
        let mut rng = DeterministicRandom::new(subseed(input, FolkloreStyle::Carnavalito, plan.id));
        let build = (plan.build + (self.traits.energy - 0.5) * 0.25).clamp(0.3, 1.0);
        let gain = 0.78 + 0.2 * self.traits.energy;
        for bar in 0..plan.bars {
            let start = bar * CARNAVALITO_BAR;
            let lift = plan.answer
                || (plan.id == "estribillo" && bar >= 4)
                || (plan.id == "copla" && bar % 4 == 2);
            let chord = if lift { &LIFT_CHORD } else { &TONIC_CHORD };

            // The low bombo base holds the binary feet; rim accents mark the
            // offbeats once the texture fills out.
            part.drum(start, "bombo", (0.62 + 0.2 * build) * gain);
            part.drum(start + BEAT, "bombo", (0.5 + 0.18 * build) * gain);
            if build + self.traits.syncopation * 0.18 > 0.68 {
                part.drum(start + EIGHTH, "bombo-rim", 0.3 * gain);
                part.drum(start + 3 * EIGHTH, "bombo-rim", 0.3 * gain);
            }

            // Charango-like plucks drive the binary forward motion, accenting
            // the offbeat eighths.
            for eighth in 0..4 {
                let degree = chord[if eighth % 2 == 0 { 0 } else { 1 }];
                let accent = if eighth % 2 == 1 { 1.12 } else { 0.92 };
                let octave = if self.traits.brightness > 0.65 && eighth == 3 {
                    24
                } else {
                    12
                };
                part.note_voice(
                    "charango",
                    start + eighth * EIGHTH,
                    340,
                    (0.34 + 0.12 * build + 0.05 * rng.next()) * accent * gain,
                    self.tonic + octave + degree,
                    "charango",
                );
            }

            if plan.lead {
                self.quena(&mut part, &mut rng, bar, plan);
            }
        }
        part.events.sort_by_key(MusicEvent::start_tick);
        PortableSection {
            id: plan.id.into(),
            label: plan.label.into(),
            feeling: plan.feeling.into(),
            color: plan.color.into(),
            length_ticks: plan.bars * CARNAVALITO_BAR,
            events: part.events,
        }
    }

    /// A short, singable pentatonic cell that returns and develops: a two-note
    /// lift on even bars, a fall on odd bars, and a held answer to vary the
    /// phrase. The `respuesta` section plays the same cells an octave lower.
    fn quena(
        &self,
        part: &mut Part<'_>,
        rng: &mut DeterministicRandom,
        bar: u32,
        plan: &CarnavalitoPlan,
    ) {
        let start = bar * CARNAVALITO_BAR;
        let closing = bar + 1 == plan.bars;
        let register = if plan.answer {
            self.tonic
        } else {
            self.tonic + 12
        };
        let strength = (0.6 + 0.16 * self.traits.energy) * (0.95 + 0.06 * rng.next());
        let d = self.hook[(bar % 5) as usize];
        let e = self.hook[((bar + 1) % 5) as usize];
        let (a, b) = if bar.is_multiple_of(2) {
            (d, e)
        } else {
            (e, d)
        };
        part.note_voice(
            "melody",
            start,
            430,
            strength,
            register + PENTATONIC[a],
            "quena",
        );
        let second_offset = if self.traits.syncopation > 0.65 && !closing {
            EIGHTH + 120
        } else {
            EIGHTH
        };
        part.note_voice(
            "melody",
            start + second_offset,
            430,
            strength * 0.88,
            register + PENTATONIC[b],
            "quena",
        );
        if closing {
            part.note_voice("melody", start + BEAT, 880, strength, register, "quena");
        } else if bar % 4 == 3 || self.traits.complexity > 0.6 {
            part.note_voice(
                "melody",
                start + BEAT,
                620,
                strength * 0.8,
                register + PENTATONIC[(a + b) % 5],
                "quena",
            );
            part.note_voice(
                "melody",
                start + 3 * EIGHTH,
                470,
                strength * 0.85,
                register + PENTATONIC[b],
                "quena",
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arrangement {
    Seeded,
    AllPhases,
}

fn parse_arrangement(value: &str) -> Result<Arrangement, String> {
    match value {
        "" | "seeded" => Ok(Arrangement::Seeded),
        "all-phases" => Ok(Arrangement::AllPhases),
        _ => Err(format!("unsupported folklore arrangement {value}")),
    }
}

fn arrangement_name(arrangement: Arrangement) -> &'static str {
    match arrangement {
        Arrangement::Seeded => "seeded",
        Arrangement::AllPhases => "all-phases",
    }
}

fn compose_chacarera(input: &FolkloreInput, traits: Traits) -> Vec<PortableSection> {
    let root = folklore_root_pitch_class(input);
    let mut motif_rng = DeterministicRandom::new(subseed(input, FolkloreStyle::Chacarera, "hook"));
    let composer = Composer {
        tonic: 48 + root,
        traits,
        hook: std::array::from_fn(|_| motif_rng.integer(3) as usize),
    };
    PLANS
        .iter()
        .map(|plan| composer.section(plan, input))
        .collect()
}

fn compose_carnavalito(input: &FolkloreInput, traits: Traits) -> Vec<PortableSection> {
    let root = folklore_root_pitch_class_with_style(input, FolkloreStyle::Carnavalito);
    let tonic = 48 + root;
    let mut motif_rng =
        DeterministicRandom::new(subseed(input, FolkloreStyle::Carnavalito, "hook"));
    let hook: [usize; 5] =
        std::array::from_fn(|_| motif_rng.integer(PENTATONIC.len() as u32) as usize);
    let composer = CarnavalitoComposer {
        tonic,
        traits,
        hook,
    };
    CARNAVALITO_PLANS
        .iter()
        .map(|plan| composer.section(plan, input))
        .collect()
}

const CHACARERA_SEEDED: &[&str] = &[
    "introduccion",
    "primera",
    "interludio",
    "punteo",
    "respiro",
    "segunda",
    "interludio",
    "estribillo",
    "pena",
    "primera",
    "estribillo",
    "cierre",
];
const CHACARERA_ALL: &[&str] = &[
    "introduccion",
    "primera",
    "interludio",
    "segunda",
    "estribillo",
    "cierre",
    "punteo",
    "respiro",
    "pena",
];
const CARNAVALITO_SEEDED: &[&str] = &[
    "preludio",
    "copla",
    "respuesta",
    "copla",
    "estribillo",
    "copla",
    "cierre",
];
const CARNAVALITO_ALL: &[&str] = &["preludio", "copla", "respuesta", "estribillo", "cierre"];

fn assemble_score(
    input: &FolkloreInput,
    style: FolkloreStyle,
    arrangement: Arrangement,
    traits: Traits,
    sections: Vec<PortableSection>,
) -> Result<PortableScore, String> {
    let root = folklore_root_pitch_class_with_style(input, style);
    let (order, version, title, bpm, beats_per_bar, default_section) = match (style, arrangement) {
        (FolkloreStyle::Chacarera, arrangement) => (
            if arrangement == Arrangement::Seeded {
                CHACARERA_SEEDED
            } else {
                CHACARERA_ALL
            },
            GENERATOR_VERSION,
            format!(
                "Folklore {} minor",
                NOTE_NAMES[root as usize].to_uppercase()
            ),
            140.0 + 14.0 * traits.energy,
            3,
            "introduccion",
        ),
        (FolkloreStyle::Carnavalito, arrangement) => (
            if arrangement == Arrangement::Seeded {
                CARNAVALITO_SEEDED
            } else {
                CARNAVALITO_ALL
            },
            CARNAVALITO_VERSION,
            format!(
                "Folklore {} Carnavalito",
                NOTE_NAMES[root as usize].to_uppercase()
            ),
            132.0 + 12.0 * traits.energy,
            2,
            "preludio",
        ),
    };
    let form = SongForm {
        steps: order
            .iter()
            .map(|section| SongFormStep {
                section: (*section).into(),
                repeats: 1,
            })
            .collect(),
        loop_from: Some(if arrangement == Arrangement::AllPhases {
            0
        } else {
            1
        }),
        origin: Some(FormOrigin::TransitionStart),
    };
    let id_hash = hash_text(&format!(
        "{version}\0{}\0{}\0{}\0{:016x}:{:016x}:{:016x}:{:016x}",
        input.secret,
        input.seed,
        arrangement_name(arrangement),
        traits.energy.to_bits(),
        traits.complexity.to_bits(),
        traits.brightness.to_bits(),
        traits.syncopation.to_bits()
    ));
    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: format!("folklore-{id_hash:08x}"),
        title,
        bpm,
        beats_per_bar,
        ticks_per_beat: BEAT,
        crossfade_bars: 1.0,
        default_section: default_section.into(),
        sections,
        rules: vec![],
        form: Some(form),
    };
    score.validate()?;
    Ok(score)
}

/// The default Chacarera entry point, preserved for existing Rust callers.
pub fn generate_folklore(
    input: &FolkloreInput,
    arrangement: &str,
) -> Result<PortableScore, String> {
    generate_folklore_with_style(input, FolkloreStyle::Chacarera, arrangement)
}

/// The style-aware entry point: `Chacarera` composes the extended chacarera,
/// `Carnavalito` composes the binary pentatonic style.
pub fn generate_folklore_with_style(
    input: &FolkloreInput,
    style: FolkloreStyle,
    arrangement: &str,
) -> Result<PortableScore, String> {
    let arrangement = parse_arrangement(arrangement)?;
    let traits = Traits::from(input);
    let sections = match style {
        FolkloreStyle::Chacarera => compose_chacarera(input, traits),
        FolkloreStyle::Carnavalito => compose_carnavalito(input, traits),
    };
    assemble_score(input, style, arrangement, traits, sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn input(seed: &str) -> FolkloreInput {
        FolkloreInput {
            secret: "guitar".into(),
            seed: seed.into(),
            energy: 0.58,
            complexity: 0.47,
            brightness: 0.61,
            syncopation: 0.64,
        }
    }

    fn serialized<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string(value).unwrap()
    }

    #[test]
    fn deterministic_and_varied() {
        let a = input("call");
        let baseline = generate_folklore(&a, "seeded").unwrap();
        assert_eq!(
            serialized(&baseline),
            serialized(&generate_folklore(&a, "").unwrap())
        );
        let other = generate_folklore(&input("answer"), "seeded").unwrap();
        assert_ne!(baseline.id, other.id);
        assert_ne!(
            serialized(&baseline.sections[1]),
            serialized(&other.sections[1])
        );
        let mut hotter = a.clone();
        hotter.energy = 0.9;
        assert_ne!(
            baseline.id,
            generate_folklore(&hotter, "seeded").unwrap().id
        );
        assert_ne!(baseline.id, generate_folklore(&a, "all-phases").unwrap().id);
        assert!(generate_folklore(&a, "other").is_err());
    }

    #[test]
    fn form_and_bar_lengths() {
        let a = input("form");
        let score = generate_folklore(&a, "seeded").unwrap();
        assert_eq!(score.validate(), Ok(()));
        assert_eq!(
            score
                .sections
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            PLANS.iter().map(|p| p.id).collect::<Vec<_>>()
        );
        assert_eq!(
            score
                .sections
                .iter()
                .map(|s| s.length_ticks / BAR)
                .collect::<Vec<_>>(),
            vec![4, 8, 4, 8, 8, 4, 4, 4, 8]
        );
        assert!(score.sections.iter().all(|s| s.length_ticks % BAR == 0
            && s.events
                .iter()
                .all(|e| e.start_tick() + e.duration_ticks() <= s.length_ticks)));
        let form = score.form.as_ref().unwrap();
        assert_eq!(
            form.steps
                .iter()
                .map(|step| step.section.as_str())
                .collect::<Vec<_>>(),
            CHACARERA_SEEDED
        );
        assert_eq!(form.loop_from, Some(1));
        for step in &form.steps {
            assert!(score.section(&step.section).is_some());
        }
        let all = generate_folklore(&a, "all-phases").unwrap();
        assert_eq!(all.form.as_ref().unwrap().loop_from, Some(0));
        assert_eq!(all.form.unwrap().steps.len(), 9);
        assert_eq!(
            score.title,
            format!(
                "Folklore {} minor",
                NOTE_NAMES[folklore_root_pitch_class(&a) as usize].to_uppercase()
            )
        );
    }

    #[test]
    fn cross_rhythm_harmony_and_final_cadence() {
        let a = input("groove");
        let score = generate_folklore(&a, "seeded").unwrap();
        let verse = score.section("primera").unwrap();
        for bar in 0..8 {
            let at = bar * BAR;
            for (offset, voice) in [
                (0, "bombo-rim"),
                (BEAT, "bombo"),
                (3 * EIGHTH, "bombo-rim"),
                (2 * BEAT, "bombo"),
            ] {
                assert!(verse
                    .events
                    .iter()
                    .any(|e| e.start_tick() == at + offset && e.voice() == voice));
            }
            assert!(verse.events.iter().any(|e| matches!(e,
                MusicEvent::Note { lane, start_tick, .. } if lane == "rasgueado" && *start_tick == at + 3 * EIGHTH)));
        }
        let tonic = 48 + folklore_root_pitch_class(&a);
        let bass_at = |bar| {
            verse
                .events
                .iter()
                .find_map(|e| match e {
                    MusicEvent::Note {
                        lane,
                        start_tick,
                        pitch,
                        ..
                    } if lane == "bass" && *start_tick == bar as u32 * BAR => {
                        Some(i32::from(*pitch))
                    }
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(
            [0, 5, 7, 0, 3, 8, 7, 0].map(|interval| LOWEST_GUITAR_PITCH
                + (tonic - 12 + interval - LOWEST_GUITAR_PITCH).rem_euclid(12)),
            std::array::from_fn::<_, 8, _>(bass_at)
        );
        assert!(verse.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, pitch, .. }
            if lane == "rasgueado" && i32::from(*pitch) == tonic + 17)));
        assert!(verse.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, start_tick, pitch, .. }
            if lane == "melody" && *start_tick == 6 * BAR + 5 * EIGHTH && i32::from(*pitch) == tonic + 23)));
        let close = score.section("cierre").unwrap();
        assert!(close.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, start_tick, pitch, .. }
            if lane == "melody" && *start_tick == 3 * BAR + 3 * EIGHTH && i32::from(*pitch) == tonic + 12)));
        let ids: HashSet<_> = score
            .sections
            .iter()
            .flat_map(|s| &s.events)
            .map(|e| match e {
                MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
            })
            .collect();
        assert_eq!(
            ids.len(),
            score.sections.iter().map(|s| s.events.len()).sum::<usize>()
        );
    }

    #[test]
    fn extreme_traits_stay_valid_and_change_texture() {
        let mut a = input("bounds");
        a.energy = f64::NAN;
        a.complexity = f64::INFINITY;
        a.brightness = -1.0;
        a.syncopation = 2.0;
        let score = generate_folklore(&a, "seeded").unwrap();
        assert!(score.bpm.is_finite());
        assert_eq!(score.validate(), Ok(()));
        let mut normal = a;
        normal.energy = 0.5;
        normal.complexity = 0.5;
        normal.brightness = 0.0;
        normal.syncopation = 1.0;
        assert_eq!(
            serialized(&score),
            serialized(&generate_folklore(&normal, "seeded").unwrap())
        );
        normal.complexity = 0.1;
        let sparse = generate_folklore(&normal, "seeded").unwrap();
        assert!(
            sparse.section("primera").unwrap().events.len()
                < score.section("primera").unwrap().events.len()
        );
    }

    #[test]
    fn original_sections_keep_bar_lengths_and_voices() {
        let a = input("stable");
        let score = generate_folklore(&a, "seeded").unwrap();
        assert_eq!(score.sections.len(), 9);
        for (section, bars) in score.sections[..6].iter().zip([4, 8, 4, 8, 8, 4]) {
            assert_eq!(section.length_ticks / BAR, bars);
            for event in &section.events {
                assert!(
                    matches!(event.voice(), "nylon-guitar" | "bombo" | "bombo-rim"),
                    "{} leaked a non-chacarera voice",
                    section.id
                );
            }
        }
    }

    #[test]
    fn new_phases_are_connected_and_distinct() {
        let a = input("phases");
        let score = generate_folklore(&a, "seeded").unwrap();

        for id in ["punteo", "respiro", "pena"] {
            assert!(score.section(id).is_some(), "{id} section missing");
            assert!(score
                .form
                .as_ref()
                .unwrap()
                .steps
                .iter()
                .any(|step| step.section == id));
        }

        let punteo = score.section("punteo").unwrap();
        let melody_ticks: Vec<_> = punteo
            .events
            .iter()
            .filter(|e| e.is_melody())
            .map(MusicEvent::start_tick)
            .collect();
        assert!(melody_ticks.contains(&0));
        assert!(melody_ticks.contains(&(BAR + EIGHTH)));

        let respiro = score.section("respiro").unwrap();
        let pena = score.section("pena").unwrap();
        let mean_velocity = |s: &PortableSection| {
            let velocities: Vec<f64> = s.events.iter().map(MusicEvent::velocity).collect();
            velocities.iter().sum::<f64>() / velocities.len() as f64
        };
        assert!(
            mean_velocity(respiro) < mean_velocity(pena),
            "respiro must sit under the peña celebration"
        );
        for section in [respiro, pena] {
            assert!(section
                .events
                .iter()
                .any(|e| e.voice() == "bombo" || e.voice() == "bombo-rim"));
        }

        let break_start = 5 * BAR;
        let break_events: Vec<_> = pena
            .events
            .iter()
            .filter(|e| e.start_tick() >= break_start && e.start_tick() < break_start + BAR)
            .collect();
        assert!(break_events
            .iter()
            .all(|e| matches!(e, MusicEvent::Percussion { .. }) || e.is_melody()));
        assert!(!break_events.iter().any(|e| {
            matches!(e, MusicEvent::Note { lane, .. } if lane == "bass" || lane == "rasgueado")
        }));
        assert_eq!(
            break_events
                .iter()
                .filter(|e| matches!(e, MusicEvent::Percussion { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn style_parse_accepts_default_and_carnavalito() {
        assert_eq!(FolkloreStyle::parse(""), Ok(FolkloreStyle::Chacarera));
        assert_eq!(
            FolkloreStyle::parse("chacarera"),
            Ok(FolkloreStyle::Chacarera)
        );
        assert_eq!(
            FolkloreStyle::parse("carnavalito"),
            Ok(FolkloreStyle::Carnavalito)
        );
        assert!(FolkloreStyle::parse("zamba").is_err());
        assert!(FolkloreStyle::parse("gato").is_err());
        assert_eq!(FolkloreStyle::Chacarera.as_str(), "chacarera");
        assert_eq!(FolkloreStyle::Carnavalito.as_str(), "carnavalito");
    }

    #[test]
    fn carnavalito_is_binary_pentatonic_and_distinct() {
        let a = input("andes");
        let score = generate_folklore_with_style(&a, FolkloreStyle::Carnavalito, "seeded").unwrap();
        assert_eq!(score.validate(), Ok(()));
        assert_eq!(score.beats_per_bar, 2);
        assert_eq!(
            score.title,
            format!(
                "Folklore {} Carnavalito",
                NOTE_NAMES
                    [folklore_root_pitch_class_with_style(&a, FolkloreStyle::Carnavalito) as usize]
                    .to_uppercase()
            )
        );

        let sections: Vec<_> = score.sections.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            sections,
            CARNAVALITO_PLANS.iter().map(|p| p.id).collect::<Vec<_>>()
        );

        // Distinct from the Chacarera score for the same input.
        let chacarera = generate_folklore(&a, "seeded").unwrap();
        assert_ne!(chacarera.id, score.id);
        assert_ne!(chacarera.beats_per_bar, score.beats_per_bar);

        // Every pitched event sits on the Andean minor pentatonic over the root.
        let root = folklore_root_pitch_class_with_style(&a, FolkloreStyle::Carnavalito);
        let pentatonic_classes: HashSet<i32> = PENTATONIC
            .iter()
            .map(|d| (root + d).rem_euclid(12))
            .collect();
        for section in &score.sections {
            for event in &section.events {
                if let MusicEvent::Note { pitch, .. } = event {
                    assert!(
                        pentatonic_classes.contains(&i32::from(*pitch).rem_euclid(12)),
                        "pitch {} off the pentatonic",
                        pitch
                    );
                }
            }
        }

        // The voices carry the Andean color, and the bombo holds the base.
        let voices: HashSet<&str> = score
            .sections
            .iter()
            .flat_map(|s| &s.events)
            .map(|e| e.voice())
            .collect();
        for voice in ["charango", "quena", "bombo"] {
            assert!(voices.contains(voice), "missing voice {voice}");
        }
        assert!(!voices.contains("nylon-guitar"));
    }

    #[test]
    fn carnavalito_root_matches_bass_and_is_deterministic() {
        let a = input("root");
        let root = folklore_root_pitch_class_with_style(&a, FolkloreStyle::Carnavalito);
        assert_eq!(
            root,
            folklore_root_pitch_class_with_style(&a, FolkloreStyle::Carnavalito)
        );
        let score = generate_folklore_with_style(&a, FolkloreStyle::Carnavalito, "seeded").unwrap();
        assert_eq!(
            serialized(&score),
            serialized(
                &generate_folklore_with_style(&a, FolkloreStyle::Carnavalito, "seeded").unwrap()
            )
        );
        // The tonic is 48 + root; the lowest charango/quena note confirms the key.
        let preludio = score.section("preludio").unwrap();
        let lowest = preludio
            .events
            .iter()
            .filter_map(|e| match e {
                MusicEvent::Note { pitch, .. } => Some(i32::from(*pitch)),
                _ => None,
            })
            .min()
            .unwrap();
        assert_eq!(lowest.rem_euclid(12), root.rem_euclid(12));
        assert_eq!(score.default_section, "preludio");
    }

    #[test]
    fn carnavalito_all_phases_tours_each_section() {
        let a = input("tour");
        let score =
            generate_folklore_with_style(&a, FolkloreStyle::Carnavalito, "all-phases").unwrap();
        assert_eq!(
            score
                .form
                .as_ref()
                .unwrap()
                .steps
                .iter()
                .map(|s| s.section.as_str())
                .collect::<Vec<_>>(),
            CARNAVALITO_PLANS.iter().map(|p| p.id).collect::<Vec<_>>()
        );
        assert_eq!(score.form.as_ref().unwrap().loop_from, Some(0));
    }

    #[test]
    fn carnavalito_traits_change_musical_events() {
        let baseline = input("traits");
        let original =
            generate_folklore_with_style(&baseline, FolkloreStyle::Carnavalito, "seeded").unwrap();
        for (name, update) in [
            ("energy", 0),
            ("complexity", 1),
            ("brightness", 2),
            ("syncopation", 3),
        ] {
            let mut changed = baseline.clone();
            match update {
                0 => changed.energy = 1.0,
                1 => changed.complexity = 1.0,
                2 => changed.brightness = 1.0,
                _ => changed.syncopation = 1.0,
            }
            let score =
                generate_folklore_with_style(&changed, FolkloreStyle::Carnavalito, "seeded")
                    .unwrap();
            assert_ne!(
                serialized(&original.sections),
                serialized(&score.sections),
                "{name}"
            );
            assert_ne!(original.id, score.id, "{name}");
        }
    }

    #[test]
    fn carnavalito_many_seeds_and_extreme_traits_remain_bounded() {
        for index in 0..48 {
            let mut a = input(&format!("bounds-{index}"));
            a.energy = if index % 2 == 0 { f64::NAN } else { 5.0 };
            a.complexity = if index % 3 == 0 { f64::INFINITY } else { -1.0 };
            a.brightness = if index % 2 == 0 { -1.0 } else { 2.0 };
            a.syncopation = if index % 3 == 0 {
                f64::NEG_INFINITY
            } else {
                2.0
            };
            let score =
                generate_folklore_with_style(&a, FolkloreStyle::Carnavalito, "seeded").unwrap();
            assert_eq!(score.validate(), Ok(()));
            assert!(score.sections.iter().all(|section| section
                .events
                .iter()
                .all(|event| event.start_tick() + event.duration_ticks() <= section.length_ticks)));
        }
        let mut outside = input("normalized");
        outside.energy = f64::NAN;
        outside.complexity = f64::INFINITY;
        outside.brightness = -1.0;
        outside.syncopation = 2.0;
        let mut normalized = outside.clone();
        normalized.energy = 0.5;
        normalized.complexity = 0.5;
        normalized.brightness = 0.0;
        normalized.syncopation = 1.0;
        assert_eq!(
            serialized(
                &generate_folklore_with_style(&outside, FolkloreStyle::Carnavalito, "seeded")
                    .unwrap()
            ),
            serialized(
                &generate_folklore_with_style(&normalized, FolkloreStyle::Carnavalito, "seeded")
                    .unwrap()
            )
        );
    }
}
