use crate::rng::DeterministicRandom;
use crate::score::{MusicEvent, PortableSection};
use crate::theory::{mode_intervals, phrase_gain, scale_pitch};

use super::{AdventureStyle, NormalizedTraits};

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
}

pub(super) struct SectionPlan {
    pub(super) id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    pub(super) bars: u32,
    pub(super) scene: Scene,
}

pub(super) const SECTION_PLANS: &[SectionPlan; 8] = &[
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
        id: "dungeon",
        label: "The Deep Halls",
        feeling: "cold stone / distant steps",
        color: "#59616c",
        bars: 16,
        scene: Scene::Dungeon,
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
        id: "boss",
        label: "No Retreat",
        feeling: "ancient dread / final challenge",
        color: "#713343",
        bars: 16,
        scene: Scene::Boss,
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
        id: "victory",
        label: "Lanterns at Dawn",
        feeling: "homecoming / earned release",
        color: "#ead27e",
        bars: 32,
        scene: Scene::Victory,
    },
];

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
        AdventureStyle::Folk => 82.0,
        AdventureStyle::Dark => 68.0,
        AdventureStyle::Orchestral => 90.0,
    };
    (base + traits.motion * 16.0 - traits.mystery * 5.0)
        .clamp(58.0, 112.0)
        .round()
}

