use crate::event_sink::EventSink;
use crate::melody::{compose_theme, fold_into, metric_accent, register_shift, Tone};
use crate::rng::{keyed_unit, DeterministicRandom};
use crate::score::PortableSection;
use crate::theory::{mode_intervals, scale_pitch};

use super::harmony::{
    chord_at, phrase_harmony, voice_chord, ChordSpan, HarmonicColor, PhraseHarmonyInput,
};
use super::{CozyStyle, NormalizedTraits};

const BEATS_PER_BAR: u32 = 4;
const PHRASE_BARS: u32 = 4;
/// Sixteenths in a bar.
const BAR: u32 = 16;
const MELODY_MIN: i32 = 55;
const MELODY_MAX: i32 = 86;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scene {
    Dawn,
    Morning,
    Market,
    Noon,
    Rain,
    Evening,
    Festival,
    Night,
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
        id: "dawn",
        label: "First Light Coffee",
        feeling: "sleepy kettle / first birds",
        color: "#f2c9a0",
        bars: 16,
        scene: Scene::Dawn,
    },
    SectionPlan {
        id: "morning",
        label: "Morning Chores",
        feeling: "fresh dew / busy hands",
        color: "#f4d27a",
        bars: 32,
        scene: Scene::Morning,
    },
    SectionPlan {
        id: "market",
        label: "Market Day",
        feeling: "friendly chatter / warm bread",
        color: "#e8a15c",
        bars: 32,
        scene: Scene::Market,
    },
    SectionPlan {
        id: "noon",
        label: "Sunny Fields",
        feeling: "long rows / easy work",
        color: "#b8d67a",
        bars: 32,
        scene: Scene::Noon,
    },
    SectionPlan {
        id: "rain",
        label: "Rain on the Roof",
        feeling: "window drops / warm blanket",
        color: "#8aa6c1",
        bars: 16,
        scene: Scene::Rain,
    },
    SectionPlan {
        id: "evening",
        label: "Golden Hour",
        feeling: "long shadows / porch light",
        color: "#e59a6b",
        bars: 32,
        scene: Scene::Evening,
    },
    SectionPlan {
        id: "festival",
        label: "Harvest Festival",
        feeling: "paper lanterns / dancing",
        color: "#e0703f",
        bars: 32,
        scene: Scene::Festival,
    },
    SectionPlan {
        id: "night",
        label: "Lanterns Out",
        feeling: "crickets / quiet stars",
        color: "#4b5b8a",
        bars: 16,
        scene: Scene::Night,
    },
];

/// The role a Cozy section plays in a composed day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CozyPhaseRole {
    Intro,
    Groove,
    Peak,
    Break,
    Outro,
}

pub(super) fn cozy_phase_role(scene: Scene) -> CozyPhaseRole {
    match scene {
        Scene::Dawn => CozyPhaseRole::Intro,
        Scene::Morning | Scene::Market | Scene::Noon | Scene::Evening => CozyPhaseRole::Groove,
        Scene::Festival => CozyPhaseRole::Peak,
        Scene::Rain => CozyPhaseRole::Break,
        Scene::Night => CozyPhaseRole::Outro,
    }
}

/// Energy on a 0-100 scale.
pub(super) fn cozy_phase_energy(scene: Scene) -> u32 {
    match scene {
        Scene::Dawn => 20,
        Scene::Morning => 50,
        Scene::Market => 64,
        Scene::Noon => 56,
        Scene::Rain => 30,
        Scene::Evening => 44,
        Scene::Festival => 80,
        Scene::Night => 16,
    }
}

/// Warm, flat keys a keyboard player reaches for.
const TONIC_PITCH_CLASSES: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];

