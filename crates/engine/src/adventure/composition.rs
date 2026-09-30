use std::collections::BTreeMap;

use crate::rng::{keyed_unit, DeterministicRandom};
use crate::score::{MusicEvent, PortableSection};
use crate::theory::{mode_intervals, phrase_gain, scale_pitch};

use super::harmony::{
    chord_at, dominant_degree, phrase_harmony, place_degree, voice_chord, Chord, ChordKind,
    ChordSpan, PhraseHarmonyInput,
};
use super::theme::{counter_line, phrase_melody, PhraseMelodyInput, Treatment, THEME_RHYTHMS};
use super::{AdventureStyle, NormalizedTraits};
use crate::event_sink::EventSink;
use crate::melody::{compose_theme, metric_accent, Tone};

const BEATS_PER_BAR: u32 = 4;
const PHRASE_BARS: u32 = 4;
const MELODY_MIN: i32 = 55;
const MELODY_MAX: i32 = 84;

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
}

pub(super) const SECTION_PLANS: &[SectionPlan; 14] = &[
    SectionPlan {
        id: "camp",
        label: "Trailhead Camp",
        feeling: "hearthlight / the road ahead",
        color: "#d5aa64",
        bars: 16,
        scene: Scene::Camp,
    },
    SectionPlan {
        id: "explore",
        label: "The Old Forest",
        feeling: "open paths / old wonders",
        color: "#78976a",
        bars: 32,
        scene: Scene::Explore,
    },
    SectionPlan {
        id: "town",
        label: "Hearth and Hall",
        feeling: "market dance / crowded tables",
        color: "#d59a42",
        bars: 32,
        scene: Scene::Town,
    },
    SectionPlan {
        id: "festival",
        label: "The Green Market",
        feeling: "dancing feet / raised cups",
        color: "#d9a441",
        bars: 32,
        scene: Scene::Festival,
    },
    SectionPlan {
        id: "reunion",
        label: "Homecoming Hearth",
        feeling: "warm embraces / old names",
        color: "#d59a66",
        bars: 32,
        scene: Scene::Reunion,
    },
    SectionPlan {
        id: "dungeon",
        label: "The Deep Halls",
        feeling: "cold stone / distant steps",
        color: "#59616c",
        bars: 16,
        scene: Scene::Dungeon,
    },
    SectionPlan {
        id: "skirmish",
        label: "Steel in the Brush",
        feeling: "blades flash / first blood",
        color: "#b5543f",
        bars: 16,
        scene: Scene::Skirmish,
    },
    SectionPlan {
        id: "combat",
        label: "Steel and Shadow",
        feeling: "measured pursuit / battle joined",
        color: "#ad5540",
        bars: 32,
        scene: Scene::Combat,
    },
    SectionPlan {
        id: "chase",
        label: "Pursuit",
        feeling: "hearts pound / ground blurs",
        color: "#a8602e",
        bars: 32,
        scene: Scene::Chase,
    },
    SectionPlan {
        id: "boss",
        label: "No Retreat",
        feeling: "ancient dread / final challenge",
        color: "#713343",
        bars: 16,
        scene: Scene::Boss,
    },
    SectionPlan {
        id: "assault",
        label: "The Red Charge",
        feeling: "full charge / no quarter",
        color: "#8f2f34",
        bars: 16,
        scene: Scene::Assault,
    },
    SectionPlan {
        id: "sanctuary",
        label: "The Hidden Glade",
        feeling: "clear water / shelter found",
        color: "#9bc78b",
        bars: 16,
        scene: Scene::Sanctuary,
    },
    SectionPlan {
        id: "dawn",
        label: "First Light",
        feeling: "soft gold / the long night breaks",
        color: "#cfe3a0",
        bars: 16,
        scene: Scene::Dawn,
    },
    SectionPlan {
        id: "victory",
        label: "Lanterns at Dawn",
        feeling: "homecoming / earned release",
        color: "#ead27e",
        bars: 32,
        scene: Scene::Victory,
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

/// What one seed fixes for the whole piece: its key and its quest theme.
pub(super) struct PieceDna {
    pub(super) tonic_pitch_class: i32,
    pub(super) theme: Vec<Tone>,
    /// Which member of each harmonic template family the statements use.
    pub(super) statement_variant: usize,
}

impl PieceDna {
    pub(super) fn new(seed: u32) -> Self {
        let mut rng = DeterministicRandom::new(seed);
        // The tonic stays the seed's first draw, so a seed keeps its key.
        let tonic_pitch_class = *rng.pick(&TONIC_PITCH_CLASSES);
        let theme = compose_theme(&mut rng, &THEME_RHYTHMS);
        Self {
            tonic_pitch_class,
            theme,
            statement_variant: rng.integer(3) as usize,
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

/// How fast a scene moves: its harmonic rhythm and accompaniment drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pace {
    Calm,
    Walking,
    Driving,
}

fn pace(scene: Scene) -> Pace {
    match scene {
        Scene::Camp | Scene::Dungeon | Scene::Sanctuary | Scene::Dawn => Pace::Calm,
        Scene::Explore | Scene::Town | Scene::Reunion | Scene::Victory | Scene::Festival => {
            Pace::Walking
        }
        Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => {
            Pace::Driving
        }
    }
}

/// The rhythmic lilt of a scene. Written rhythms are straight sixteenths; a
/// lilt or jig moves each beat's off-beat later, so the same figures dance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Feel {
    Straight,
    /// A hornpipe lean: the off-beat lands at 60% of the beat.
    Lilt,
    /// Compound time inside four beats: the off-beat is the third triplet.
    Jig,
}

impl Feel {
    /// Where a beat's written midpoint is heard, as a fraction of the beat.
    fn swing_point(self) -> f64 {
        match self {
            Self::Straight => 0.5,
            Self::Lilt => 0.6,
            Self::Jig => 2.0 / 3.0,
        }
    }
}

fn feel(style: AdventureStyle, scene: Scene) -> Feel {
    match (style, scene) {
        (AdventureStyle::Folk, Scene::Festival | Scene::Victory) => Feel::Jig,
        (AdventureStyle::Folk, Scene::Town | Scene::Reunion) => Feel::Lilt,
        (AdventureStyle::Orchestral, Scene::Festival) => Feel::Jig,
        (AdventureStyle::Dark, Scene::Festival) => Feel::Lilt,
        _ => Feel::Straight,
    }
}

fn treatment(style: AdventureStyle, scene: Scene) -> Treatment {
    match scene {
        Scene::Camp if style == AdventureStyle::Orchestral => Treatment::Augmented,
        Scene::Reunion if style == AdventureStyle::Orchestral => Treatment::Augmented,
        Scene::Sanctuary | Scene::Dawn => Treatment::Augmented,
        Scene::Dungeon => Treatment::Fragment,
        Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Assault | Scene::Chase => {
            Treatment::Diminution
        }
        _ => Treatment::Statement,
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

/// Everything fixed for one section while its lanes are written.
struct Section<'a> {
    plan: &'a SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    tonic: i32,
    mode: &'static str,
    intervals: Vec<i32>,
    pace: Pace,
    bar: u32,
    sixteenth: u32,
    seed: u32,
    energy: f64,
    /// Semitone offset that places the section's melody degrees in register.
    melody_octave: i32,
}

impl Section<'_> {
    fn scene(&self) -> Scene {
        self.plan.scene
    }

    /// A melody-space degree as a pitch, before any register folding.
    fn melody_pitch(&self, degree: i32) -> i32 {
        scale_pitch(60 + self.tonic, degree, &self.intervals) + self.melody_octave
    }

    fn place(&self, degree: i32, target: i32, minimum: i32, maximum: i32) -> i32 {
        place_degree(
            self.tonic,
            degree,
            &self.intervals,
            target,
            minimum,
            maximum,
        )
    }

    fn chance(&self, tag: &str, a: u32, b: u32) -> f64 {
        keyed_unit(self.seed, tag, a, b)
    }

    fn semitone_rub(&self, upper: i32, lower: i32) -> bool {
        matches!(
            (self.melody_pitch(upper) - self.melody_pitch(lower)).rem_euclid(12),
            1 | 11
        )
    }
}

/// One phrase's material, shared by every lane that follows it.
struct PhrasePlan {
    index: u32,
    kind: PhraseKind,
    start: u32,
    chords: Vec<ChordSpan>,
    melody: Vec<Tone>,
    /// Folk and Dark camps open with the theme on the harp alone.
    harp_intro: bool,
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

fn warm(scene: Scene) -> bool {
    matches!(
        scene,
        Scene::Camp
            | Scene::Explore
            | Scene::Town
            | Scene::Sanctuary
            | Scene::Victory
            | Scene::Festival
            | Scene::Reunion
            | Scene::Dawn
    )
}

fn build_section(
    plan: &SectionPlan,
    style: AdventureStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    ticks_per_beat: u32,
    section_seed: u32,
) -> PortableSection {
    let bar = ticks_per_beat * BEATS_PER_BAR;
    let mode = mode_for(style, plan.scene);
    let intervals = mode_intervals(mode);
    let mut section = Section {
        plan,
        style,
        traits,
        tonic: dna.tonic_pitch_class,
        mode,
        intervals,
        pace: pace(plan.scene),
        bar,
        sixteenth: ticks_per_beat / 4,
        seed: section_seed,
        energy: scene_energy(plan.scene),
        melody_octave: 0,
    };
    let phrases = plan_phrases(&section, dna);
    section.melody_octave = melody_octave(&section, &phrases);

    let mut sink = EventSink::new(
        plan.id,
        ticks_per_beat,
        feel(style, plan.scene).swing_point(),
        plan.bars * bar,
    );
    let chords: Vec<ChordSpan> = phrases
        .iter()
        .flat_map(|p| p.chords.iter().copied())
        .collect();
    for phrase in &phrases {
        add_melody(&mut sink, &section, phrase);
        add_counter(&mut sink, &section, phrase);
        add_horn(&mut sink, &section, phrase);
        add_bell(&mut sink, &section, phrase);
        add_pedal(&mut sink, &section, phrase);
    }
    add_harmony(&mut sink, &section, &chords);
    add_bass(&mut sink, &section, &chords);
    add_harp(&mut sink, &section, &phrases);
    add_ostinato(&mut sink, &section, &chords);
    add_percussion(&mut sink, &section, &phrases);
    add_timpani(&mut sink, &section, &phrases, &chords);
    damp_harp(&mut sink.events, &chords, section.sixteenth, bar);

    PortableSection {
        id: plan.id.to_string(),
        label: plan.label.to_string(),
        feeling: plan.feeling.to_string(),
        color: plan.color.to_string(),
        length_ticks: plan.bars * bar,
        events: sink.finish(),
    }
}

fn plan_phrases(section: &Section, dna: &PieceDna) -> Vec<PhrasePlan> {
    let count = section.plan.bars / PHRASE_BARS;
    let scene_treatment = treatment(section.style, section.scene());
    let dominant = dominant_degree(section.mode);
    (0..count)
        .map(|index| {
            let kind = phrase_kind(index, count);
            let start = index * PHRASE_BARS * section.bar;
            let chords = phrase_harmony(&PhraseHarmonyInput {
                mode: section.mode,
                intervals: &section.intervals,
                kind,
                pace: section.pace,
                phrase_start: start,
                bar_ticks: section.bar,
                statement_variant: dna.statement_variant,
                seed: section.seed,
                phrase: index,
                danger: section.traits.danger,
                mystery: section.traits.mystery,
                open_color: section.style == AdventureStyle::Dark,
                suspends: section.style != AdventureStyle::Folk,
            });
            let melody = phrase_melody(&PhraseMelodyInput {
                theme: &dna.theme,
                intervals: &section.intervals,
                kind,
                treatment: scene_treatment,
                dominant,
                chords: &chords,
                phrase_start: start,
                sixteenth: section.sixteenth,
                seed: section.seed,
                phrase: index,
                motion: section.traits.motion,
                mystery: section.traits.mystery,
                pickup: index + 1 < count,
            });
            PhrasePlan {
                index,
                kind,
                start,
                chords,
                melody,
                harp_intro: index == 0
                    && section.scene() == Scene::Camp
                    && section.style != AdventureStyle::Orchestral,
            }
        })
        .collect()
}

/// One octave placement for the whole section, so phrases join without
/// register jumps: the mean melody pitch sits nearest the scene's centre.
fn melody_octave(section: &Section, phrases: &[PhrasePlan]) -> i32 {
    let degrees: Vec<i32> = phrases
        .iter()
        .flat_map(|phrase| phrase.melody.iter().map(|tone| tone.degree))
        .collect();
    let center = melody_center(section.style, section.scene())
        + (section.traits.wonder * 2.0).round() as i32;
    let base = |degree: i32| scale_pitch(60 + section.tonic, degree, &section.intervals);
    let mean =
        degrees.iter().map(|degree| base(*degree)).sum::<i32>() / degrees.len().max(1) as i32;
    [-24, -12, 0, 12]
        .into_iter()
        .min_by_key(|shift| {
            let low = degrees.iter().map(|d| base(*d) + shift).min().unwrap_or(0);
            let high = degrees.iter().map(|d| base(*d) + shift).max().unwrap_or(0);
            let outside = (MELODY_MIN - low).max(0) + (high - MELODY_MAX).max(0);
            (outside, (mean + shift - center).abs())
        })
        .unwrap()
}

fn fold_into(pitch: i32, minimum: i32, maximum: i32) -> i32 {
    let mut pitch = pitch;
    while pitch > maximum {
        pitch -= 12;
    }
    while pitch < minimum {
        pitch += 12;
    }
    pitch
}

fn articulation(style: AdventureStyle, scene: Scene) -> f64 {
    match style {
        AdventureStyle::Folk if matches!(scene, Scene::Town | Scene::Festival) => 0.62,
        AdventureStyle::Folk => 0.78,
        AdventureStyle::Dark => 0.86,
        AdventureStyle::Orchestral if pace(scene) == Pace::Driving => 0.72,
        AdventureStyle::Orchestral => 0.94,
    }
}

fn add_melody(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let voice = melody_voice(section.style, section.scene());
    let bar_arch = [0.9, 1.0, 1.08, 0.95];
    let base = 0.3 + section.traits.wonder * 0.06 + section.energy * 0.1;
    let ornaments = section.style != AdventureStyle::Orchestral;
    let final_bar = section.plan.bars * section.bar - section.bar;
    let mut previous_end = 0;
    for (index, tone) in phrase.melody.iter().enumerate() {
        let start = phrase.start + tone.at * section.sixteenth;
        let pitch = fold_into(section.melody_pitch(tone.degree), MELODY_MIN, MELODY_MAX);
        if phrase.harp_intro && tone.at < 32 {
            sink.note(
                "harp-theme",
                start,
                tone.length * section.sixteenth,
                0.2 + section.traits.wonder * 0.08,
                fold_into(pitch, 48, 76),
                "harp",
                false,
            );
            continue;
        }
        let length = ((f64::from(tone.length * section.sixteenth)
            * articulation(section.style, section.scene())) as u32)
            .max(section.sixteenth / 2);
        let long = if tone.length >= 8 { 0.03 } else { 0.0 };
        let velocity = (base + long)
            * metric_accent(tone.at)
            * bar_arch[(tone.at / 16).min(3) as usize]
            * texture_gain(phrase.kind, phrase.index);
        let grace = section.sixteenth / 2;
        let ornament = ornaments
            && tone.length >= 6
            && tone.at > 0
            && start < final_bar
            && start >= previous_end + grace
            && section.chance("ornament", phrase.index, index as u32)
                < 0.12 + section.traits.motion * 0.3;
        if ornament {
            let upper = fold_into(
                section.melody_pitch(tone.degree + 1),
                MELODY_MIN,
                MELODY_MAX,
            );
            sink.note(
                "melody",
                start - grace,
                grace,
                velocity * 0.75,
                upper,
                voice,
                true,
            );
        }
        sink.note("melody", start, length, velocity, pitch, voice, true);
        previous_end = start + length;
    }
}

fn counter_voice(style: AdventureStyle) -> &'static str {
    match style {
        AdventureStyle::Folk | AdventureStyle::Orchestral => "recorder",
        AdventureStyle::Dark => "vielle",
    }
}

/// A second voice joins after the first statement. Orchestral always answers
/// in its warm scenes; elsewhere the line appears more often as wonder climbs.
fn add_counter(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    if matches!(phrase.kind, PhraseKind::Antecedent) || phrase.harp_intro {
        return;
    }
    let always = section.style == AdventureStyle::Orchestral && warm(section.scene());
    let likely = section.chance("counter", phrase.index, 0) < 0.2 + section.traits.wonder * 0.7;
    if !(always || likely) {
        return;
    }
    let line = counter_line(
        &phrase.melody,
        &phrase.chords,
        phrase.start,
        section.sixteenth,
        |upper, lower| section.semitone_rub(upper, lower),
    );
    let velocity = (0.19 + section.traits.wonder * 0.04 + section.energy * 0.06)
        * texture_gain(phrase.kind, phrase.index);
    for tone in line {
        let pitch = section.melody_pitch(tone.degree);
        if !(50..=MELODY_MAX).contains(&pitch) {
            continue;
        }
        sink.note(
            "counter",
            phrase.start + tone.at * section.sixteenth,
            tone.length * section.sixteenth * 9 / 10,
            velocity * metric_accent(tone.at),
            pitch,
            counter_voice(section.style),
            false,
        );
    }
}

/// Orchestral horns: they double the theme an octave down when it returns in
/// the heroic scenes, carry it in every phrase of the boss fights, and hold a
/// soft root-and-fifth in the quiet scenes.
fn add_horn(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    if section.style != AdventureStyle::Orchestral {
        return;
    }
    let scene = section.scene();
    let heroic = matches!(
        scene,
        Scene::Victory | Scene::Festival | Scene::Combat | Scene::Explore | Scene::Dawn
    );
    let fight = matches!(scene, Scene::Boss | Scene::Assault);
    let returning = matches!(phrase.kind, PhraseKind::Consequent | PhraseKind::Return);
    if fight || (heroic && returning) {
        for tone in phrase.melody.iter().filter(|tone| tone.length >= 4) {
            let pitch = section.melody_pitch(tone.degree) - 12;
            if !(45..=67).contains(&pitch) {
                continue;
            }
            sink.note(
                "horn",
                phrase.start + tone.at * section.sixteenth,
                tone.length * section.sixteenth * 9 / 10,
                (0.22 + section.energy * 0.12) * metric_accent(tone.at),
                pitch,
                "horn",
                false,
            );
        }
    } else if matches!(scene, Scene::Camp | Scene::Sanctuary | Scene::Reunion) && returning {
        for span in &phrase.chords {
            for (offset, target) in [(0, 45), (4, 52)] {
                let pitch = section.place(span.chord.degree + offset, target, 41, 60);
                sink.note(
                    "horn",
                    span.start,
                    span.length * 15 / 16,
                    0.11 + section.traits.wonder * 0.03,
                    pitch,
                    "horn",
                    false,
                );
            }
        }
    }
}

/// Bells mark arrivals: the authored landmarks per style, a sparkle that grows
/// with wonder, and a stranger glint that grows with mystery.
fn add_bell(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let scene = section.scene();
    let authored = match section.style {
        AdventureStyle::Folk => {
            matches!(scene, Scene::Sanctuary | Scene::Victory) && phrase.index % 2 == 1
        }
        AdventureStyle::Dark => {
            matches!(scene, Scene::Dungeon | Scene::Boss) && phrase.index.is_multiple_of(2)
        }
        AdventureStyle::Orchestral => {
            matches!(phrase.kind, PhraseKind::Development | PhraseKind::Cadence)
                || scene == Scene::Victory
        }
    };
    let wonder = section.chance("bell-wonder", phrase.index, 0) < section.traits.wonder * 0.35;
    let arrival = phrase.start + 3 * section.bar;
    let velocity = 0.15 + section.energy * 0.05;
    if authored || wonder {
        let degree = chord_at(&phrase.chords, arrival).map_or(0, |chord| chord.degree + 2);
        let pitch = section.place(degree, 78, 67, 84);
        sink.note(
            "bell",
            arrival,
            section.bar / 2,
            velocity,
            pitch,
            "bell",
            false,
        );
    }
    if section.chance("bell-mystery", phrase.index, 0) < section.traits.mystery * 0.45 {
        let at = phrase.start + section.bar + section.bar / 2;
        let degree = chord_at(&phrase.chords, at).map_or(4, |chord| chord.degree + 4);
        let pitch = section.place(degree, 76, 67, 84);
        sink.note(
            "bell",
            at,
            section.bar / 2,
            velocity * 0.8,
            pitch,
            "bell",
            false,
        );
    }
}

/// Dark's bowed drone: the tonic held under every run of chords that contain
/// it (up to two bars), with the open fifth above only when every chord in
/// the run contains that too.
fn add_drone(
    sink: &mut EventSink,
    section: &Section,
    phrase: &PhrasePlan,
    tonic: i32,
    velocity: f64,
) {
    let fifth = section.place(4, tonic + 7, tonic + 5, tonic + 9);
    let mut run: Vec<ChordSpan> = Vec::new();
    let flush = |sink: &mut EventSink, run: &mut Vec<ChordSpan>| {
        let (Some(first), Some(last)) = (run.first(), run.last()) else {
            return;
        };
        let (start, end) = (first.start, last.start + last.length);
        sink.note(
            "pedal",
            start,
            end - start,
            velocity,
            tonic,
            "vielle",
            false,
        );
        if run.iter().all(|span| span.chord.contains(4)) {
            sink.note(
                "pedal",
                start,
                end - start,
                velocity,
                fifth,
                "vielle",
                false,
            );
        }
        run.clear();
    };
    for span in &phrase.chords {
        let holds = span.chord.contains(0) && !span.chord.rubs(0, &section.intervals);
        let fits_run = run
            .first()
            .is_some_and(|first| span.start + span.length - first.start <= 2 * section.bar);
        if !holds || !fits_run {
            flush(sink, &mut run);
        }
        if holds {
            run.push(*span);
        }
    }
    flush(sink, &mut run);
}

/// The bed. Dark keeps a bowed open-fifth drone under every chord that
/// agrees with it; Folk plucks a harp bourdon and Orchestral holds low
/// strings at the authored landmarks, and both lean on it more with mystery.
fn add_pedal(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let scene = section.scene();
    let velocity = 0.09 + section.energy * 0.05;
    let tonic = section.place(0, 43, 36, 50);
    if section.style == AdventureStyle::Dark {
        // The battle ostinato drives the fights; the drone holds everywhere else.
        if section.pace != Pace::Driving {
            add_drone(sink, section, phrase, tonic, velocity);
        }
        return;
    }
    let authored = match (section.style, scene) {
        (_, Scene::Dungeon) => phrase.index == 0 || phrase.index == 2,
        (AdventureStyle::Folk, Scene::Camp | Scene::Sanctuary | Scene::Dawn) => phrase.index == 0,
        (AdventureStyle::Orchestral, Scene::Sanctuary) => phrase.index == 2,
        (AdventureStyle::Orchestral, Scene::Boss) => true,
        _ => false,
    };
    let mysterious = section.pace != Pace::Driving
        && section.chance("pedal", phrase.index, 0) < section.traits.mystery * 0.5;
    if !(authored || mysterious) {
        return;
    }
    let voice = match section.style {
        AdventureStyle::Folk => "harp",
        _ => "vielle",
    };
    // The pedal holds for up to two bars, and only while the harmony keeps
    // the tonic.
    let end = phrase
        .chords
        .iter()
        .take_while(|span| span.chord.contains(0) && span.start < phrase.start + 2 * section.bar)
        .map(|span| (span.start + span.length).min(phrase.start + 2 * section.bar))
        .last();
    if let Some(end) = end {
        sink.note(
            "pedal",
            phrase.start,
            end - phrase.start,
            velocity,
            tonic,
            voice,
            false,
        );
    }
}

fn pedal_sounds(sink: &EventSink, tick: u32) -> bool {
    sink.events.iter().any(|event| {
        matches!(event, MusicEvent::Note { lane, start_tick, duration_ticks, .. }
            if lane == "pedal" && *start_tick <= tick && tick < start_tick + duration_ticks)
    })
}

fn add_harmony(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    let mut previous = None;
    for span in chords {
        let voicing = voice_chord(section.tonic, span.chord, &section.intervals, previous);
        previous = Some(voicing);
        let bar_index = span.start / section.bar;
        let velocity =
            (0.13 + section.traits.wonder * 0.06 + section.energy * 0.05) * phrase_gain(bar_index);
        match section.style {
            AdventureStyle::Folk => {
                // A rolled harp chord, lowest string first.
                let roll = section.sixteenth / 6;
                for (index, pitch) in voicing.iter().enumerate() {
                    sink.note(
                        "harmony",
                        span.start + index as u32 * roll,
                        span.length.min(section.bar / 2),
                        velocity * (1.0 - index as f64 * 0.06),
                        *pitch,
                        "harp",
                        false,
                    );
                }
            }
            AdventureStyle::Dark => {
                let voices: &[i32] = if span.chord.kind == ChordKind::Open5 {
                    &voicing[..2]
                } else {
                    &voicing[1..]
                };
                for pitch in voices {
                    sink.note(
                        "harmony",
                        span.start,
                        span.length * 7 / 8,
                        velocity,
                        *pitch,
                        "vielle",
                        false,
                    );
                }
            }
            AdventureStyle::Orchestral => {
                for pitch in voicing {
                    sink.note(
                        "harmony",
                        span.start,
                        span.length * 15 / 16,
                        velocity * 0.8,
                        pitch,
                        "vielle",
                        false,
                    );
                }
            }
        }
    }
}

/// How a harpist phrases the written notes: chord tones ring until the
/// harmony changes, and a moving line (the bass, a harp melody) is damped
/// when its next note sounds.
fn damp_harp(events: &mut [MusicEvent], chords: &[ChordSpan], sixteenth: u32, bar: u32) {
    let line = |lane: &str| matches!(lane, "bass" | "harp-theme");
    let onsets: BTreeMap<String, Vec<u32>> =
        events.iter().fold(BTreeMap::new(), |mut onsets, event| {
            if let MusicEvent::Note {
                lane,
                voice,
                start_tick,
                ..
            } = event
            {
                if voice == "harp" && line(lane) {
                    onsets
                        .entry(lane.clone())
                        .or_insert_with(Vec::new)
                        .push(*start_tick);
                }
            }
            onsets
        });
    for event in events.iter_mut() {
        let MusicEvent::Note {
            lane,
            voice,
            start_tick,
            duration_ticks,
            ..
        } = event
        else {
            continue;
        };
        if voice != "harp" {
            continue;
        }
        let start = *start_tick;
        let chord_end = chords
            .iter()
            .find(|span| (span.start..span.start + span.length).contains(&start))
            .map_or(start + bar, |span| span.start + span.length);
        let next = onsets
            .get(lane.as_str())
            .and_then(|starts| {
                starts
                    .iter()
                    .copied()
                    .filter(|onset| *onset >= start + sixteenth / 2)
                    .min()
            })
            .unwrap_or(u32::MAX);
        let end = if line(lane) {
            chord_end.min(next)
        } else {
            chord_end
        };
        *duration_ticks = (end - start).clamp(sixteenth / 2, bar);
    }
}

/// Bass figures in sixteenths from the chord's start: `(position, role)` where
/// role 0 is the root, 1 the fifth and 2 the octave.
fn bass_figure(style: AdventureStyle, pace: Pace) -> &'static [(u32, u8)] {
    match (style, pace) {
        (_, Pace::Calm) => &[(0, 0)],
        (_, Pace::Walking) => &[(0, 0), (8, 1)],
        (AdventureStyle::Folk, Pace::Driving) => &[(0, 0), (4, 1), (8, 0), (12, 1)],
        (AdventureStyle::Dark, Pace::Driving) => &[(0, 0), (6, 0), (12, 1)],
        (AdventureStyle::Orchestral, Pace::Driving) => &[
            (0, 0),
            (2, 0),
            (4, 2),
            (6, 0),
            (8, 0),
            (10, 0),
            (12, 2),
            (14, 1),
        ],
    }
}

fn add_bass(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    let voice = match section.style {
        AdventureStyle::Folk | AdventureStyle::Dark => "harp",
        AdventureStyle::Orchestral => "vielle",
    };
    let final_bar = section.plan.bars * section.bar - section.bar;
    let root_of = |chord: Chord| section.place(chord.degree, 42, 36, 55);
    for (index, span) in chords.iter().enumerate() {
        let root = root_of(span.chord);
        let velocity = 0.22 + section.energy * 0.13 + section.traits.danger * 0.04;
        if span.start >= final_bar {
            sink.note(
                "bass",
                span.start,
                section.bar * 3 / 4,
                velocity,
                root,
                voice,
                false,
            );
            continue;
        }
        // Outside battle the bass rests while a pedal or drone carries the root.
        if section.pace != Pace::Driving && pedal_sounds(sink, span.start) {
            continue;
        }
        let bar_index = span.start / section.bar;
        // Danger quickens a walking bass into the driving figure bar by bar.
        let pace = if section.pace == Pace::Walking
            && section.chance("bass-drive", bar_index, 0) < section.traits.danger * 0.6
        {
            Pace::Driving
        } else {
            section.pace
        };
        let span_sixteenths = span.length / section.sixteenth;
        let folk_calm_rest = section.style == AdventureStyle::Folk
            && pace == Pace::Calm
            && bar_index % 2 == 1
            && span.length <= section.bar;
        if folk_calm_rest {
            continue;
        }
        for &(position, role) in bass_figure(section.style, pace) {
            if position >= span_sixteenths {
                continue;
            }
            let pitch = match role {
                0 => root,
                1 => section.place(span.chord.degree + 4, root + 7, root + 5, root + 9),
                _ => fold_into(root + 12, 36, 57),
            };
            let nominal = if pace == Pace::Calm {
                span.length.min(section.bar) * 3 / 4
            } else {
                section.sixteenth * 3
            };
            // Every bass note stops at its chord's end, never under the next.
            let length = nominal.min(span.length - position * section.sixteenth);
            sink.note(
                "bass",
                span.start + position * section.sixteenth,
                length,
                velocity * metric_accent(position),
                pitch,
                voice,
                false,
            );
        }
        // A stepwise approach note walks into a distant next root.
        if let Some(next) = chords.get(index + 1) {
            let leap = (next.chord.degree - span.chord.degree).rem_euclid(7);
            let approach = [next.chord.degree - 1, next.chord.degree + 1]
                .into_iter()
                .find(|degree| !span.chord.rubs(*degree, &section.intervals));
            if let Some(approach) = approach
                .filter(|_| pace != Pace::Calm && matches!(leap, 2..=5) && span_sixteenths >= 8)
            {
                let pitch = section.place(approach, root_of(next.chord), 36, 55);
                sink.note(
                    "bass",
                    next.start - 2 * section.sixteenth,
                    section.sixteenth * 2,
                    velocity * 0.82,
                    pitch,
                    voice,
                    false,
                );
            }
        }
    }
}

/// Harp figuration in sixteenths: `(position, chord tone)` where the tone
/// index walks root, third, fifth, octave, tenth.
fn harp_figure(style: AdventureStyle, scene: Scene, variant: u32) -> &'static [(u32, u8)] {
    let jig = feel(style, scene) == Feel::Jig;
    match (style, pace(scene)) {
        (AdventureStyle::Folk, _) if jig => &[
            (1, 1),
            (2, 2),
            (5, 1),
            (6, 2),
            (9, 1),
            (10, 2),
            (13, 1),
            (14, 2),
        ],
        (AdventureStyle::Folk, Pace::Calm) => &[(0, 0), (1, 1), (2, 2), (3, 3), (8, 2)],
        (AdventureStyle::Folk, Pace::Walking) => match variant % 3 {
            0 => &[
                (0, 0),
                (2, 2),
                (4, 1),
                (6, 2),
                (8, 3),
                (10, 2),
                (12, 1),
                (14, 2),
            ],
            1 => &[(0, 0), (4, 2), (6, 1), (8, 3), (12, 2)],
            _ => &[(2, 1), (4, 2), (6, 3), (10, 2), (12, 1), (14, 2)],
        },
        (AdventureStyle::Folk, Pace::Driving) => &[
            (0, 0),
            (2, 2),
            (4, 3),
            (6, 2),
            (8, 0),
            (10, 2),
            (12, 3),
            (14, 2),
        ],
        (AdventureStyle::Dark, Pace::Calm) => match variant % 2 {
            0 => &[(4, 2)],
            _ => &[(10, 1)],
        },
        (AdventureStyle::Dark, Pace::Walking) => &[(0, 0), (10, 2)],
        (AdventureStyle::Dark, Pace::Driving) => &[(0, 0), (6, 2), (8, 0), (14, 1)],
        (AdventureStyle::Orchestral, Pace::Calm) => &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)],
        (AdventureStyle::Orchestral, Pace::Walking) => &[(0, 0), (6, 2), (12, 1)],
        (AdventureStyle::Orchestral, Pace::Driving) => &[(0, 0), (8, 2)],
    }
}

