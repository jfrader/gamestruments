use crate::rng::DeterministicRandom;
use crate::score::{MusicEvent, PortableSection};
use crate::theory::{mode_intervals, phrase_gain, scale_pitch};

use super::{AdventureStyle, NormalizedTraits};

const BEATS_PER_BAR: u32 = 4;
const PHRASE_BARS: u32 = 4;
const MELODY_MIN: i32 = 55;
const MELODY_MAX: i32 = 84;
const NYLON_GUITAR: &str = "nylon-guitar";
const GUITAR_LOW_E: i32 = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scene {
    Camp,
    Explore,
    Town,
    Dungeon,
    Combat,
    Boss,
    Sanctuary,
    Victory,
    // Combat set — the skirmish / assault / chase family of driving phases.
    Skirmish,
    Assault,
    Chase,
    // Happiness set — the festival / reunion / dawn family of bright phases.
    Festival,
    Reunion,
    Dawn,
}

pub(super) struct SectionPlan {
    pub(super) id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    pub(super) bars: u32,
    pub(super) scene: Scene,
    performance: Performance,
}

#[derive(Clone, Copy)]
enum Performance {
    Standard,
    Trail,
    Courtyard,
}

impl Performance {
    fn lead(self, style: AdventureStyle, scene: Scene) -> &'static str {
        match self {
            Self::Standard => melody_voice(style, scene),
            Self::Trail | Self::Courtyard => NYLON_GUITAR,
        }
    }

    fn pedal_voice(self, style: AdventureStyle) -> &'static str {
        match self {
            Self::Standard => match style {
                AdventureStyle::Folk => "harp",
                AdventureStyle::Dark | AdventureStyle::Orchestral => "vielle",
            },
            Self::Trail | Self::Courtyard => NYLON_GUITAR,
        }
    }

    fn bass_voice(self, style: AdventureStyle) -> &'static str {
        match self {
            Self::Standard => match style {
                AdventureStyle::Folk | AdventureStyle::Dark => "harp",
                AdventureStyle::Orchestral => "vielle",
            },
            Self::Trail | Self::Courtyard => NYLON_GUITAR,
        }
    }

    fn pluck_voice(self) -> &'static str {
        match self {
            Self::Standard => "harp",
            Self::Trail | Self::Courtyard => NYLON_GUITAR,
        }
    }

    fn low_register(self) -> i32 {
        match self {
            Self::Standard => 36,
            Self::Trail | Self::Courtyard => GUITAR_LOW_E,
        }
    }

    fn pedal_start(self, style: AdventureStyle, scene: Scene, phrase: u32) -> bool {
        match self {
            Self::Standard => standard_pedal_start(scene, style, phrase),
            Self::Trail => phrase == 0,
            Self::Courtyard => phrase.is_multiple_of(2),
        }
    }

    fn bass_onsets(
        self,
        style: AdventureStyle,
        scene: Scene,
        final_bar: bool,
    ) -> &'static [(u8, i32)] {
        if final_bar {
            return &[(0, 0)];
        }
        match self {
            Self::Standard => standard_bass_onsets(style, scene),
            Self::Trail => &[(0, 0), (4, 4)],
            Self::Courtyard => &[(0, 0), (3, 4), (6, 0)],
        }
    }

    fn harmony_active(
        self,
        style: AdventureStyle,
        scene: Scene,
        kind: PhraseKind,
        bar: u32,
        local_bar: u32,
    ) -> bool {
        match self {
            Self::Standard => standard_harmony_active(style, scene, kind, bar, local_bar),
            Self::Trail => local_bar.is_multiple_of(2) || matches!(kind, PhraseKind::Cadence),
            Self::Courtyard => !matches!(kind, PhraseKind::Antecedent) || local_bar > 0,
        }
    }

    fn harmony_sound(self, style: AdventureStyle, bar_ticks: u32) -> (u32, &'static str, f64) {
        match self {
            Self::Trail | Self::Courtyard => (bar_ticks * 3 / 4, NYLON_GUITAR, 1.0),
            Self::Standard => match style {
                AdventureStyle::Folk => (bar_ticks * 3 / 8, "harp", 1.0),
                AdventureStyle::Dark => (bar_ticks * 7 / 8, "vielle", 1.0),
                AdventureStyle::Orchestral => (bar_ticks * 15 / 16, "vielle", 0.8),
            },
        }
    }

    fn pluck_pattern(
        self,
        style: AdventureStyle,
        scene: Scene,
        bar: u32,
        kind: PhraseKind,
        variant: usize,
    ) -> Vec<(u8, i32)> {
        if matches!(kind, PhraseKind::Cadence) && bar % PHRASE_BARS == 3 {
            return vec![(0, 0)];
        }
        match self {
            Self::Standard => standard_harp_pattern(style, scene, bar, kind, variant),
            Self::Trail => match (bar + variant as u32) % 4 {
                0 => vec![(0, 0), (5, 4)],
                1 => vec![(2, 2), (6, 0)],
                2 => vec![(1, 4), (4, 2)],
                _ => vec![(3, 0), (7, 4)],
            },
            Self::Courtyard => match (bar + variant as u32) % 2 {
                0 => vec![(0, 0), (3, 2), (6, 4)],
                _ => vec![(0, 0), (2, 4), (5, 2), (7, 0)],
            },
        }
    }

    fn pluck_duration(self, style: AdventureStyle, scene: Scene, pulse: u32) -> u32 {
        match self {
            Self::Trail => pulse * 6 / 5,
            Self::Courtyard => pulse * 4 / 5,
            Self::Standard => match style {
                AdventureStyle::Folk if scene == Scene::Town => pulse * 3 / 4,
                AdventureStyle::Folk => pulse * 5 / 4,
                AdventureStyle::Dark => pulse * 7 / 4,
                AdventureStyle::Orchestral => pulse * 6 / 5,
            },
        }
    }

    fn melody_onsets(
        self,
        style: AdventureStyle,
        scene: Scene,
        kind: PhraseKind,
        local_bar: u32,
        variant: usize,
        motion: f64,
    ) -> Vec<u8> {
        if matches!(kind, PhraseKind::Cadence) && local_bar == PHRASE_BARS - 1 {
            return vec![0];
        }
        if local_bar == PHRASE_BARS - 1 {
            return vec![0, 4];
        }
        match self {
            Self::Standard => {
                standard_melody_onsets(style, scene, kind, local_bar, variant, motion)
            }
            Self::Trail => match (local_bar as usize + variant) % 3 {
                0 => vec![0, 5],
                1 => vec![2, 6],
                _ => vec![1, 4],
            },
            Self::Courtyard => match (local_bar as usize + variant) % 3 {
                0 => vec![0, 3, 6],
                1 => vec![0, 2, 5],
                _ => vec![1, 4, 7],
            },
        }
    }

    fn melody_duration(self, style: AdventureStyle, scene: Scene, gap: u32, pulse: u32) -> u32 {
        let duration = match self {
            Self::Trail => gap * 4 / 5,
            Self::Courtyard => gap * 2 / 3,
            Self::Standard => match style {
                AdventureStyle::Folk if scene == Scene::Town => gap * 3 / 5,
                AdventureStyle::Folk => gap * 3 / 4,
                AdventureStyle::Dark => gap * 4 / 5,
                AdventureStyle::Orchestral if matches!(scene, Scene::Combat | Scene::Boss) => {
                    gap * 2 / 3
                }
                AdventureStyle::Orchestral => gap * 7 / 8,
            },
        };
        duration.max(pulse / 2)
    }

    fn percussion(
        self,
        style: AdventureStyle,
        scene: Scene,
        phrase: u32,
        local_bar: u32,
        motion: f64,
    ) -> &'static [(&'static str, u8)] {
        if local_bar == PHRASE_BARS - 1 && phrase > 0 {
            return &[];
        }
        match self {
            Self::Standard => standard_percussion_pattern(style, scene, phrase, motion),
            Self::Trail if phrase % 3 == 1 && motion > 0.4 => &[("bombo", 0), ("bombo-rim", 6)],
            Self::Trail => &[("bombo", 0)],
            Self::Courtyard => &[
                ("bombo", 0),
                ("bombo-rim", 3),
                ("bombo", 6),
                ("bombo-rim", 7),
            ],
        }
    }
}