/// Two-bar theme rhythms as `(onset, length)` in sixteenths: relaxed,
/// syncopated, often starting off the downbeat, each ending on a held note.
const THEME_RHYTHMS: [&[(u32, u32)]; 6] = [
    &[
        (0, 3),
        (3, 3),
        (6, 2),
        (8, 8),
        (16, 2),
        (18, 2),
        (20, 4),
        (24, 8),
    ],
    &[(2, 2), (4, 2), (6, 4), (10, 6), (16, 2), (18, 4), (22, 10)],
    &[
        (0, 4),
        (4, 2),
        (6, 2),
        (8, 2),
        (10, 6),
        (18, 2),
        (20, 4),
        (24, 8),
    ],
    &[(0, 2), (2, 4), (6, 2), (8, 8), (16, 4), (20, 4), (24, 8)],
    &[(4, 2), (6, 2), (8, 4), (12, 4), (16, 6), (22, 2), (24, 8)],
    &[(0, 6), (6, 2), (8, 2), (10, 2), (12, 4), (16, 8), (24, 8)],
];

/// What one seed fixes for the whole day: the key, the theme, and which loop
/// each harmonic colour plays.
pub(super) struct PieceDna {
    pub(super) tonic_pitch_class: i32,
    theme: Vec<Tone>,
    loop_variant: usize,
}

impl PieceDna {
    pub(super) fn new(seed: u32) -> Self {
        let mut rng = DeterministicRandom::new(seed);
        let tonic_pitch_class = *rng.pick(&TONIC_PITCH_CLASSES);
        let theme = compose_theme(&mut rng, &THEME_RHYTHMS);
        Self {
            tonic_pitch_class,
            theme,
            loop_variant: rng.integer(3) as usize,
        }
    }
}

pub(super) fn tempo(style: CozyStyle, traits: NormalizedTraits) -> f64 {
    let base = match style {
        CozyStyle::Acoustic => 92.0,
        CozyStyle::Lofi => 74.0,
        CozyStyle::Bossa => 116.0,
    };
    (base + traits.bustle * 14.0).round()
}

/// Where each beat's written off-beat is heard: bossa stays straight, lo-fi
/// leans back, and the swing trait pushes every style further.
fn swing_point(style: CozyStyle, traits: NormalizedTraits) -> f64 {
    let (base, reach) = match style {
        CozyStyle::Acoustic => (0.52, 0.08),
        CozyStyle::Lofi => (0.58, 0.1),
        CozyStyle::Bossa => (0.5, 0.04),
    };
    base + traits.swing * reach
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Drums {
    None,
    Brushes,
    Light,
    Groove,
    Full,
}

/// How a scene is played.
struct Character {
    color: HarmonicColor,
    drums: Drums,
    /// How often a phrase's melody answers itself instead of leaving the
    /// second half to the answering instrument.
    singing: f64,
    pad: bool,
    register: i32,
}

fn character(scene: Scene) -> Character {
    let (color, drums, singing, pad, register) = match scene {
        Scene::Dawn => (HarmonicColor::Still, Drums::None, 0.25, true, -2),
        Scene::Morning => (HarmonicColor::Bright, Drums::Light, 0.7, false, 2),
        Scene::Market => (HarmonicColor::Busy, Drums::Groove, 0.8, false, 1),
        Scene::Noon => (HarmonicColor::Bright, Drums::Groove, 0.6, false, 0),
        Scene::Rain => (HarmonicColor::Wistful, Drums::Brushes, 0.35, true, -3),
        Scene::Evening => (HarmonicColor::Golden, Drums::Light, 0.5, true, -1),
        Scene::Festival => (HarmonicColor::Busy, Drums::Full, 0.9, false, 3),
        Scene::Night => (HarmonicColor::Still, Drums::None, 0.2, true, -4),
    };
    Character {
        color,
        drums,
        singing,
        pad,
        register,
    }
}

struct Voices {
    melody: &'static str,
    answer: &'static str,
    comp: &'static str,
    pad: &'static str,
    bass: &'static str,
    melody_center: i32,
}

fn voices(style: CozyStyle) -> Voices {
    match style {
        CozyStyle::Acoustic => Voices {
            melody: "glass",
            answer: "bell",
            comp: "pluck",
            pad: "felt",
            bass: "bass",
            melody_center: 76,
        },
        CozyStyle::Lofi => Voices {
            melody: "felt",
            answer: "glass",
            comp: "epiano",
            pad: "dusk",
            bass: "triangle",
            melody_center: 69,
        },
        CozyStyle::Bossa => Voices {
            melody: "recorder",
            answer: "felt",
            comp: "pluck",
            pad: "epiano",
            bass: "bass",
            melody_center: 74,
        },
    }
}

pub(super) fn build_sections<F>(
    style: CozyStyle,
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
    style: CozyStyle,
    traits: NormalizedTraits,
    tonic: i32,
    character: Character,
    voices: Voices,
    bar: u32,
    sixteenth: u32,
    seed: u32,
    melody_octave: i32,
}

impl Section<'_> {
    fn chance(&self, tag: &str, a: u32, b: u32) -> f64 {
        keyed_unit(self.seed, tag, a, b)
    }

    /// Pitch class above the tonic of a major-scale degree.
    fn class(&self, degree: i32) -> i32 {
        scale_pitch(0, degree, &mode_intervals("ionian")).rem_euclid(12)
    }

    fn melody_pitch(&self, degree: i32) -> i32 {
        scale_pitch(60 + self.tonic, degree, &mode_intervals("ionian")) + self.melody_octave
    }

    /// The pitch of `class` (above the tonic) nearest `target` in range.
    fn place(&self, class: i32, target: i32, low: i32, high: i32) -> i32 {
        (low..=high)
            .filter(|pitch| (pitch - self.tonic - class).rem_euclid(12) == 0)
            .min_by_key(|pitch| ((pitch - target).abs(), *pitch))
            .expect("every pitch class fits an octave")
    }
}