fn add_harp(sink: &mut EventSink, section: &Section, phrases: &[PhrasePlan]) {
    let target = match section.style {
        AdventureStyle::Dark => 57,
        AdventureStyle::Folk => 62,
        AdventureStyle::Orchestral => 66,
    };
    for phrase in phrases {
        let variant = (section.seed % 3) + phrase.index;
        for local_bar in 0..PHRASE_BARS {
            let bar_start = phrase.start + local_bar * section.bar;
            if phrase.harp_intro && local_bar < 2 {
                continue;
            }
            let closing =
                matches!(phrase.kind, PhraseKind::Cadence) && local_bar == PHRASE_BARS - 1;
            let figure: &[(u32, u8)] = if closing {
                &[(0, 0)]
            } else {
                harp_figure(section.style, section.scene(), variant)
            };
            let bar_index = bar_start / section.bar;
            for &(position, tone) in figure {
                let at = bar_start + position * section.sixteenth;
                let Some(chord) = chord_at(&phrase.chords, at) else {
                    continue;
                };
                let offsets = match chord.kind {
                    ChordKind::Sus2 => [0, 1, 4, 7, 8],
                    ChordKind::Sus4 => [0, 3, 4, 7, 10],
                    _ => [0, 2, 4, 7, 9],
                };
                let degree = chord.degree + offsets[usize::from(tone)];
                let pitch = section.place(degree, target + i32::from(tone) * 3, 48, 76);
                let length = match section.style {
                    AdventureStyle::Folk => section.sixteenth * 5 / 2,
                    AdventureStyle::Dark => section.sixteenth * 7 / 2,
                    AdventureStyle::Orchestral => section.sixteenth * 12 / 5,
                };
                let velocity = (0.15 + section.traits.wonder * 0.08 + section.energy * 0.05)
                    * phrase_gain(bar_index)
                    * metric_accent(position);
                sink.note("harp", at, length, velocity, pitch, "harp", false);
            }
        }
    }
}