pub(super) fn mode_for(style: AdventureStyle, scene: Scene) -> &'static str {
    match (style, scene) {
        (AdventureStyle::Folk, Scene::Camp | Scene::Victory) => "ionian",
        (AdventureStyle::Folk, Scene::Town) => "mixolydian",
        (AdventureStyle::Folk, Scene::Dungeon | Scene::Boss) => "aeolian",
        (AdventureStyle::Folk, Scene::Sanctuary) => "lydian",
        (AdventureStyle::Folk, Scene::Explore | Scene::Combat) => "dorian",
        (AdventureStyle::Dark, Scene::Dungeon | Scene::Boss) => "phrygian",
        (AdventureStyle::Dark, Scene::Explore | Scene::Combat) => "aeolian",
        (AdventureStyle::Dark, Scene::Victory) => "mixolydian",
        (AdventureStyle::Dark, _) => "dorian",
        (AdventureStyle::Orchestral, Scene::Dungeon | Scene::Boss) => "aeolian",
        (AdventureStyle::Orchestral, Scene::Explore) => "mixolydian",
        (AdventureStyle::Orchestral, Scene::Combat) => "dorian",
        (AdventureStyle::Orchestral, Scene::Sanctuary) => "lydian",
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

fn progression(kind: PhraseKind, scene: Scene, phrase_index: u32, variant: usize) -> [i32; 4] {
    let color = match (scene_index(scene) + variant + phrase_index as usize) % 3 {
        0 => 1,
        1 => 3,
        _ => 5,
    };
    match kind {
        PhraseKind::Antecedent => [0, color, 1 + (variant % 2) as i32, 4],
        PhraseKind::Consequent => [0, color, 4, 0],
        PhraseKind::Development => [5, 2 + (phrase_index % 2) as i32, color, 4],
        PhraseKind::Return => [0, 3, 4, 0],
        PhraseKind::Cadence => [1, 3, 4, 0],
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
        (AdventureStyle::Folk, Scene::Town | Scene::Boss) => "vielle",
        (AdventureStyle::Folk, _) => "recorder",
        (AdventureStyle::Dark, Scene::Camp | Scene::Sanctuary) => "recorder",
        (AdventureStyle::Dark, _) => "vielle",
        (AdventureStyle::Orchestral, Scene::Dungeon) => "vielle",
        (AdventureStyle::Orchestral, _) => "recorder",
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

fn pedal_start(scene: Scene, style: AdventureStyle, phrase: u32) -> bool {
    match (style, scene) {
        (AdventureStyle::Dark, Scene::Dungeon | Scene::Boss) => phrase.is_multiple_of(2),
        (_, Scene::Dungeon) => phrase == 0 || phrase == 2,
        (AdventureStyle::Folk, Scene::Camp | Scene::Sanctuary) => phrase == 0,
        (AdventureStyle::Orchestral, Scene::Sanctuary) => phrase == 2,
        _ => false,
    }
}

fn pedal_covers(scene: Scene, style: AdventureStyle, bar: u32) -> bool {
    let phrase = bar / PHRASE_BARS;
    pedal_start(scene, style, phrase) && bar % PHRASE_BARS < 2
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
    if !pedal_start(plan.scene, style, phrase) {
        return;
    }
    let pitch = nearest_scale_pitch(dna.tonic_pitch_class, 0, intervals, 43, 36, 52);
    push_note(
        events,
        plan.id,
        "pedal",
        &mut counters.pedal,
        phrase * PHRASE_BARS * bar_ticks,
        2 * bar_ticks,
        0.1 + scene_energy(plan.scene) * 0.05,
        pitch,
        "vielle",
        false,
    );
}

fn bass_onsets(style: AdventureStyle, scene: Scene, final_bar: bool) -> &'static [(u8, i32)] {
    if final_bar {
        return &[(0, 0)];
    }
    match (style, scene) {
        (AdventureStyle::Folk, Scene::Town | Scene::Combat) => &[(0, 0), (4, 4)],
        (AdventureStyle::Folk, Scene::Boss) => &[(0, 0), (6, 0)],
        (AdventureStyle::Dark, Scene::Combat | Scene::Boss) => &[(0, 0), (3, 4), (6, 0)],
        (AdventureStyle::Orchestral, Scene::Town | Scene::Combat | Scene::Victory) => {
            &[(0, 0), (4, 4)]
        }
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
    if pedal_covers(plan.scene, style, bar) && !final_bar {
        return;
    }
    if matches!(style, AdventureStyle::Folk)
        && matches!(plan.scene, Scene::Camp | Scene::Dungeon | Scene::Sanctuary)
        && bar % 2 == 1
        && !final_bar
    {
        return;
    }
    let voice = if style == AdventureStyle::Dark {
        "harp"
    } else {
        "vielle"
    };
    for &(onset, offset) in bass_onsets(style, plan.scene, final_bar) {
        let pitch = nearest_scale_pitch(
            dna.tonic_pitch_class,
            chord_degree + offset,
            intervals,
            42,
            36,
            55,
        );
        let duration = if onset == 0 && bass_onsets(style, plan.scene, final_bar).len() == 1 {
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
            0.19 + scene_energy(plan.scene) * 0.11 + traits.danger * 0.04,
            pitch,
            voice,
            false,
        );
    }
}

fn harmony_active(
    style: AdventureStyle,
    scene: Scene,
    kind: PhraseKind,
    bar: u32,
    local_bar: u32,
) -> bool {
    if bar == 0 && scene == Scene::Camp {
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
    if !harmony_active(style, plan.scene, kind, bar, local_bar) {
        return;
    }
    let voices = match style {
        AdventureStyle::Folk | AdventureStyle::Dark => 2,
        AdventureStyle::Orchestral => 3,
    };
    let duration = match style {
        AdventureStyle::Folk => bar_ticks * 5 / 8,
        AdventureStyle::Dark => bar_ticks * 7 / 8,
        AdventureStyle::Orchestral => bar_ticks * 15 / 16,
    };
    for &pitch in chord.iter().take(voices) {
        push_note(
            events,
            plan.id,
            "harmony",
            &mut counters.harmony,
            bar * bar_ticks,
            duration,
            (0.11 + traits.wonder * 0.05 + scene_energy(plan.scene) * 0.035) * phrase_gain(bar),
            pitch,
            "vielle",
            false,
        );
    }
}

fn harp_pattern(
    style: AdventureStyle,
    scene: Scene,
    bar: u32,
    kind: PhraseKind,
    variant: usize,
) -> Vec<(u8, i32)> {
    if matches!(kind, PhraseKind::Cadence) && bar % PHRASE_BARS == 3 {
        return vec![(0, 0)];
    }
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
        },
        AdventureStyle::Dark => match scene {
            Scene::Combat | Scene::Boss => {
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
            Scene::Town => vec![(0, 0), (5, 4)],
            _ => {
                if bar.is_multiple_of(2) {
                    vec![(0, 0), (6, 4)]
                } else {
                    vec![(3, 2)]
                }
            }
        },
        AdventureStyle::Orchestral => {
            let mut pattern = match scene {
                Scene::Camp | Scene::Dungeon => vec![(0, 0), (4, 2), (6, 4)],
                Scene::Town => vec![(0, 0), (2, 2), (3, 4), (5, 2), (6, 4)],
                Scene::Combat | Scene::Boss => {
                    vec![(0, 0), (1, 2), (3, 4), (4, 0), (6, 4), (7, 2)]
                }
                Scene::Explore | Scene::Sanctuary | Scene::Victory => {
                    vec![(0, 0), (2, 2), (4, 4), (5, 2), (7, 4)]
                }
            };
            if (bar as usize + variant) % 2 == 1 {
                for (_, degree) in &mut pattern {
                    *degree = 4 - *degree;
                }
            }
            pattern
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
    let pattern = harp_pattern(style, plan.scene, bar, kind, variant);
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
        let duration = match style {
            AdventureStyle::Folk if plan.scene == Scene::Town => pulse * 3 / 4,
            AdventureStyle::Folk => pulse * 5 / 4,
            AdventureStyle::Dark => pulse * 7 / 4,
            AdventureStyle::Orchestral => pulse * 6 / 5,
        };
        push_note(
            events,
            plan.id,
            "harp",
            &mut counters.harp,
            bar * bar_ticks + u32::from(*onset) * pulse,
            duration,
            (0.13 + traits.wonder * 0.07 + scene_energy(plan.scene) * 0.035) * phrase_gain(bar),
            pitch,
            "harp",
            false,
        );
    }
}

fn melody_onsets(
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
    if scene == Scene::Camp && local_bar == 0 && matches!(kind, PhraseKind::Antecedent) {
        return Vec::new();
    }
    if style == AdventureStyle::Dark && scene == Scene::Dungeon && local_bar == 1 {
        return Vec::new();
    }
    if local_bar == PHRASE_BARS - 1 {
        return vec![0, 4];
    }
    let mut onsets = match style {
        AdventureStyle::Folk => {
            if scene == Scene::Town {
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
        AdventureStyle::Orchestral => match (local_bar as usize + variant) % 4 {
            0 => vec![0, 2, 4, 6],
            1 => vec![0, 3, 4, 7],
            2 => vec![1, 3, 6],
            _ => vec![0, 4, 6],
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
    let onsets = melody_onsets(style, plan.scene, kind, local_bar, variant, traits.motion);
    for (note, onset) in onsets.iter().enumerate() {
        let is_phrase_arrival = local_bar == PHRASE_BARS - 1 && note == onsets.len() - 1;
        let degree = if is_phrase_arrival {
            match kind {
                PhraseKind::Antecedent | PhraseKind::Development => 4,
                PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Cadence => 0,
            }
        } else if note == 0 && *onset == 0 {
            chord_degree + [0, 2, 4][(bar as usize + phrase as usize) % 3]
        } else {
            chord_degree + transformed_motif_degree(dna, kind, phrase, local_bar, note)
        };
        let phrase_register = match kind {
            PhraseKind::Development => 2,
            PhraseKind::Return => -1,
            PhraseKind::Cadence => -2,
            _ => 0,
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
            match (style, plan.scene) {
                (AdventureStyle::Folk, Scene::Town) => gap * 3 / 5,
                (AdventureStyle::Dark, _) => gap * 4 / 5,
                (AdventureStyle::Orchestral, Scene::Combat | Scene::Boss) => gap * 2 / 3,
                _ => gap * 3 / 4,
            }
            .max(pulse / 2)
        };
        let articulation = if note == 0 { 0.02 } else { 0.0 };
        let velocity =
            (0.22 + traits.wonder * 0.055 + scene_energy(plan.scene) * 0.08 + articulation)
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
            melody_voice(style, plan.scene),
            true,
        );
        *previous_pitch = Some(pitch);
    }
}

fn percussion_pattern(
    style: AdventureStyle,
    scene: Scene,
    phrase: u32,
    local_bar: u32,
    motion: f64,
) -> &'static [(&'static str, u8)] {
    if local_bar == PHRASE_BARS - 1 && phrase > 0 {
        return &[];
    }
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
            _ => &[],
        },
        AdventureStyle::Dark => match scene {
            Scene::Town => &[("frame-drum", 0), ("tambourine", 6)],
            Scene::Combat | Scene::Boss => &[
                ("frame-drum", 0),
                ("frame-drum", 3),
                ("frame-drum", 6),
                ("tambourine", 7),
            ],
            Scene::Dungeon if phrase >= 2 => &[("frame-drum", 0)],
            Scene::Victory if phrase < 4 => &[("frame-drum", 0), ("frame-drum", 6)],
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
            Scene::Combat | Scene::Boss => &[
                ("frame-drum", 0),
                ("tambourine", 2),
                ("frame-drum", 3),
                ("frame-drum", 4),
                ("tambourine", 5),
                ("frame-drum", 6),
                ("tambourine", 7),
            ],
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
    for &(voice, onset) in percussion_pattern(style, plan.scene, phrase, local_bar, traits.motion) {
        let is_frame = voice == "frame-drum";
        push_percussion(
            events,
            plan.id,
            &mut counters.percussion,
            bar * bar_ticks + u32::from(onset) * pulse,
            if is_frame { pulse } else { pulse / 2 },
            0.16 + scene_energy(plan.scene) * 0.13
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
        0.13 + scene_energy(plan.scene) * 0.04,
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
    let intervals = mode_intervals(mode_for(style, plan.scene));
    let mut rng = DeterministicRandom::new(section_seed);
    let section_variant = dna.rhythm_variant + rng.integer(8) as usize;
    let mut events = Vec::new();
    let mut counters = EventCounters::new();
    let mut previous_chord = None;
    let mut previous_melody = None;

    for phrase in 0..phrase_count {
        let kind = phrase_kind(phrase, phrase_count);
        let chords = progression(kind, plan.scene, phrase, dna.harmony_variant);
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