struct PhrasePlan {
    index: u32,
    start: u32,
    chords: Vec<ChordSpan>,
    melody: Vec<Tone>,
    /// Bars three and four are left to the answering instrument.
    answered: bool,
}

fn build_section(
    plan: &SectionPlan,
    style: CozyStyle,
    traits: NormalizedTraits,
    dna: &PieceDna,
    ticks_per_beat: u32,
    section_seed: u32,
) -> PortableSection {
    let bar = ticks_per_beat * BEATS_PER_BAR;
    let mut section = Section {
        plan,
        style,
        traits,
        tonic: dna.tonic_pitch_class,
        character: character(plan.scene),
        voices: voices(style),
        bar,
        sixteenth: ticks_per_beat / 4,
        seed: section_seed,
        melody_octave: 0,
    };
    let phrases = plan_phrases(&section, dna);
    section.melody_octave = melody_octave(&section, &phrases);
    let mut sink = EventSink::new(
        plan.id,
        ticks_per_beat,
        swing_point(style, traits),
        plan.bars * bar,
    );
    let chords: Vec<ChordSpan> = phrases
        .iter()
        .flat_map(|p| p.chords.iter().copied())
        .collect();
    for phrase in &phrases {
        add_melody(&mut sink, &section, phrase);
        add_answer(&mut sink, &section, phrase);
        add_sparkle(&mut sink, &section, phrase);
        add_drums(&mut sink, &section, phrase);
    }
    add_comp(&mut sink, &section, &chords);
    add_pad(&mut sink, &section, &chords);
    add_bass(&mut sink, &section, &chords);
    PortableSection {
        id: plan.id.to_string(),
        label: plan.label.to_string(),
        feeling: plan.feeling.to_string(),
        color: plan.color.to_string(),
        length_ticks: plan.bars * bar,
        events: sink.finish(),
    }
}