/// The battle ostinato: an Orchestral string gallop (two sixteenths and an
/// eighth) or a low Dark eighth-note pulse, riding the chord root.
fn add_ostinato(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    if section.pace != Pace::Driving || section.style == AdventureStyle::Folk {
        return;
    }
    let gallop: &[(u32, u8)] = &[
        (0, 0),
        (1, 0),
        (2, 1),
        (4, 0),
        (5, 0),
        (6, 2),
        (8, 0),
        (9, 0),
        (10, 1),
        (12, 0),
        (13, 0),
        (14, 2),
    ];
    let pulse: &[(u32, u8)] = &[
        (0, 0),
        (2, 0),
        (4, 1),
        (6, 0),
        (8, 0),
        (10, 0),
        (12, 1),
        (14, 0),
    ];
    let (figure, target) = match section.style {
        AdventureStyle::Orchestral if section.traits.motion >= 0.3 => (gallop, 55),
        AdventureStyle::Orchestral => (pulse, 55),
        _ => (pulse, 50),
    };
    let final_bar = section.plan.bars * section.bar - section.bar;
    let velocity = 0.13 + section.energy * 0.06 + section.traits.danger * 0.04;
    for span in chords.iter().filter(|span| span.start < final_bar) {
        let root = section.place(span.chord.degree, target, target - 6, target + 6);
        let span_sixteenths = span.length / section.sixteenth;
        let mut offset = 0;
        while offset < span_sixteenths {
            for &(position, role) in figure {
                let local = offset + position;
                if local >= span_sixteenths {
                    break;
                }
                let pitch = match role {
                    0 => root,
                    1 => section.place(span.chord.degree + 4, root + 7, root + 3, root + 9),
                    _ => root + 12,
                };
                let at = span.start + local * section.sixteenth;
                let quick = figure.iter().any(|&(p, _)| p == position + 1);
                let length = if quick {
                    section.sixteenth * 3 / 4
                } else {
                    section.sixteenth * 3 / 2
                };
                sink.note(
                    "ostinato",
                    at,
                    length,
                    velocity * metric_accent((at / section.sixteenth) % 16),
                    fold_into(pitch, 36, 72),
                    "vielle",
                    false,
                );
            }
            offset += 16;
        }
    }
}