pub(super) const SECTION_PLANS: &[SectionPlan; 16] = &[
    SectionPlan {
        id: "camp",
        label: "Trailhead Camp",
        feeling: "hearthlight / the road ahead",
        color: "#d5aa64",
        bars: 16,
        scene: Scene::Camp,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "explore",
        label: "The Old Forest",
        feeling: "open paths / old wonders",
        color: "#78976a",
        bars: 32,
        scene: Scene::Explore,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "explore-strings",
        label: "The Winding Trail",
        feeling: "winding paths / distant strings",
        color: "#7a9a7a",
        bars: 32,
        scene: Scene::Explore,
        performance: Performance::Trail,
    },
    SectionPlan {
        id: "town",
        label: "Hearth and Hall",
        feeling: "market dance / crowded tables",
        color: "#d59a42",
        bars: 32,
        scene: Scene::Town,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "town-strings",
        label: "Courtyard Dance",
        feeling: "plucked steps / courtyard reels",
        color: "#d59a52",
        bars: 32,
        scene: Scene::Town,
        performance: Performance::Courtyard,
    },
    SectionPlan {
        id: "festival",
        label: "The Green Market",
        feeling: "dancing feet / raised cups",
        color: "#d9a441",
        bars: 32,
        scene: Scene::Festival,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "reunion",
        label: "Homecoming Hearth",
        feeling: "warm embraces / old names",
        color: "#d59a66",
        bars: 32,
        scene: Scene::Reunion,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "dungeon",
        label: "The Deep Halls",
        feeling: "cold stone / distant steps",
        color: "#59616c",
        bars: 16,
        scene: Scene::Dungeon,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "skirmish",
        label: "Steel in the Brush",
        feeling: "blades flash / first blood",
        color: "#b5543f",
        bars: 16,
        scene: Scene::Skirmish,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "combat",
        label: "Steel and Shadow",
        feeling: "measured pursuit / battle joined",
        color: "#ad5540",
        bars: 32,
        scene: Scene::Combat,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "chase",
        label: "Pursuit",
        feeling: "hearts pound / ground blurs",
        color: "#a8602e",
        bars: 32,
        scene: Scene::Chase,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "boss",
        label: "No Retreat",
        feeling: "ancient dread / final challenge",
        color: "#713343",
        bars: 16,
        scene: Scene::Boss,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "assault",
        label: "The Red Charge",
        feeling: "full charge / no quarter",
        color: "#8f2f34",
        bars: 16,
        scene: Scene::Assault,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "sanctuary",
        label: "The Hidden Glade",
        feeling: "clear water / shelter found",
        color: "#9bc78b",
        bars: 16,
        scene: Scene::Sanctuary,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "dawn",
        label: "First Light",
        feeling: "soft gold / the long night breaks",
        color: "#cfe3a0",
        bars: 16,
        scene: Scene::Dawn,
        performance: Performance::Standard,
    },
    SectionPlan {
        id: "victory",
        label: "Lanterns at Dawn",
        feeling: "homecoming / earned release",
        color: "#ead27e",
        bars: 32,
        scene: Scene::Victory,
        performance: Performance::Standard,
    },
];

/// The role an Adventure section plays when it is composed into a song form,
/// derived from its authored [`Scene`]. It describes the material, it never
/// changes how that material is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AdventurePhaseRole {
    Intro,
    Groove,
    Build,
    Peak,
    Break,
    Outro,
}

/// The role of an Adventure scene, read back from its authored plan. Pure,
/// additive metadata used only by the composed arrangement.
pub(super) fn adventure_phase_role(scene: Scene) -> AdventurePhaseRole {
    match scene {
        Scene::Camp => AdventurePhaseRole::Intro,
        Scene::Explore | Scene::Town => AdventurePhaseRole::Groove,
        Scene::Dungeon => AdventurePhaseRole::Build,
        Scene::Combat | Scene::Boss => AdventurePhaseRole::Peak,
        Scene::Sanctuary => AdventurePhaseRole::Break,
        Scene::Victory => AdventurePhaseRole::Outro,
        // Combat set lands in the Peak band; the happiness set spreads across the
        // groove (festival, reunion) and break (dawn) bands.
        Scene::Skirmish | Scene::Assault | Scene::Chase => AdventurePhaseRole::Peak,
        Scene::Festival | Scene::Reunion => AdventurePhaseRole::Groove,
        Scene::Dawn => AdventurePhaseRole::Break,
    }
}

/// The energy band of an Adventure scene on a 0-100 scale, derived from its
/// authored [`scene_energy`]. Boss (0.9) reads as the climax at 90.
pub(super) fn adventure_phase_energy(scene: Scene) -> u32 {
    (scene_energy(scene) * 100.0).round() as u32
}

const TONIC_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const MOTIFS: [[i32; 6]; 6] = [
    [0, 2, 1, 3, 2, 0],
    [0, 1, 3, 2, 4, 2],
    [0, 3, 2, 1, 2, 0],
    [0, 2, 4, 3, 1, 2],
    [0, 1, 2, 4, 3, 1],
    [0, 3, 1, 2, 4, 2],
];

pub(super) struct PieceDna {
    pub(super) tonic_pitch_class: i32,
    motif: [i32; 6],
    rhythm_variant: usize,
    harmony_variant: usize,
}

impl PieceDna {
    pub(super) fn new(seed: u32) -> Self {
        let mut rng = DeterministicRandom::new(seed);
        let tonic_pitch_class = *rng.pick(&TONIC_PITCH_CLASSES);
        let mut motif = *rng.pick(&MOTIFS);
        let changed = 1 + rng.integer((motif.len() - 1) as u32) as usize;
        motif[changed] += *rng.pick(&[-1, 1]);
        motif[0] = 0;
        Self {
            tonic_pitch_class,
            motif,
            rhythm_variant: rng.integer(8) as usize,
            harmony_variant: rng.integer(3) as usize,
        }
    }
}

pub(super) fn tempo(style: AdventureStyle, traits: NormalizedTraits) -> f64 {
    let base = match style {
        AdventureStyle::Folk => 92.0,
        AdventureStyle::Dark => 78.0,
        AdventureStyle::Orchestral => 100.0,
    };
    (base + traits.motion * 20.0 - traits.mystery * 5.0)
        .clamp(70.0, 126.0)
        .round()
}