/// Phrases alternate the day's loop with a contrasting one: A A' B A.
fn plan_phrases(section: &Section, dna: &PieceDna) -> Vec<PhrasePlan> {
    let count = section.plan.bars / PHRASE_BARS;
    (0..count)
        .map(|index| {
            let start = index * PHRASE_BARS * section.bar;
            let contrast = index % 4 == 2;
            let chords = phrase_harmony(&PhraseHarmonyInput {
                color: section.character.color,
                loop_index: dna.loop_variant + usize::from(contrast),
                phrase_start: start,
                bar_ticks: section.bar,
                seed: section.seed,
                phrase: index,
                jazz: section.traits.jazz,
            });
            let answered = index + 1 < count
                && section.chance("answered", index, 0) >= section.character.singing;
            let mut melody = phrase_line(&dna.theme, contrast, answered);
            fit_to_harmony(&mut melody, section, &chords, start);
            anticipate(&mut melody, section, &chords, start, index);
            PhrasePlan {
                index,
                start,
                chords,
                melody,
                answered,
            }
        })
        .collect()
}

/// The theme in bars one and two (lifted a third in the contrasting phrase),
/// then either nothing — the answering instrument replies — or the theme's
/// head a step lower and a held arrival on the fourth bar's downbeat.
fn phrase_line(theme: &[Tone], contrast: bool, answered: bool) -> Vec<Tone> {
    let lift = if contrast { 2 } else { 0 };
    let mut line: Vec<Tone> = theme
        .iter()
        .map(|tone| Tone {
            degree: tone.degree + lift,
            ..*tone
        })
        .collect();
    if answered {
        return line;
    }
    let head: Vec<Tone> = theme.iter().filter(|tone| tone.at < BAR).copied().collect();
    line.extend(head.iter().map(|tone| Tone {
        at: tone.at + 2 * BAR,
        length: tone.length,
        degree: tone.degree + lift - 1,
    }));
    let last = line.last().map_or(0, |tone| tone.degree);
    line.push(Tone {
        at: 3 * BAR,
        length: 10,
        degree: last - 1,
    });
    line
}

/// Strong beats and long notes rest on chord tones; a weak note stays only if
/// it moves by step and does not rub a semitone against the chord.
fn fit_to_harmony(line: &mut [Tone], section: &Section, chords: &[ChordSpan], start: u32) {
    for index in 0..line.len() {
        let tone = line[index];
        let Some(chord) = chord_at(chords, start + tone.at * section.sixteenth) else {
            continue;
        };
        let class = section.class(tone.degree);
        if chord.rests_on(class) {
            continue;
        }
        let strong = tone.at.is_multiple_of(8) || tone.length >= 6;
        let by_step =
            |other: Option<&Tone>| other.is_some_and(|o| (o.degree - tone.degree).abs() <= 1);
        let passing =
            by_step(index.checked_sub(1).map(|i| &line[i])) || by_step(line.get(index + 1));
        if !strong && passing && !chord.rubs(class) {
            continue;
        }
        let previous = index.checked_sub(1).map_or(tone.degree, |i| line[i].degree);
        let rising = tone.degree >= previous;
        if let Some(degree) = [1, -1, 2, -2, 3, -3]
            .into_iter()
            .map(|offset| tone.degree + offset)
            .filter(|degree| chord.rests_on(section.class(*degree)))
            .min_by_key(|degree| {
                let keeps = (*degree >= previous) == rising;
                (i32::from(!keeps), (degree - tone.degree).abs(), *degree)
            })
        {
            line[index].degree = degree;
        }
    }
}

/// Swing pushes a downbeat note an eighth early, into the previous bar, when
/// the note it cuts into has already ended and it does not rub that chord.
fn anticipate(line: &mut [Tone], section: &Section, chords: &[ChordSpan], start: u32, phrase: u32) {
    for index in 1..line.len() {
        let tone = line[index];
        if tone.at == 0 || !tone.at.is_multiple_of(BAR) {
            continue;
        }
        let pushed = tone.at - 2;
        let previous = line[index - 1];
        if previous.at + previous.length > pushed {
            continue;
        }
        let under = chord_at(chords, start + pushed * section.sixteenth);
        if under.is_none_or(|chord| chord.rubs(section.class(tone.degree))) {
            continue;
        }
        if section.chance("anticipate", phrase, tone.at) < section.traits.swing * 0.6 {
            line[index] = Tone {
                at: pushed,
                length: tone.length + 2,
                ..tone
            };
        }
    }
}