type Hit = (&'static str, u32, u8);

const FRAME: &str = "frame-drum";
const TAMB: &str = "tambourine";

/// A scene's one-bar groove as `(voice, sixteenth, accent)`; accent 2 marks
/// the downbeat stress, 1 an ordinary stroke.
fn groove(style: AdventureStyle, scene: Scene, phrase: u32, motion: f64) -> &'static [Hit] {
    let jig = feel(style, scene) == Feel::Jig;
    match style {
        AdventureStyle::Folk => match scene {
            Scene::Camp if phrase >= 2 => &[(TAMB, 8, 1)],
            Scene::Explore if phrase % 2 == 1 && motion > 0.45 => &[(FRAME, 0, 2), (TAMB, 12, 1)],
            Scene::Town => &[(FRAME, 0, 2), (TAMB, 6, 1), (FRAME, 8, 1), (TAMB, 14, 1)],
            Scene::Combat | Scene::Skirmish => {
                &[(FRAME, 0, 2), (FRAME, 6, 1), (TAMB, 10, 1), (FRAME, 12, 1)]
            }
            Scene::Boss => &[(FRAME, 0, 2), (FRAME, 8, 2), (TAMB, 14, 1)],
            Scene::Victory if phrase < 6 && jig => &[
                (FRAME, 0, 2),
                (TAMB, 2, 1),
                (FRAME, 8, 1),
                (TAMB, 10, 1),
                (TAMB, 14, 1),
            ],
            Scene::Assault => &[
                (FRAME, 0, 2),
                (TAMB, 2, 1),
                (FRAME, 4, 1),
                (TAMB, 6, 1),
                (FRAME, 8, 2),
                (FRAME, 10, 1),
                (FRAME, 12, 1),
                (TAMB, 14, 1),
            ],
            Scene::Chase => &[
                (FRAME, 0, 2),
                (FRAME, 4, 1),
                (FRAME, 8, 2),
                (TAMB, 10, 1),
                (FRAME, 12, 1),
            ],
            Scene::Festival => &[
                (FRAME, 0, 2),
                (TAMB, 2, 1),
                (TAMB, 6, 1),
                (FRAME, 8, 2),
                (TAMB, 10, 1),
                (TAMB, 14, 1),
            ],
            Scene::Reunion => &[(FRAME, 0, 2), (TAMB, 6, 1), (TAMB, 12, 1)],
            Scene::Dawn => &[(TAMB, 8, 1)],
            _ => &[],
        },
        AdventureStyle::Dark => match scene {
            Scene::Town | Scene::Festival | Scene::Reunion => &[(FRAME, 0, 2), (TAMB, 12, 1)],
            Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Chase => {
                &[(FRAME, 0, 2), (FRAME, 6, 1), (FRAME, 12, 1), (TAMB, 14, 1)]
            }
            Scene::Assault => &[
                (FRAME, 0, 2),
                (FRAME, 6, 1),
                (FRAME, 8, 1),
                (FRAME, 12, 1),
                (TAMB, 14, 1),
            ],
            Scene::Dungeon if phrase >= 2 => &[(FRAME, 0, 1)],
            Scene::Victory if phrase < 4 => &[(FRAME, 0, 2), (FRAME, 12, 1)],
            Scene::Dawn => &[(TAMB, 8, 1)],
            _ => &[],
        },
        AdventureStyle::Orchestral => match scene {
            Scene::Camp if phrase >= 2 => &[(FRAME, 0, 1), (TAMB, 12, 1)],
            Scene::Explore => &[(FRAME, 0, 2), (TAMB, 8, 1)],
            Scene::Town | Scene::Victory | Scene::Festival | Scene::Reunion => {
                &[(FRAME, 0, 2), (TAMB, 4, 1), (FRAME, 8, 1), (TAMB, 12, 1)]
            }
            Scene::Combat | Scene::Boss | Scene::Skirmish | Scene::Chase => &[
                (FRAME, 0, 2),
                (TAMB, 4, 1),
                (FRAME, 6, 1),
                (FRAME, 8, 2),
                (TAMB, 10, 1),
                (FRAME, 12, 1),
                (TAMB, 14, 1),
            ],
            Scene::Assault => &[
                (FRAME, 0, 2),
                (TAMB, 2, 1),
                (TAMB, 4, 1),
                (FRAME, 6, 1),
                (FRAME, 8, 2),
                (TAMB, 10, 1),
                (FRAME, 12, 1),
                (TAMB, 14, 1),
            ],
            Scene::Dawn => &[(TAMB, 8, 1)],
            Scene::Sanctuary if phrase == 2 => &[(TAMB, 8, 1)],
            _ => &[],
        },
    }
}