pub(super) fn mode_for(style: AdventureStyle, scene: Scene) -> &'static str {
    match (style, scene) {
        // Folk — a small, lively ensemble. Warm, bright, and danceable across
        // the safe phases; only the dangerous scenes turn minor.
        (AdventureStyle::Folk, Scene::Camp | Scene::Victory) => "ionian",
        (AdventureStyle::Folk, Scene::Explore | Scene::Town) => "mixolydian",
        (AdventureStyle::Folk, Scene::Sanctuary) => "lydian",
        (AdventureStyle::Folk, Scene::Dungeon | Scene::Boss) => "aeolian",
        (AdventureStyle::Folk, Scene::Combat) => "dorian",
        // Folk combat set: skirmish and chase keep the combat's dorian snap,
        // assault hardens to aeolian like the boss.
        (AdventureStyle::Folk, Scene::Skirmish | Scene::Chase) => "dorian",
        (AdventureStyle::Folk, Scene::Assault) => "aeolian",
        // Folk happiness set: bright mixolydian/ionian for the dance, lydian dawn.
        (AdventureStyle::Folk, Scene::Festival) => "mixolydian",
        (AdventureStyle::Folk, Scene::Reunion) => "ionian",
        (AdventureStyle::Folk, Scene::Dawn) => "lydian",
        // Dark — low, spacious, drone-driven. Bittersweet (dorian) where safe,
        // phrygian/aeolian where dangerous.
        (AdventureStyle::Dark, Scene::Dungeon | Scene::Boss) => "phrygian",
        (AdventureStyle::Dark, Scene::Explore | Scene::Combat) => "aeolian",
        (AdventureStyle::Dark, Scene::Victory) => "mixolydian",
        (AdventureStyle::Dark, Scene::Skirmish | Scene::Chase) => "aeolian",
        (AdventureStyle::Dark, Scene::Assault) => "phrygian",
        // Dark happiness set: festival and reunion stay bittersweet (dorian),
        // dawn earns the bright mixolydian release like victory.
        (AdventureStyle::Dark, Scene::Festival | Scene::Reunion) => "dorian",
        (AdventureStyle::Dark, Scene::Dawn) => "mixolydian",
        (AdventureStyle::Dark, _) => "dorian",
        // Orchestral — broad bowed strings, heroic and warm. Major through the
        // safe arc, rising to a triumphant Lydian victory.
        (AdventureStyle::Orchestral, Scene::Dungeon | Scene::Boss) => "aeolian",
        (AdventureStyle::Orchestral, Scene::Combat) => "dorian",
        (AdventureStyle::Orchestral, Scene::Victory) => "lydian",
        (AdventureStyle::Orchestral, Scene::Skirmish | Scene::Chase) => "dorian",
        (AdventureStyle::Orchestral, Scene::Assault) => "aeolian",
        // Orchestral happiness set: ionian for the celebration, lydian dawn.
        (AdventureStyle::Orchestral, Scene::Festival | Scene::Reunion) => "ionian",
        (AdventureStyle::Orchestral, Scene::Dawn) => "lydian",
        (AdventureStyle::Orchestral, _) => "ionian",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PhraseKind {
    Antecedent,
    Consequent,
    Development,
    Return,
    Cadence,
}

pub(super) fn phrase_kind(index: u32, phrase_count: u32) -> PhraseKind {
    match phrase_count {
        4 => [
            PhraseKind::Antecedent,
            PhraseKind::Consequent,
            PhraseKind::Development,
            PhraseKind::Cadence,
        ][index as usize],
        8 => [
            PhraseKind::Antecedent,
            PhraseKind::Consequent,
            PhraseKind::Development,
            PhraseKind::Development,
            PhraseKind::Return,
            PhraseKind::Consequent,
            PhraseKind::Return,
            PhraseKind::Cadence,
        ][index as usize],
        _ => unreachable!("adventure sections contain four or eight phrases"),
    }
}

/// Functional chord degrees for a mode, each guaranteed to be a consonant
/// (major or minor) triad — never the mode's diminished triad. This is what the
/// old mode-blind progression was missing: degree 4 is a strong major dominant
/// in Ionian/Lydian but a weak minor v in Mixolydian/Dorian; degree 3 is major
/// in Ionian but a diminished #iv° in Lydian; degree 2 is a minor iii in
/// Ionian but a diminished iii° in Mixolydian.
struct ModeHarmony {
    /// Degree of the tonic (always 0).
    tonic: i32,
    /// Degree of the chord that pulls home (major V, or a modal bVII/bII).
    dominant: i32,
    /// Degree of the predominant chord.
    subdominant: i32,
    /// Stable color degrees to vary the middle bars without leaving the mode.
    colors: &'static [i32],
}

fn mode_harmony(mode: &str) -> ModeHarmony {
    match mode {
        "ionian" => ModeHarmony {
            tonic: 0,
            dominant: 4,
            subdominant: 3,
            colors: &[1, 5],
        },
        "lydian" => ModeHarmony {
            tonic: 0,
            dominant: 4,
            subdominant: 1,
            colors: &[5, 2],
        },
        "mixolydian" => ModeHarmony {
            tonic: 0,
            dominant: 6,
            subdominant: 3,
            colors: &[5, 1],
        },
        "dorian" => ModeHarmony {
            tonic: 0,
            dominant: 6,
            subdominant: 3,
            colors: &[2, 1],
        },
        "aeolian" => ModeHarmony {
            tonic: 0,
            dominant: 6,
            subdominant: 5,
            colors: &[2, 3],
        },
        "phrygian" => ModeHarmony {
            tonic: 0,
            dominant: 1,
            subdominant: 5,
            colors: &[2, 3],
        },
        _ => ModeHarmony {
            tonic: 0,
            dominant: 4,
            subdominant: 3,
            colors: &[1, 5],
        },
    }
}

/// The scale degree for an open phrase ending (V or a modal turnaround), so
/// melody and accompaniment agree on the arrival.
pub(super) fn dominant_degree(mode: &str) -> i32 {
    mode_harmony(mode).dominant
}

pub(super) fn progression(
    mode: &str,
    kind: PhraseKind,
    scene: Scene,
    phrase_index: u32,
    variant: usize,
) -> [i32; 4] {
    let harmony = mode_harmony(mode);
    let shift = scene_index(scene);
    let color = harmony.colors[(shift + phrase_index as usize + variant) % harmony.colors.len()];
    let color2 =
        harmony.colors[(shift + phrase_index as usize + variant + 1) % harmony.colors.len()];
    match kind {
        // I — predominant — dominant — dominant (open half cadence).
        PhraseKind::Antecedent => [
            harmony.tonic,
            harmony.subdominant,
            harmony.dominant,
            harmony.dominant,
        ],
        // I — color — turnaround — I (answered return).
        PhraseKind::Consequent => [harmony.tonic, color, harmony.dominant, harmony.tonic],
        // Predominant — color — dominant — dominant (departure, half cadence).
        PhraseKind::Development => [
            harmony.subdominant,
            color,
            harmony.dominant,
            harmony.dominant,
        ],
        // I — predominant — dominant — I (restatement).
        PhraseKind::Return => [
            harmony.tonic,
            harmony.subdominant,
            harmony.dominant,
            harmony.tonic,
        ],
        // Predominant — color — dominant — I (final cadence).
        PhraseKind::Cadence => [harmony.subdominant, color2, harmony.dominant, harmony.tonic],
    }
}

pub(super) fn build_sections<F>(
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    ticks_per_beat: u32,
    section_seed: F,
) -> Vec<PortableSection>
where
    F: Fn(&str) -> u32,
{
    SECTION_PLANS
        .iter()
        .map(|plan| {
            build_section(
                plan,
                style,
                traits,
                dna,
                ticks_per_beat,
                section_seed(plan.id),
            )
        })
        .collect()
}

struct EventCounters {
    bass: usize,
    pedal: usize,
    harmony: usize,
    harp: usize,
    melody: usize,
    bell: usize,
    percussion: usize,
    recorder: usize,
}

impl EventCounters {
    fn new() -> Self {
        Self {
            bass: 0,
            pedal: 0,
            harmony: 0,
            harp: 0,
            melody: 0,
            bell: 0,
            percussion: 0,
            recorder: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_note(
    events: &mut Vec<MusicEvent>,
    section: &str,
    lane: &str,
    index: &mut usize,
    start_tick: u32,
    duration_ticks: u32,
    velocity: f64,
    pitch: i32,
    voice: &str,
    melody: bool,
) {
    let pitch = u8::try_from(pitch).expect("adventure pitch must remain within MIDI range");
    events.push(MusicEvent::Note {
        id: format!("{section}:{lane}:{}", *index),
        section: section.to_string(),
        lane: lane.to_string(),
        start_tick,
        duration_ticks: duration_ticks.max(1),
        velocity: velocity.clamp(0.04, 0.95),
        pitch,
        voice: voice.to_string(),
        role: melody.then(|| "melody".to_string()),
    });
    *index += 1;
}

#[allow(clippy::too_many_arguments)]
fn push_percussion(
    events: &mut Vec<MusicEvent>,
    section: &str,
    index: &mut usize,
    start_tick: u32,
    duration_ticks: u32,
    velocity: f64,
    voice: &str,
) {
    events.push(MusicEvent::Percussion {
        id: format!("{section}:percussion:{}", *index),
        section: section.to_string(),
        lane: "percussion".to_string(),
        start_tick,
        duration_ticks: duration_ticks.max(1),
        velocity: velocity.clamp(0.04, 0.95),
        voice: voice.to_string(),
    });
    *index += 1;
}

fn scene_index(scene: Scene) -> usize {
    match scene {
        Scene::Camp => 0,
        Scene::Explore => 1,
        Scene::Town => 2,
        Scene::Dungeon => 3,
        Scene::Combat => 4,
        Scene::Boss => 5,
        Scene::Sanctuary => 6,
        Scene::Victory => 7,
        Scene::Skirmish => 8,
        Scene::Assault => 9,
        Scene::Chase => 10,
        Scene::Festival => 11,
        Scene::Reunion => 12,
        Scene::Dawn => 13,
    }
}

fn nearest_scale_pitch(
    tonic_pitch_class: i32,
    degree: i32,
    intervals: &[i32],
    target: i32,
    minimum: i32,
    maximum: i32,
) -> i32 {
    let base = scale_pitch(60 + tonic_pitch_class, degree, intervals);
    (-5..=5)
        .map(|octave| base + octave * 12)
        .filter(|pitch| (minimum..=maximum).contains(pitch))
        .min_by_key(|pitch| (pitch - target).abs())
        .expect("adventure register must contain a modal spelling")
}

pub(super) fn voice_chord(
    tonic_pitch_class: i32,
    chord_degree: i32,
    intervals: &[i32],
    previous: Option<[i32; 3]>,
) -> [i32; 3] {
    let targets = previous.unwrap_or([50, 58, 65]);
    let mut result = [0; 3];
    for (voice, chord_offset) in [0, 2, 4].into_iter().enumerate() {
        let minimum = if voice == 0 {
            40
        } else {
            result[voice - 1] + 3
        };
        result[voice] = nearest_scale_pitch(
            tonic_pitch_class,
            chord_degree + chord_offset,
            intervals,
            targets[voice],
            minimum,
            [60, 72, 84][voice],
        );
    }
    result
}

fn melody_voice(style: AdventureStyle, scene: Scene) -> &'static str {
    match (style, scene) {
        // Folk — recorder leads; the vielle adds fiddle color in town, boss,
        // and the heavy combat scenes.
        (AdventureStyle::Folk, Scene::Town | Scene::Boss | Scene::Assault | Scene::Chase) => {
            "vielle"
        }
        (AdventureStyle::Folk, _) => "recorder",
        // Dark — bowed vielle leads, recorder for the sparse camp/sanctuary/dawn.
        (AdventureStyle::Dark, Scene::Camp | Scene::Sanctuary | Scene::Dawn) => "recorder",
        (AdventureStyle::Dark, _) => "vielle",
        // Orchestral — the bowed string (vielle) is always the singing lead.
        (AdventureStyle::Orchestral, _) => "vielle",
    }
}

fn melody_center(style: AdventureStyle, scene: Scene) -> i32 {
    let style_center = match style {
        AdventureStyle::Folk => 67,
        AdventureStyle::Dark => 62,
        AdventureStyle::Orchestral => 70,
    };
    style_center
        + match scene {
            Scene::Camp | Scene::Dungeon => -2,
            Scene::Town | Scene::Combat => 0,
            Scene::Boss => -1,
            Scene::Explore | Scene::Sanctuary => 2,
            Scene::Victory => 3,
            Scene::Skirmish | Scene::Chase => 0,
            Scene::Assault => -1,
            Scene::Festival | Scene::Dawn => 2,
            Scene::Reunion => 1,
        }
}

fn scene_energy(scene: Scene) -> f64 {
    match scene {
        Scene::Camp => 0.25,
        Scene::Explore => 0.48,
        Scene::Town => 0.58,
        Scene::Dungeon => 0.2,
        Scene::Combat => 0.82,
        Scene::Boss => 0.9,
        Scene::Sanctuary => 0.34,
        Scene::Victory => 0.7,
        Scene::Skirmish => 0.76,
        Scene::Assault => 0.87,
        Scene::Chase => 0.8,
        Scene::Festival => 0.62,
        Scene::Reunion => 0.54,
        Scene::Dawn => 0.36,
    }
}

fn texture_gain(kind: PhraseKind, phrase_index: u32) -> f64 {
    let base = match kind {
        PhraseKind::Antecedent => 0.88,
        PhraseKind::Consequent => 0.96,
        PhraseKind::Development => 1.04,
        PhraseKind::Return => 0.94,
        PhraseKind::Cadence => 0.82,
    };
    base + f64::from(phrase_index.min(5)) * 0.012
}

fn standard_pedal_start(scene: Scene, style: AdventureStyle, phrase: u32) -> bool {
    match (style, scene) {
        (AdventureStyle::Dark, Scene::Dungeon | Scene::Boss | Scene::Assault) => {
            phrase.is_multiple_of(2)
        }
        (_, Scene::Dungeon) => phrase == 0 || phrase == 2,
        (AdventureStyle::Folk, Scene::Camp | Scene::Sanctuary | Scene::Dawn) => phrase == 0,
        (AdventureStyle::Orchestral, Scene::Sanctuary) => phrase == 2,
        _ => false,
    }
}

fn pedal_covers(plan: &SectionPlan, style: AdventureStyle, bar: u32) -> bool {
    let phrase = bar / PHRASE_BARS;
    plan.performance.pedal_start(style, plan.scene, phrase) && bar % PHRASE_BARS < 2
}

#[allow(clippy::too_many_arguments)]
fn add_pedal(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    dna: &PieceDna,
    intervals: &[i32],
    phrase: u32,
    bar_ticks: u32,
) {
    if !plan.performance.pedal_start(style, plan.scene, phrase) {
        return;
    }
    let voice = plan.performance.pedal_voice(style);
    let pitch = nearest_scale_pitch(
        dna.tonic_pitch_class,
        0,
        intervals,
        43,
        plan.performance.low_register(),
        52,
    );
    push_note(
        events,
        plan.id,
        "pedal",
        &mut counters.pedal,
        phrase * PHRASE_BARS * bar_ticks,
        2 * bar_ticks,
        0.1 + scene_energy(plan.scene) * 0.05,
        pitch,
        voice,
        false,
    );
}

fn standard_bass_onsets(style: AdventureStyle, scene: Scene) -> &'static [(u8, i32)] {
    match (style, scene) {
        (AdventureStyle::Folk, Scene::Town | Scene::Combat) => &[(0, 0), (4, 4)],
        (AdventureStyle::Folk, Scene::Boss) => &[(0, 0), (6, 0)],
        // Folk combat set: skirmish/chase bounce like combat, assault hammers.
        (AdventureStyle::Folk, Scene::Skirmish | Scene::Chase) => &[(0, 0), (4, 4)],
        (AdventureStyle::Folk, Scene::Assault) => &[(0, 0), (4, 4), (6, 0)],
        // Folk happiness set: festival and reunion bounce like the town dance.
        (AdventureStyle::Folk, Scene::Festival | Scene::Reunion) => &[(0, 0), (4, 4)],
        (AdventureStyle::Dark, Scene::Combat | Scene::Boss) => &[(0, 0), (3, 4), (6, 0)],
        (AdventureStyle::Dark, Scene::Skirmish | Scene::Assault | Scene::Chase) => {
            &[(0, 0), (3, 4), (6, 0)]
        }
        (AdventureStyle::Dark, Scene::Festival | Scene::Reunion) => &[(0, 0), (5, 4)],
        (AdventureStyle::Orchestral, Scene::Town | Scene::Combat | Scene::Victory) => {
            &[(0, 0), (4, 4)]
        }
        (AdventureStyle::Orchestral, Scene::Skirmish | Scene::Chase) => &[(0, 0), (4, 4)],
        (AdventureStyle::Orchestral, Scene::Assault) => &[(0, 0), (6, 0)],
        (AdventureStyle::Orchestral, Scene::Festival | Scene::Reunion) => &[(0, 0), (4, 4)],
        (AdventureStyle::Orchestral, _) => &[(0, 0), (6, 4)],
        _ => &[(0, 0)],
    }
}

#[allow(clippy::too_many_arguments)]
fn add_bass(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    intervals: &[i32],
    chord_degree: i32,
    bar: u32,
    bar_ticks: u32,
    pulse: u32,
) {
    let final_bar = bar == plan.bars - 1;
    if pedal_covers(plan, style, bar) && !final_bar {
        return;
    }
    if matches!(style, AdventureStyle::Folk)
        && matches!(plan.scene, Scene::Camp | Scene::Dungeon | Scene::Sanctuary)
        && bar % 2 == 1
        && !final_bar
    {
        return;
    }
    let voice = plan.performance.bass_voice(style);
    let onsets = plan.performance.bass_onsets(style, plan.scene, final_bar);
    for &(onset, offset) in onsets {
        let pitch = nearest_scale_pitch(
            dna.tonic_pitch_class,
            chord_degree + offset,
            intervals,
            42,
            plan.performance.low_register(),
            55,
        );
        let duration = if onset == 0 && onsets.len() == 1 {
            bar_ticks * 3 / 4
        } else {
            pulse * 3 / 2
        };
        push_note(
            events,
            plan.id,
            "bass",
            &mut counters.bass,
            bar * bar_ticks + u32::from(onset) * pulse,
            duration,
            0.22 + scene_energy(plan.scene) * 0.13 + traits.danger * 0.04,
            pitch,
            voice,
            false,
        );
    }
}

fn standard_harmony_active(
    style: AdventureStyle,
    scene: Scene,
    kind: PhraseKind,
    bar: u32,
    local_bar: u32,
) -> bool {
    if bar == 0 && scene == Scene::Camp && style != AdventureStyle::Orchestral {
        return false;
    }
    match style {
        AdventureStyle::Folk => match scene {
            Scene::Town => local_bar == 0 || matches!(kind, PhraseKind::Cadence),
            Scene::Dungeon => local_bar.is_multiple_of(2),
            _ => local_bar != 1 || matches!(kind, PhraseKind::Development),
        },
        AdventureStyle::Dark => local_bar.is_multiple_of(2) || matches!(kind, PhraseKind::Cadence),
        AdventureStyle::Orchestral => {
            !matches!(kind, PhraseKind::Antecedent) || local_bar > 0 || scene != Scene::Explore
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_harmony(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    kind: PhraseKind,
    chord: [i32; 3],
    bar: u32,
    local_bar: u32,
    bar_ticks: u32,
) {
    if !plan
        .performance
        .harmony_active(style, plan.scene, kind, bar, local_bar)
    {
        return;
    }
    let voices = match style {
        AdventureStyle::Folk | AdventureStyle::Dark => 2,
        AdventureStyle::Orchestral => 3,
    };
    let (duration, voice, backing_weight) = plan.performance.harmony_sound(style, bar_ticks);
    for &pitch in chord.iter().take(voices) {
        push_note(
            events,
            plan.id,
            "harmony",
            &mut counters.harmony,
            bar * bar_ticks,
            duration,
            (0.13 + traits.wonder * 0.06 + scene_energy(plan.scene) * 0.05)
                * phrase_gain(bar)
                * backing_weight,
            pitch,
            voice,
            false,
        );
    }
}

fn standard_harp_pattern(
    style: AdventureStyle,
    scene: Scene,
    bar: u32,
    kind: PhraseKind,
    variant: usize,
) -> Vec<(u8, i32)> {
    match style {
        AdventureStyle::Folk => match scene {
            Scene::Camp => {
                if bar.is_multiple_of(2) {
                    vec![(0, 0), (3, 2), (6, 4)]
                } else {
                    vec![(1, 4), (4, 2)]
                }
            }
            Scene::Explore => match (bar + variant as u32) % 3 {
                0 => vec![(0, 0), (3, 4), (5, 2), (7, 4)],
                1 => vec![(1, 2), (4, 0), (6, 4)],
                _ => vec![(0, 4), (2, 2), (6, 0)],
            },
            Scene::Town => {
                if bar.is_multiple_of(2) {
                    vec![(0, 0), (3, 2), (4, 4), (6, 2)]
                } else {
                    vec![(0, 0), (2, 4), (5, 2), (7, 4)]
                }
            }
            Scene::Dungeon => vec![(0, 0), (6, 4)],
            Scene::Combat => vec![(0, 0), (2, 4), (3, 2), (5, 4), (7, 0)],
            Scene::Boss => vec![(0, 0), (4, 4), (6, 2)],
            Scene::Sanctuary => vec![(0, 0), (3, 4), (7, 2)],
            Scene::Victory => {
                if matches!(kind, PhraseKind::Return | PhraseKind::Cadence) {
                    vec![(0, 0), (4, 2)]
                } else {
                    vec![(0, 0), (2, 2), (4, 4), (6, 2)]
                }
            }
            // Combat set: driving ostinato, denser than the safe phases.
            Scene::Skirmish => vec![(0, 0), (2, 4), (3, 2), (5, 4), (7, 0)],
            Scene::Assault => vec![(0, 0), (1, 4), (3, 2), (4, 4), (6, 2), (7, 0)],
            Scene::Chase => vec![(0, 0), (2, 4), (4, 2), (6, 4)],
            // Happiness set: bright dance for the festival, warm for the reunion,
            // a gentle morning shimmer for dawn.
            Scene::Festival => vec![(0, 0), (2, 4), (3, 2), (4, 4), (6, 2), (7, 4)],
            Scene::Reunion => vec![(0, 0), (3, 2), (4, 4), (6, 2)],
            Scene::Dawn => vec![(0, 0), (3, 4), (7, 2)],
        },
        AdventureStyle::Dark => match scene {
            Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => {
                if bar.is_multiple_of(2) {
                    vec![(0, 0), (3, 4), (4, 0), (7, 2)]
                } else {
                    vec![(0, 0), (5, 4)]
                }
            }
            Scene::Dungeon => {
                if bar.is_multiple_of(2) {
                    vec![(2, 0)]
                } else {
                    Vec::new()
                }
            }
            Scene::Town | Scene::Festival | Scene::Reunion => vec![(0, 0), (5, 4)],
            _ => {
                if bar.is_multiple_of(2) {
                    vec![(0, 0), (6, 4)]
                } else {
                    vec![(3, 2)]
                }
            }
        },
        AdventureStyle::Orchestral => {
            // A few accents, not Folk's running ostinato: the strings carry the
            // foundation, so the harp only decorates arrivals and landmarks.
            match scene {
                Scene::Camp | Scene::Dungeon => vec![(0, 0), (6, 4)],
                Scene::Town => vec![(0, 0), (3, 2), (6, 4)],
                Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => {
                    vec![(0, 0), (4, 4)]
                }
                Scene::Festival => vec![(0, 0), (3, 2), (6, 4)],
                Scene::Reunion | Scene::Dawn => vec![(0, 0), (2, 2), (6, 4)],
                Scene::Explore | Scene::Sanctuary | Scene::Victory => vec![(0, 0), (2, 2), (6, 4)],
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_harp(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    intervals: &[i32],
    kind: PhraseKind,
    chord_degree: i32,
    bar: u32,
    bar_ticks: u32,
    pulse: u32,
    variant: usize,
) {
    if matches!(kind, PhraseKind::Return)
        && bar.is_multiple_of(PHRASE_BARS)
        && style != AdventureStyle::Orchestral
    {
        return;
    }
    let pattern = plan
        .performance
        .pluck_pattern(style, plan.scene, bar, kind, variant);
    for (position, (onset, offset)) in pattern.iter().enumerate() {
        let target = match style {
            AdventureStyle::Dark => 57,
            AdventureStyle::Folk => 62,
            AdventureStyle::Orchestral => 66,
        } + (position as i32 % 2) * 3;
        let pitch = nearest_scale_pitch(
            dna.tonic_pitch_class,
            chord_degree + offset,
            intervals,
            target,
            48,
            76,
        );
        let voice = plan.performance.pluck_voice();
        let duration = plan.performance.pluck_duration(style, plan.scene, pulse);
        push_note(
            events,
            plan.id,
            "harp",
            &mut counters.harp,
            bar * bar_ticks + u32::from(*onset) * pulse,
            duration,
            (0.15 + traits.wonder * 0.08 + scene_energy(plan.scene) * 0.05) * phrase_gain(bar),
            pitch,
            voice,
            false,
        );
    }
}

fn standard_melody_onsets(
    style: AdventureStyle,
    scene: Scene,
    kind: PhraseKind,
    local_bar: u32,
    variant: usize,
    motion: f64,
) -> Vec<u8> {
    if scene == Scene::Camp
        && local_bar == 0
        && matches!(kind, PhraseKind::Antecedent)
        && style != AdventureStyle::Orchestral
    {
        return Vec::new();
    }
    if style == AdventureStyle::Dark && scene == Scene::Dungeon && local_bar == 1 {
        return Vec::new();
    }
    let mut onsets = match style {
        AdventureStyle::Folk => {
            if matches!(scene, Scene::Town | Scene::Festival) {
                match (local_bar as usize + variant) % 3 {
                    0 => vec![0, 3, 4, 6],
                    1 => vec![0, 2, 5, 7],
                    _ => vec![1, 3, 6],
                }
            } else {
                match (local_bar as usize + variant) % 4 {
                    0 => vec![0, 2, 5],
                    1 => vec![1, 4, 6],
                    2 => vec![0, 3, 6],
                    _ => vec![0, 4],
                }
            }
        }
        AdventureStyle::Dark => match (local_bar as usize + variant) % 3 {
            0 => vec![0, 4],
            1 => vec![2, 6],
            _ => vec![0, 5],
        },
        AdventureStyle::Orchestral => match (local_bar as usize + variant) % 3 {
            0 => vec![0, 4],
            1 => vec![0, 3, 6],
            _ => vec![0, 5],
        },
    };
    if motion < 0.3 && onsets.len() > 2 {
        onsets.remove(1 + variant % (onsets.len() - 1));
    } else if motion > 0.78 && style == AdventureStyle::Folk && !onsets.contains(&7) {
        onsets.push(7);
        onsets.sort_unstable();
    }
    onsets
}

fn transformed_motif_degree(
    dna: &PieceDna,
    kind: PhraseKind,
    phrase: u32,
    bar: u32,
    note: usize,
) -> i32 {
    let index = (bar as usize * 2 + note) % dna.motif.len();
    match kind {
        PhraseKind::Antecedent => dna.motif[index],
        PhraseKind::Consequent => {
            if index < 3 {
                dna.motif[index]
            } else {
                -dna.motif[dna.motif.len() - 1 - index]
            }
        }
        PhraseKind::Development => -dna.motif[index] + 1 + (phrase % 2) as i32,
        PhraseKind::Return => dna.motif[(index + 1) % dna.motif.len()] - 1,
        PhraseKind::Cadence => dna.motif[index] / 2,
    }
}

#[allow(clippy::too_many_arguments)]
fn add_melody(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    intervals: &[i32],
    mode: &str,
    kind: PhraseKind,
    phrase: u32,
    chord_degree: i32,
    bar: u32,
    local_bar: u32,
    bar_ticks: u32,
    pulse: u32,
    variant: usize,
    previous_pitch: &mut Option<i32>,
) {
    let onsets = plan
        .performance
        .melody_onsets(style, plan.scene, kind, local_bar, variant, traits.motion);
    for (note, onset) in onsets.iter().enumerate() {
        let is_phrase_arrival = local_bar == PHRASE_BARS - 1 && note == onsets.len() - 1;
        let degree = if is_phrase_arrival {
            match kind {
                PhraseKind::Antecedent | PhraseKind::Development => dominant_degree(mode),
                PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Cadence => 0,
            }
        } else if note == 0 && *onset == 0 {
            // Open a bar on a bright chord tone — the fifth, then the third,
            // then the root — so phrases lift instead of hovering on the tonic.
            chord_degree + [4, 2, 0][(bar as usize + phrase as usize) % 3]
        } else {
            chord_degree + transformed_motif_degree(dna, kind, phrase, local_bar, note)
        };
        let phrase_register = match kind {
            PhraseKind::Antecedent => 1,
            PhraseKind::Consequent => 0,
            PhraseKind::Development => 2,
            PhraseKind::Return => -1,
            PhraseKind::Cadence => -2,
        };
        let center = melody_center(style, plan.scene)
            + phrase_register
            + (traits.wonder * 2.0).round() as i32;
        let target = previous_pitch
            .map(|previous| (previous * 2 + center) / 3)
            .unwrap_or(center);
        let pitch = nearest_scale_pitch(
            dna.tonic_pitch_class,
            degree,
            intervals,
            target,
            MELODY_MIN,
            MELODY_MAX,
        );
        let next = onsets.get(note + 1).copied().unwrap_or(8);
        let gap = u32::from(next - onset) * pulse;
        let duration = if matches!(kind, PhraseKind::Cadence) && local_bar == PHRASE_BARS - 1 {
            pulse * 6
        } else {
            plan.performance
                .melody_duration(style, plan.scene, gap, pulse)
        };
        let articulation = if note == 0 { 0.02 } else { 0.0 };
        let velocity =
            (0.28 + traits.wonder * 0.06 + scene_energy(plan.scene) * 0.1 + articulation)
                * phrase_gain(bar)
                * texture_gain(kind, phrase);
        push_note(
            events,
            plan.id,
            "melody",
            &mut counters.melody,
            bar * bar_ticks + u32::from(*onset) * pulse,
            duration,
            velocity,
            pitch,
            plan.performance.lead(style, plan.scene),
            true,
        );
        *previous_pitch = Some(pitch);
    }
}

#[allow(clippy::too_many_arguments)]
fn add_recorder_answer(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    dna: &PieceDna,
    intervals: &[i32],
    chord_degree: i32,
    bar: u32,
    local_bar: u32,
    bar_ticks: u32,
    pulse: u32,
    variant: usize,
) {
    // Orchestral bows the lead; the recorder only answers, never leads. A short
    // echo lands on the second bar of each phrase in the warm scenes.
    let warm = matches!(
        plan.scene,
        Scene::Camp
            | Scene::Explore
            | Scene::Town
            | Scene::Sanctuary
            | Scene::Victory
            | Scene::Festival
            | Scene::Reunion
            | Scene::Dawn
    );
    if style != AdventureStyle::Orchestral || !warm || local_bar != 1 {
        return;
    }
    let degree = chord_degree + [4, 2][(bar as usize + variant) % 2];
    let pitch = nearest_scale_pitch(
        dna.tonic_pitch_class,
        degree,
        intervals,
        71,
        MELODY_MIN,
        MELODY_MAX,
    );
    push_note(
        events,
        plan.id,
        "recorder",
        &mut counters.recorder,
        bar * bar_ticks + 4 * pulse,
        pulse,
        0.17 + scene_energy(plan.scene) * 0.06,
        pitch,
        "recorder",
        false,
    );
}

fn standard_percussion_pattern(
    style: AdventureStyle,
    scene: Scene,
    phrase: u32,
    motion: f64,
) -> &'static [(&'static str, u8)] {
    match style {
        AdventureStyle::Folk => match scene {
            Scene::Camp if phrase >= 2 => &[("tambourine", 4)],
            Scene::Explore if phrase % 2 == 1 && motion > 0.45 => {
                &[("frame-drum", 0), ("tambourine", 6)]
            }
            Scene::Town => &[
                ("frame-drum", 0),
                ("tambourine", 3),
                ("frame-drum", 4),
                ("tambourine", 7),
            ],
            Scene::Combat => &[
                ("frame-drum", 0),
                ("frame-drum", 3),
                ("tambourine", 5),
                ("frame-drum", 6),
            ],
            Scene::Boss => &[("frame-drum", 0), ("frame-drum", 4), ("tambourine", 7)],
            Scene::Victory if phrase < 6 => {
                &[("frame-drum", 0), ("tambourine", 3), ("tambourine", 6)]
            }
            // Combat set: driving frame-drum work, the assault densest of all.
            Scene::Skirmish => &[
                ("frame-drum", 0),
                ("frame-drum", 3),
                ("tambourine", 5),
                ("frame-drum", 6),
            ],
            Scene::Assault => &[
                ("frame-drum", 0),
                ("frame-drum", 2),
                ("tambourine", 3),
                ("frame-drum", 4),
                ("frame-drum", 6),
                ("tambourine", 7),
            ],
            Scene::Chase => &[
                ("frame-drum", 0),
                ("frame-drum", 2),
                ("frame-drum", 4),
                ("tambourine", 5),
                ("frame-drum", 6),
            ],
            // Happiness set: bright tambourine/frame-drum for the dance and the
            // hearth, a single shimmer for dawn.
            Scene::Festival => &[
                ("frame-drum", 0),
                ("tambourine", 2),
                ("frame-drum", 4),
                ("tambourine", 6),
            ],
            Scene::Reunion => &[("frame-drum", 0), ("tambourine", 3), ("tambourine", 6)],
            Scene::Dawn => &[("tambourine", 4)],
            _ => &[],
        },
        AdventureStyle::Dark => match scene {
            Scene::Town => &[("frame-drum", 0), ("tambourine", 6)],
            Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => &[
                ("frame-drum", 0),
                ("frame-drum", 3),
                ("frame-drum", 6),
                ("tambourine", 7),
            ],
            Scene::Dungeon if phrase >= 2 => &[("frame-drum", 0)],
            Scene::Victory if phrase < 4 => &[("frame-drum", 0), ("frame-drum", 6)],
            Scene::Festival | Scene::Reunion => &[("frame-drum", 0), ("tambourine", 6)],
            Scene::Dawn => &[("tambourine", 4)],
            _ => &[],
        },
        AdventureStyle::Orchestral => match scene {
            Scene::Camp if phrase >= 2 => &[("frame-drum", 0), ("tambourine", 6)],
            Scene::Explore => &[("frame-drum", 0), ("tambourine", 4)],
            Scene::Town | Scene::Victory => &[
                ("frame-drum", 0),
                ("tambourine", 2),
                ("frame-drum", 4),
                ("tambourine", 6),
            ],
            Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => &[
                ("frame-drum", 0),
                ("tambourine", 2),
                ("frame-drum", 3),
                ("frame-drum", 4),
                ("tambourine", 5),
                ("frame-drum", 6),
                ("tambourine", 7),
            ],
            Scene::Festival | Scene::Reunion => &[
                ("frame-drum", 0),
                ("tambourine", 2),
                ("frame-drum", 4),
                ("tambourine", 6),
            ],
            Scene::Dawn => &[("tambourine", 4)],
            Scene::Sanctuary if phrase == 2 => &[("tambourine", 4)],
            _ => &[],
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn add_percussion(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    phrase: u32,
    local_bar: u32,
    bar: u32,
    bar_ticks: u32,
    pulse: u32,
) {
    for &(voice, onset) in plan
        .performance
        .percussion(style, plan.scene, phrase, local_bar, traits.motion)
    {
        let is_frame = voice == "frame-drum";
        push_percussion(
            events,
            plan.id,
            &mut counters.percussion,
            bar * bar_ticks + u32::from(onset) * pulse,
            if is_frame { pulse } else { pulse / 2 },
            0.2 + scene_energy(plan.scene) * 0.15
                + traits.danger * 0.08
                + if is_frame { 0.035 } else { 0.0 },
            voice,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn add_bell_accent(
    events: &mut Vec<MusicEvent>,
    counters: &mut EventCounters,
    plan: &SectionPlan,
    style: AdventureStyle,
    dna: &PieceDna,
    intervals: &[i32],
    kind: PhraseKind,
    phrase: u32,
    bar: u32,
    local_bar: u32,
    bar_ticks: u32,
) {
    let accent = match style {
        AdventureStyle::Folk => {
            local_bar == 3
                && matches!(plan.scene, Scene::Sanctuary | Scene::Victory)
                && phrase % 2 == 1
        }
        AdventureStyle::Dark => {
            local_bar == 3
                && matches!(plan.scene, Scene::Dungeon | Scene::Boss)
                && phrase.is_multiple_of(2)
        }
        AdventureStyle::Orchestral => {
            local_bar == 3
                && (matches!(kind, PhraseKind::Development | PhraseKind::Cadence)
                    || plan.scene == Scene::Victory)
        }
    };
    if !accent {
        return;
    }
    let degree = if matches!(kind, PhraseKind::Cadence) {
        4
    } else {
        2
    };
    let pitch = nearest_scale_pitch(dna.tonic_pitch_class, degree, intervals, 78, 67, 84);
    push_note(
        events,
        plan.id,
        "bell",
        &mut counters.bell,
        bar * bar_ticks,
        bar_ticks / 2,
        0.15 + scene_energy(plan.scene) * 0.05,
        pitch,
        "bell",
        false,
    );
}

fn build_section(
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    ticks_per_beat: u32,
    section_seed: u32,
) -> PortableSection {
    let bar_ticks = ticks_per_beat * BEATS_PER_BAR;
    let pulse = ticks_per_beat / 2;
    let phrase_count = plan.bars / PHRASE_BARS;
    let mode = mode_for(style, plan.scene);
    let intervals = mode_intervals(mode);
    let mut rng = DeterministicRandom::new(section_seed);
    let section_variant = dna.rhythm_variant + rng.integer(8) as usize;
    let mut events = Vec::new();
    let mut counters = EventCounters::new();
    let mut previous_chord = None;
    let mut previous_melody = None;

    for phrase in 0..phrase_count {
        let kind = phrase_kind(phrase, phrase_count);
        let chords = progression(mode, kind, plan.scene, phrase, dna.harmony_variant);
        add_pedal(
            &mut events,
            &mut counters,
            plan,
            style,
            dna,
            &intervals,
            phrase,
            bar_ticks,
        );
        for local_bar in 0..PHRASE_BARS {
            let bar = phrase * PHRASE_BARS + local_bar;
            let chord_degree = chords[local_bar as usize];
            let chord = voice_chord(
                dna.tonic_pitch_class,
                chord_degree,
                &intervals,
                previous_chord,
            );
            previous_chord = Some(chord);
            add_bass(
                &mut events,
                &mut counters,
                plan,
                style,
                traits,
                dna,
                &intervals,
                chord_degree,
                bar,
                bar_ticks,
                pulse,
            );
            add_harmony(
                &mut events,
                &mut counters,
                plan,
                style,
                traits,
                kind,
                chord,
                bar,
                local_bar,
                bar_ticks,
            );
            add_harp(
                &mut events,
                &mut counters,
                plan,
                style,
                traits,
                dna,
                &intervals,
                kind,
                chord_degree,
                bar,
                bar_ticks,
                pulse,
                section_variant + phrase as usize,
            );
            add_melody(
                &mut events,
                &mut counters,
                plan,
                style,
                traits,
                dna,
                &intervals,
                mode,
                kind,
                phrase,
                chord_degree,
                bar,
                local_bar,
                bar_ticks,
                pulse,
                section_variant + phrase as usize,
                &mut previous_melody,
            );
            add_recorder_answer(
                &mut events,
                &mut counters,
                plan,
                style,
                dna,
                &intervals,
                chord_degree,
                bar,
                local_bar,
                bar_ticks,
                pulse,
                section_variant + phrase as usize,
            );
            add_percussion(
                &mut events,
                &mut counters,
                plan,
                style,
                traits,
                phrase,
                local_bar,
                bar,
                bar_ticks,
                pulse,
            );
            add_bell_accent(
                &mut events,
                &mut counters,
                plan,
                style,
                dna,
                &intervals,
                kind,
                phrase,
                bar,
                local_bar,
                bar_ticks,
            );
        }
    }

    fn event_id(event: &MusicEvent) -> &str {
        match event {
            MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
        }
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
        length_ticks: plan.bars * bar_ticks,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::{adventure_phase_energy, adventure_phase_role, AdventurePhaseRole, Scene};

    #[test]
    fn metadata_derives_role_and_energy_from_scene() {
        for (scene, role, energy) in [
            (Scene::Camp, AdventurePhaseRole::Intro, 25),
            (Scene::Explore, AdventurePhaseRole::Groove, 48),
            (Scene::Town, AdventurePhaseRole::Groove, 58),
            (Scene::Dungeon, AdventurePhaseRole::Build, 20),
            (Scene::Combat, AdventurePhaseRole::Peak, 82),
            (Scene::Boss, AdventurePhaseRole::Peak, 90),
            (Scene::Sanctuary, AdventurePhaseRole::Break, 34),
            (Scene::Victory, AdventurePhaseRole::Outro, 70),
            (Scene::Skirmish, AdventurePhaseRole::Peak, 76),
            (Scene::Assault, AdventurePhaseRole::Peak, 87),
            (Scene::Chase, AdventurePhaseRole::Peak, 80),
            (Scene::Festival, AdventurePhaseRole::Groove, 62),
            (Scene::Reunion, AdventurePhaseRole::Groove, 54),
            (Scene::Dawn, AdventurePhaseRole::Break, 36),
        ] {
            assert_eq!(adventure_phase_role(scene), role, "role for {scene:?}");
            assert_eq!(adventure_phase_energy(scene), energy, "energy for {scene:?}");
        }
    }

    #[test]
    fn metadata_is_total_and_stays_in_the_energy_band() {
        for scene in [
            Scene::Camp,
            Scene::Explore,
            Scene::Town,
            Scene::Dungeon,
            Scene::Combat,
            Scene::Boss,
            Scene::Sanctuary,
            Scene::Victory,
            Scene::Skirmish,
            Scene::Assault,
            Scene::Chase,
            Scene::Festival,
            Scene::Reunion,
            Scene::Dawn,
        ] {
            let energy = adventure_phase_energy(scene);
            assert!(energy <= 100, "{scene:?} energy {energy}");
        }
        // The quest arc is framed by a single intro and a single outro.
        assert_eq!(adventure_phase_role(Scene::Camp), AdventurePhaseRole::Intro);
        assert_eq!(
            adventure_phase_role(Scene::Victory),
            AdventurePhaseRole::Outro
        );
    }
}