fn melody_octave(section: &Section, phrases: &[PhrasePlan]) -> i32 {
    let degrees: Vec<i32> = phrases
        .iter()
        .flat_map(|phrase| phrase.melody.iter().map(|tone| tone.degree))
        .collect();
    let center = section.voices.melody_center
        + section.character.register
        + (section.traits.warmth * 3.0).round() as i32;
    let pitches: Vec<i32> = degrees
        .iter()
        .map(|degree| scale_pitch(60 + section.tonic, *degree, &mode_intervals("ionian")))
        .collect();
    register_shift(
        &pitches,
        center,
        MELODY_MIN,
        MELODY_MAX,
        &[-24, -12, 0, 12, 24],
    )
}

fn add_melody(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let arch = [0.92, 1.0, 1.06, 0.95];
    let base = 0.3 + section.traits.warmth * 0.06;
    for tone in &phrase.melody {
        let pitch = fold_into(section.melody_pitch(tone.degree), MELODY_MIN, MELODY_MAX);
        let length = (tone.length * section.sixteenth * 7 / 8).max(section.sixteenth / 2);
        sink.note(
            "melody",
            phrase.start + tone.at * section.sixteenth,
            length,
            base * metric_accent(tone.at) * arch[(tone.at / BAR).min(3) as usize],
            pitch,
            section.voices.melody,
            true,
        );
    }
}

/// When the melody leaves the second half open, another instrument answers:
/// a short descending figure on the chord, then a held note.
fn add_answer(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    if !phrase.answered {
        return;
    }
    let at = phrase.start + 2 * section.bar;
    let Some(chord) = chord_at(&phrase.chords, at) else {
        return;
    };
    let classes = chord.pitch_classes();
    let top = section.voices.melody_center + section.character.register;
    let figure = [(2, 2, 3), (4, 2, 2), (6, 2, 1), (8, 8, 0)];
    let mut previous = top + 3;
    for (position, length, index) in figure {
        let class = classes[index % classes.len()];
        let pitch = section.place(class, previous - 2, top - 12, top + 6);
        previous = pitch;
        sink.note(
            "answer",
            at + position * section.sixteenth,
            length * section.sixteenth,
            0.24 + section.traits.warmth * 0.05,
            pitch,
            section.voices.answer,
            false,
        );
    }
}

/// A high bell on the chord's third at phrase openings, more often as warmth
/// rises; the festival always rings in its phrases.
fn add_sparkle(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let festive = section.plan.scene == Scene::Festival;
    if !festive && section.chance("sparkle", phrase.index, 0) >= section.traits.warmth * 0.45 {
        return;
    }
    let Some(chord) = chord_at(&phrase.chords, phrase.start) else {
        return;
    };
    let third = chord.pitch_classes()[1];
    let pitch = section.place(third, 81, 74, 88);
    sink.note(
        "sparkle",
        phrase.start,
        section.bar / 2,
        0.16,
        pitch,
        "bell",
        false,
    );
}