/// A phrase's last bar: calm scenes breathe; walking scenes turn the bar
/// around; driving scenes roll into the next phrase.
fn phrase_fill(pace: Pace) -> &'static [Hit] {
    match pace {
        Pace::Calm => &[],
        Pace::Walking => &[(FRAME, 0, 2), (FRAME, 8, 1), (TAMB, 12, 1), (TAMB, 14, 1)],
        Pace::Driving => &[
            (FRAME, 0, 2),
            (FRAME, 4, 1),
            (FRAME, 8, 1),
            (FRAME, 10, 1),
            (FRAME, 12, 1),
            (FRAME, 13, 1),
            (FRAME, 14, 2),
            (FRAME, 15, 2),
        ],
    }
}

fn add_percussion(sink: &mut EventSink, section: &Section, phrases: &[PhrasePlan]) {
    let base = 0.2 + section.energy * 0.15 + section.traits.danger * 0.08;
    let ghost_chance = 0.04 + section.traits.motion * 0.12 + section.traits.danger * 0.14;
    for phrase in phrases {
        let pattern = groove(
            section.style,
            section.scene(),
            phrase.index,
            section.traits.motion,
        );
        if pattern.is_empty() {
            continue;
        }
        for local_bar in 0..PHRASE_BARS {
            let bar_start = phrase.start + local_bar * section.bar;
            let bar_index = bar_start / section.bar;
            let last = local_bar == PHRASE_BARS - 1;
            if last && section.pace == Pace::Calm && phrase.index > 0 {
                continue;
            }
            let hits = if last && phrase.index > 0 {
                phrase_fill(section.pace)
            } else {
                pattern
            };
            let crescendo = last && section.pace == Pace::Driving;
            for &(voice, position, accent) in hits {
                let frame = voice == FRAME;
                let ramp = if crescendo {
                    0.8 + f64::from(position) / 16.0 * 0.3
                } else {
                    1.0
                };
                let weight = if accent == 2 { 1.0 } else { 0.85 };
                sink.hit(
                    bar_start + position * section.sixteenth,
                    if frame {
                        section.sixteenth * 2
                    } else {
                        section.sixteenth
                    },
                    (base + if frame { 0.035 } else { 0.0 }) * weight * ramp,
                    voice,
                );
            }
            if section.pace == Pace::Calm || last {
                continue;
            }
            // Ghost strokes on the empty eighths: more with motion and danger.
            for position in (0..16).step_by(2) {
                if hits.iter().any(|&(_, p, _)| p == position) {
                    continue;
                }
                if section.chance("ghost", bar_index, position) < ghost_chance {
                    sink.hit(
                        bar_start + position * section.sixteenth,
                        section.sixteenth,
                        base * 0.5,
                        TAMB,
                    );
                }
            }
        }
    }
}