/// A comping figure as `(sixteenth, length, down-strum)` within one bar.
fn comp_figure(style: CozyStyle, drums: Drums, bar_index: u32) -> &'static [(u32, u32, bool)] {
    let still = matches!(drums, Drums::None | Drums::Brushes);
    match style {
        CozyStyle::Acoustic if still => &[
            (0, 3, true),
            (2, 3, true),
            (4, 3, true),
            (6, 3, true),
            (8, 3, true),
            (10, 3, true),
            (12, 3, true),
            (14, 3, true),
        ],
        CozyStyle::Acoustic => &[(0, 4, true), (6, 2, false), (8, 4, true), (14, 2, false)],
        CozyStyle::Lofi if still => &[(0, 14, true)],
        CozyStyle::Lofi => &[(0, 6, true), (10, 5, true)],
        CozyStyle::Bossa if bar_index.is_multiple_of(2) => &[
            (0, 2, true),
            (3, 2, true),
            (6, 2, true),
            (10, 2, true),
            (12, 2, true),
        ],
        CozyStyle::Bossa => &[(2, 2, true), (6, 2, true), (8, 2, true), (12, 2, true)],
    }
}

fn add_comp(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    let still = matches!(section.character.drums, Drums::None | Drums::Brushes);
    let fingerpicked = section.style == CozyStyle::Acoustic && still;
    let velocity = 0.2 + section.traits.warmth * 0.05 + section.traits.bustle * 0.04;
    let mut previous: Option<Vec<i32>> = None;
    for span in chords {
        let voicing = voice_chord(section.tonic, span.chord, previous.as_deref());
        previous = Some(voicing.clone());
        let span_end = span.start + span.length;
        let mut bar_start = span.start - span.start % section.bar;
        while bar_start < span_end {
            let bar_index = bar_start / section.bar;
            for (step, &(position, length, down)) in
                comp_figure(section.style, section.character.drums, bar_index)
                    .iter()
                    .enumerate()
            {
                let at = bar_start + position * section.sixteenth;
                if !(span.start..span_end).contains(&at) {
                    continue;
                }
                let length = (length * section.sixteenth).min(span_end - at);
                let accent = metric_accent(position) * if down { 1.0 } else { 0.8 };
                if fingerpicked {
                    let pitch = voicing[step % voicing.len()];
                    sink.note(
                        "comp",
                        at,
                        length,
                        velocity * accent,
                        pitch,
                        section.voices.comp,
                        false,
                    );
                    continue;
                }
                // A strum rolls across the strings: down low-to-high, up high-to-low.
                let roll = section.sixteenth / 10;
                let order: Vec<i32> = if down {
                    voicing.clone()
                } else {
                    voicing.iter().rev().copied().collect()
                };
                for (string, pitch) in order.into_iter().enumerate() {
                    let offset = if section.style == CozyStyle::Lofi {
                        0
                    } else {
                        string as u32 * roll
                    };
                    sink.note(
                        "comp",
                        at + offset,
                        length.saturating_sub(offset).max(1),
                        velocity * accent * (1.0 - string as f64 * 0.05),
                        pitch,
                        section.voices.comp,
                        false,
                    );
                }
            }
            bar_start += section.bar;
        }
    }
}

/// A soft held chord under the quiet scenes.
fn add_pad(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    if !section.character.pad {
        return;
    }
    let mut previous: Option<Vec<i32>> = None;
    for span in chords {
        let voicing = voice_chord(section.tonic, span.chord, previous.as_deref());
        previous = Some(voicing.clone());
        for pitch in voicing.iter().take(2) {
            sink.note(
                "pad",
                span.start,
                span.length * 15 / 16,
                0.11 + section.traits.warmth * 0.04,
                pitch - 12,
                section.voices.pad,
                false,
            );
        }
    }
}

/// Bass figures as `(sixteenth, role, length)`; role 0 is the root, 1 the
/// fifth.
fn bass_figure(style: CozyStyle, drums: Drums) -> &'static [(u32, u8, u32)] {
    match (style, drums) {
        (_, Drums::None) => &[(0, 0, 14)],
        (CozyStyle::Acoustic, Drums::Full) => &[(0, 0, 3), (4, 1, 3), (8, 0, 3), (12, 1, 3)],
        (CozyStyle::Acoustic, _) => &[(0, 0, 6), (8, 1, 6)],
        (CozyStyle::Lofi, _) => &[(0, 0, 6), (10, 0, 4)],
        (CozyStyle::Bossa, _) => &[(0, 0, 6), (6, 1, 2), (8, 1, 6), (14, 0, 2)],
    }
}

fn add_bass(sink: &mut EventSink, section: &Section, chords: &[ChordSpan]) {
    let velocity = 0.3 + section.traits.bustle * 0.06;
    let final_bar = section.plan.bars * section.bar - section.bar;
    for (index, span) in chords.iter().enumerate() {
        let root = section.place(span.chord.root, 40, 33, 47);
        let fifth = section.place(span.chord.root + 7, root + 7, root + 5, root + 8);
        let span_end = span.start + span.length;
        let mut bar_start = span.start - span.start % section.bar;
        while bar_start < span_end {
            for &(position, role, length) in bass_figure(section.style, section.character.drums) {
                let at = bar_start + position * section.sixteenth;
                if !(span.start..span_end).contains(&at) {
                    continue;
                }
                let pitch = if role == 0 { root } else { fifth };
                sink.note(
                    "bass",
                    at,
                    (length * section.sixteenth).min(span_end - at),
                    velocity * metric_accent(position),
                    pitch,
                    section.voices.bass,
                    false,
                );
            }
            bar_start += section.bar;
        }
        // Jazz walks into the next root from a semitone below.
        let Some(next) = chords.get(index + 1) else {
            continue;
        };
        let approach_at = next.start - 2 * section.sixteenth;
        let walks = next.chord.root != span.chord.root
            && span.start < approach_at
            && approach_at < final_bar
            && section.character.drums != Drums::None
            && section.chance("approach", index as u32, 0) < section.traits.jazz * 0.7;
        if walks {
            let target = section.place(next.chord.root, root, 33, 47);
            sink.note(
                "bass",
                approach_at,
                2 * section.sixteenth,
                velocity * 0.8,
                target - 1,
                section.voices.bass,
                false,
            );
        }
    }
}

type Hit = (&'static str, u32, f64);

/// A bar of drums as `(voice, sixteenth, weight)`.
fn groove(style: CozyStyle, drums: Drums, bar_index: u32) -> Vec<Hit> {
    let eighths = |voice: &'static str, strong: f64, weak: f64| -> Vec<Hit> {
        (0..16)
            .step_by(2)
            .map(|p| (voice, p, if p % 4 == 0 { strong } else { weak }))
            .collect()
    };
    let mut hits: Vec<Hit> = Vec::new();
    match (style, drums) {
        (_, Drums::None) => {}
        (CozyStyle::Acoustic, Drums::Brushes) => hits.extend(eighths("tambourine", 0.5, 0.35)),
        (CozyStyle::Acoustic, Drums::Light) => {
            hits.extend([
                ("frame-drum", 0, 0.8),
                ("tambourine", 4, 0.7),
                ("tambourine", 12, 0.7),
            ]);
        }
        (CozyStyle::Acoustic, Drums::Groove) => {
            hits.extend([("frame-drum", 0, 0.9), ("frame-drum", 8, 0.75)]);
            hits.extend(eighths("tambourine", 0.7, 0.45));
        }
        (CozyStyle::Acoustic, Drums::Full) => {
            hits.extend([0, 4, 8, 12].map(|p| ("frame-drum", p, if p == 0 { 1.0 } else { 0.8 })));
            hits.extend(eighths("tambourine", 0.8, 0.55));
        }
        (CozyStyle::Lofi, Drums::Brushes) => {
            hits.extend(eighths("hat", 0.45, 0.3));
            hits.extend([("snare", 4, 0.35), ("snare", 12, 0.35)]);
        }
        (CozyStyle::Lofi, Drums::Light) => {
            hits.extend([("kick", 0, 0.8), ("snare", 12, 0.6)]);
            hits.extend(eighths("hat", 0.55, 0.35));
        }
        (CozyStyle::Lofi, _) => {
            hits.extend([
                ("kick", 0, 0.9),
                ("kick", 10, 0.7),
                ("snare", 4, 0.75),
                ("snare", 12, 0.75),
            ]);
            hits.extend(eighths("hat", 0.6, 0.38));
        }
        (CozyStyle::Bossa, Drums::Brushes) => hits.extend(eighths("hat", 0.4, 0.28)),
        (CozyStyle::Bossa, level) => {
            hits.extend(eighths("hat", 0.55, 0.35));
            // The bossa clave, three strokes then two, played as a rim click.
            let clave: &[u32] = if bar_index.is_multiple_of(2) {
                &[0, 6, 12]
            } else {
                &[4, 10]
            };
            hits.extend(clave.iter().map(|p| ("snare", *p, 0.35)));
            if level >= Drums::Groove {
                hits.extend([
                    ("kick", 0, 0.8),
                    ("kick", 6, 0.5),
                    ("kick", 8, 0.8),
                    ("kick", 14, 0.5),
                ]);
            }
            if level == Drums::Full {
                hits.extend((1..16).step_by(2).map(|p| ("tambourine", p, 0.35)));
            }
        }
    }
    hits
}