/// Orchestral timpani on the tonic and dominant: downbeats and rolls in the
/// fights, arrivals in the heroic scenes, a distant stroke in the dungeon,
/// and a phrase-opening stroke anywhere once danger runs high.
fn add_timpani(
    sink: &mut EventSink,
    section: &Section,
    phrases: &[PhrasePlan],
    chords: &[ChordSpan],
) {
    if section.style != AdventureStyle::Orchestral {
        return;
    }
    let tonic = section.place(0, 45, 38, 52);
    let dominant = section.place(4, tonic - 5, 38, 52);
    let tuned = |chord: Chord| {
        if chord.contains(0) {
            Some(tonic)
        } else if chord.contains(4) {
            Some(dominant)
        } else {
            None
        }
    };
    let base = 0.2 + section.energy * 0.2 + section.traits.danger * 0.1;
    let stroke = section.sixteenth * 6;
    let scene = section.scene();
    let count = phrases.len() as u32;
    for phrase in phrases {
        let roll_in = phrase.index + 1 < count;
        match section.pace {
            Pace::Driving => {
                for local_bar in 0..PHRASE_BARS {
                    if scene != Scene::Boss && scene != Scene::Assault && local_bar % 2 == 1 {
                        continue;
                    }
                    let at = phrase.start + local_bar * section.bar;
                    if let Some(pitch) = chord_at(chords, at).and_then(tuned) {
                        sink.note("timpani", at, stroke, base, pitch, "timpani", false);
                    }
                }
                if roll_in {
                    let bar_start = phrase.start + 3 * section.bar;
                    for step in 0..4u32 {
                        let at = bar_start + (12 + step) * section.sixteenth;
                        let swell = 0.6 + f64::from(step) * 0.12;
                        sink.note(
                            "timpani",
                            at,
                            section.sixteenth,
                            base * swell,
                            dominant,
                            "timpani",
                            false,
                        );
                    }
                }
            }
            Pace::Walking if matches!(scene, Scene::Victory | Scene::Festival | Scene::Explore) => {
                let at = phrase.start + 3 * section.bar;
                if let Some(pitch) = chord_at(chords, at).and_then(tuned) {
                    sink.note("timpani", at, stroke, base * 0.9, pitch, "timpani", false);
                }
            }
            _ if scene == Scene::Dungeon => {
                sink.note(
                    "timpani",
                    phrase.start,
                    stroke,
                    base * 0.55,
                    tonic,
                    "timpani",
                    false,
                );
            }
            _ if section.chance("timpani-danger", phrase.index, 0)
                < (section.traits.danger - 0.5) * 1.2 =>
            {
                sink.note(
                    "timpani",
                    phrase.start,
                    stroke,
                    base * 0.7,
                    tonic,
                    "timpani",
                    false,
                );
            }
            _ => {}
        }
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
            assert_eq!(
                adventure_phase_energy(scene),
                energy,
                "energy for {scene:?}"
            );
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