fn add_drums(sink: &mut EventSink, section: &Section, phrase: &PhrasePlan) {
    let drums = section.character.drums;
    if drums == Drums::None {
        return;
    }
    let base = 0.3 + section.traits.bustle * 0.1;
    let ghosts = 0.03 + section.traits.bustle * 0.25;
    for local_bar in 0..PHRASE_BARS {
        let bar_start = phrase.start + local_bar * section.bar;
        let bar_index = bar_start / section.bar;
        let mut hits = groove(section.style, drums, bar_index);
        if local_bar == PHRASE_BARS - 1 && drums >= Drums::Groove {
            // A turnaround into the next phrase.
            let fill_voice = if section.style == CozyStyle::Acoustic {
                "frame-drum"
            } else {
                "snare"
            };
            hits.retain(|&(voice, p, _)| p < 12 || voice == "hat" || voice == "tambourine");
            hits.extend([12, 13, 14, 15].map(|p| (fill_voice, p, 0.45 + f64::from(p - 12) * 0.12)));
        }
        let ghost_voice = if section.style == CozyStyle::Acoustic {
            "tambourine"
        } else {
            "snare"
        };
        for position in (1..16).step_by(2) {
            if hits.iter().any(|&(_, p, _)| p == position) {
                continue;
            }
            if drums >= Drums::Light && section.chance("ghost", bar_index, position) < ghosts {
                hits.push((ghost_voice, position, 0.22));
            }
        }
        for (voice, position, weight) in hits {
            sink.hit(
                bar_start + position * section.sixteenth,
                section.sixteenth,
                base * weight,
                voice,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_has_a_role_and_an_energy() {
        for plan in SECTION_PLANS {
            assert!(cozy_phase_energy(plan.scene) <= 100);
        }
        assert_eq!(cozy_phase_role(Scene::Dawn), CozyPhaseRole::Intro);
        assert_eq!(cozy_phase_role(Scene::Night), CozyPhaseRole::Outro);
    }

    #[test]
    fn swing_grows_with_the_trait_and_bossa_stays_nearly_straight() {
        let traits = |swing| NormalizedTraits {
            warmth: 0.5,
            bustle: 0.5,
            jazz: 0.5,
            swing,
        };
        for style in [CozyStyle::Acoustic, CozyStyle::Lofi, CozyStyle::Bossa] {
            assert!(swing_point(style, traits(0.0)) < swing_point(style, traits(1.0)));
        }
        assert!(swing_point(CozyStyle::Bossa, traits(1.0)) <= 0.55);
        assert!(swing_point(CozyStyle::Lofi, traits(0.0)) > 0.55);
    }
}
