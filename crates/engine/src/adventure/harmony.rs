//! Modal harmony for Adventure: functional chord choices per church mode, the
//! phrase templates that turn them into a harmonic plan, and voice leading.
//!
//! Everything stays diatonic to the section's mode. Colour comes from the
//! mode's own signature chord (Lydian II, Mixolydian bVII, Dorian IV, Aeolian
//! bVI, Phrygian bII), suspended and open-fifth sonorities, and 4–3
//! suspensions at cadences — never from borrowed accidentals.

use super::composition::{Pace, PhraseKind};
use crate::rng::keyed_unit as chance;
use crate::theory::scale_pitch;

/// The sonority built on a chord's root degree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChordKind {
    Triad,
    Sus2,
    Sus4,
    Open5,
}

impl ChordKind {
    /// Scale-degree offsets from the root for the three voiced notes. Suspended
    /// chords are spread (root, fifth, ninth or eleventh) so their neighbouring
    /// tones never sit a second apart in the accompaniment.
    fn offsets(self) -> [i32; 3] {
        match self {
            Self::Triad => [0, 2, 4],
            Self::Sus2 => [0, 4, 8],
            Self::Sus4 => [0, 4, 10],
            Self::Open5 => [0, 4, 7],
        }
    }

    /// Whether this sonority keeps its character on `degree` of the mode: a
    /// suspension needs a perfect fourth or fifth and a whole step, or it is
    /// just a tritone and a semitone rub (a Lydian "sus4" on the tonic).
    fn fits(self, degree: i32, intervals: &[i32]) -> bool {
        let span = |from: i32, to: i32| semitones(intervals, from, to);
        match self {
            Self::Triad => true,
            Self::Sus2 => span(degree, degree + 1) == 2 && span(degree, degree + 4) == 7,
            Self::Sus4 => span(degree, degree + 3) == 5 && span(degree + 3, degree + 4) == 2,
            Self::Open5 => span(degree, degree + 4) == 7,
        }
    }
}

/// Semitones from scale degree `from` up to scale degree `to`.
fn semitones(intervals: &[i32], from: i32, to: i32) -> i32 {
    scale_pitch(0, to, intervals) - scale_pitch(0, from, intervals)
}

/// Whether two scale degrees sound a semitone (or major seventh) apart.
pub(super) fn rub(intervals: &[i32], a: i32, b: i32) -> bool {
    matches!(semitones(intervals, b, a).rem_euclid(12), 1 | 11)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Chord {
    pub(super) degree: i32,
    pub(super) kind: ChordKind,
}

impl Chord {
    pub(super) const fn triad(degree: i32) -> Self {
        Self {
            degree,
            kind: ChordKind::Triad,
        }
    }

    /// Whether `degree` rubs a semitone against any sounding tone of this chord.
    pub(super) fn rubs(self, degree: i32, intervals: &[i32]) -> bool {
        self.kind
            .offsets()
            .into_iter()
            .any(|offset| rub(intervals, degree, self.degree + offset))
    }

    /// Whether a scale degree is a stable tone of this chord. An open fifth
    /// leaves its third to the melody, so the third still counts as stable.
    pub(super) fn contains(self, degree: i32) -> bool {
        let relative = (degree - self.degree).rem_euclid(7);
        match self.kind {
            ChordKind::Triad | ChordKind::Open5 => matches!(relative, 0 | 2 | 4),
            ChordKind::Sus2 => matches!(relative, 0 | 1 | 4),
            ChordKind::Sus4 => matches!(relative, 0 | 3 | 4),
        }
    }
}

/// A chord sounding from `start` (ticks from the section start) for `length`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ChordSpan {
    pub(super) start: u32,
    pub(super) length: u32,
    pub(super) chord: Chord,
}

/// The chord that sounds at `tick`, if the plan covers it.
pub(super) fn chord_at(plan: &[ChordSpan], tick: u32) -> Option<Chord> {
    plan.iter()
        .find(|span| (span.start..span.start + span.length).contains(&tick))
        .map(|span| span.chord)
}

/// Functional chord degrees for a mode. Every listed degree is a major or
/// minor triad in that mode — the diminished triad is never planted.
struct ModeHarmony {
    /// Chords that prepare the arrival (IV, ii, or the modal equivalent).
    predominants: &'static [i32],
    /// The chord that pulls home: major V, or the modal bVII/bII.
    dominant: i32,
    /// Stable colour chords for the middle of a phrase.
    colors: &'static [i32],
    /// The mode's own signature chord, heard beside the tonic.
    signature: i32,
}

fn mode_harmony(mode: &str) -> ModeHarmony {
    match mode {
        "lydian" => ModeHarmony {
            predominants: &[1, 5],
            dominant: 4,
            colors: &[5, 2],
            signature: 1,
        },
        "mixolydian" => ModeHarmony {
            predominants: &[3, 1],
            dominant: 6,
            colors: &[5, 4],
            signature: 6,
        },
        "dorian" => ModeHarmony {
            predominants: &[3, 1],
            dominant: 6,
            colors: &[2, 4],
            signature: 3,
        },
        "aeolian" => ModeHarmony {
            predominants: &[5, 3],
            dominant: 6,
            colors: &[2, 5],
            signature: 5,
        },
        "phrygian" => ModeHarmony {
            predominants: &[6, 3],
            dominant: 1,
            colors: &[2, 5],
            signature: 1,
        },
        _ => ModeHarmony {
            predominants: &[3, 1],
            dominant: 4,
            colors: &[5, 2],
            signature: 3,
        },
    }
}

/// The scale degree of an open phrase ending (V or a modal turnaround), so
/// melody and accompaniment agree on the half-cadence arrival.
pub(super) fn dominant_degree(mode: &str) -> i32 {
    mode_harmony(mode).dominant
}

/// Harmonic functions a template is written in, resolved per mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Function {
    Tonic,
    Predominant,
    Dominant,
    Color,
}

use Function::{Color as C, Dominant as D, Predominant as P, Tonic as T};

/// One bar of a template: a single chord, or two chords split at beat three.
type Bar = &'static [Function];
type Template = [Bar; 4];

const ANTECEDENT: &[Template] = &[
    [&[T], &[P], &[D], &[D]],
    [&[T], &[C], &[P], &[D]],
    [&[T], &[D], &[C], &[D]],
];
const CONSEQUENT: &[Template] = &[
    [&[T], &[C], &[P, D], &[T]],
    [&[T], &[P], &[C, D], &[T]],
    [&[T], &[D], &[P, D], &[T]],
];
const DEVELOPMENT: &[Template] = &[
    [&[C], &[P], &[C], &[D]],
    [&[P], &[C], &[P], &[D]],
    [&[C], &[D], &[P], &[D]],
];
const CADENCE: &[Template] = &[[&[P], &[C], &[P, D], &[T]], [&[C], &[P], &[D], &[T]]];

/// Inputs that shape one phrase's harmony.
pub(super) struct PhraseHarmonyInput<'a> {
    pub(super) mode: &'a str,
    pub(super) intervals: &'a [i32],
    pub(super) kind: PhraseKind,
    pub(super) pace: Pace,
    pub(super) phrase_start: u32,
    pub(super) bar_ticks: u32,
    /// Which template family member this section's statements use, so a
    /// return restates the consequent's harmony instead of inventing new.
    pub(super) statement_variant: usize,
    pub(super) seed: u32,
    pub(super) phrase: u32,
    pub(super) danger: f64,
    pub(super) mystery: f64,
    /// Dark prefers bare fifths to suspensions for its colour chords.
    pub(super) open_color: bool,
    /// Whether this style bows a 4–3 suspension into its cadences.
    pub(super) suspends: bool,
}

/// The harmonic plan of one four-bar phrase: chord spans in section ticks.
pub(super) fn phrase_harmony(input: &PhraseHarmonyInput) -> Vec<ChordSpan> {
    let harmony = mode_harmony(input.mode);
    let template = choose_template(input);
    let bars = pace_bars(template, input.kind, input.pace);
    let resolve = |function: Function, slot: u32| match function {
        Function::Tonic => 0,
        Function::Dominant => harmony.dominant,
        Function::Predominant => {
            let pick = chance(input.seed, "predominant", input.phrase, slot);
            harmony.predominants[usize::from(pick >= 0.62)]
        }
        Function::Color => {
            // Danger tips the colour toward the darker of the two choices.
            let pick = chance(input.seed, "color", input.phrase, slot);
            harmony.colors[usize::from(pick < 0.3 + input.danger * 0.5)]
        }
    };
    let half = input.bar_ticks / 2;
    let mut spans = Vec::new();
    for (bar_index, bar) in bars.iter().enumerate() {
        let bar_start = input.phrase_start + bar_index as u32 * input.bar_ticks;
        let chords: Vec<i32> = bar
            .iter()
            .enumerate()
            .map(|(slot, function)| resolve(*function, bar_index as u32 * 2 + slot as u32))
            .collect();
        match chords.as_slice() {
            [single] => spans.push(ChordSpan {
                start: bar_start,
                length: input.bar_ticks,
                chord: Chord::triad(*single),
            }),
            [first, second] => {
                spans.push(ChordSpan {
                    start: bar_start,
                    length: half,
                    chord: Chord::triad(*first),
                });
                spans.push(ChordSpan {
                    start: bar_start + half,
                    length: half,
                    chord: Chord::triad(*second),
                });
            }
            _ => unreachable!("a template bar holds one or two chords"),
        }
    }
    if input.pace == Pace::Driving {
        let arrival_bar = input.phrase_start + 3 * input.bar_ticks;
        add_signature_neighbours(&mut spans, harmony.signature, half, arrival_bar);
    }
    merge_repeats(&mut spans);
    color_chords(&mut spans, input);
    if input.suspends && ChordKind::Sus4.fits(harmony.dominant, input.intervals) {
        suspend_cadence(&mut spans, input.bar_ticks / 4, harmony.dominant);
    }
    spans
}

fn choose_template(input: &PhraseHarmonyInput) -> &'static Template {
    let family = match input.kind {
        PhraseKind::Antecedent => ANTECEDENT,
        PhraseKind::Consequent | PhraseKind::Return => CONSEQUENT,
        PhraseKind::Development => DEVELOPMENT,
        PhraseKind::Cadence => CADENCE,
    };
    // Statements keep the section's variant so a return is heard as a return;
    // developments and cadences pick their own departure.
    let index = match input.kind {
        PhraseKind::Antecedent | PhraseKind::Consequent | PhraseKind::Return => {
            input.statement_variant
        }
        PhraseKind::Development | PhraseKind::Cadence => {
            (chance(input.seed, "template", input.phrase, 0) * family.len() as f64) as usize
        }
    };
    &family[index % family.len()]
}

/// Harmonic rhythm by pace. Calm phases hold a chord for two bars; walking and
/// driving phases change every bar.
fn pace_bars(template: &'static Template, kind: PhraseKind, pace: Pace) -> [Bar; 4] {
    if pace != Pace::Calm {
        return *template;
    }
    let first: Bar = &template[0][..1];
    let closing = matches!(
        kind,
        PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Cadence
    );
    if closing {
        [first, first, template[2], template[3]]
    } else {
        [first, first, template[3], template[3]]
    }
}

/// Driving phases rock between the tonic and the mode's signature chord on the
/// second half of each tonic bar — the modal riff under a battle theme. The
/// arrival bar is left alone so the phrase still lands where it promised.
fn add_signature_neighbours(
    spans: &mut Vec<ChordSpan>,
    signature: i32,
    half: u32,
    arrival_bar: u32,
) {
    let mut result = Vec::with_capacity(spans.len() + 2);
    for span in spans.drain(..) {
        if span.chord.degree == 0 && span.length > half && span.start < arrival_bar {
            result.push(ChordSpan {
                length: half,
                ..span
            });
            result.push(ChordSpan {
                start: span.start + half,
                length: span.length - half,
                chord: Chord::triad(signature),
            });
        } else {
            result.push(span);
        }
    }
    *spans = result;
}

fn merge_repeats(spans: &mut Vec<ChordSpan>) {
    let mut merged: Vec<ChordSpan> = Vec::with_capacity(spans.len());
    for span in spans.drain(..) {
        match merged.last_mut() {
            Some(last) if last.chord == span.chord && last.start + last.length == span.start => {
                last.length += span.length;
            }
            _ => merged.push(span),
        }
    }
    *spans = merged;
}

/// Mystery clouds the non-cadential chords into suspended or bare-fifth
/// sonorities. The arrival chord of the phrase always stays a plain triad.
fn color_chords(spans: &mut [ChordSpan], input: &PhraseHarmonyInput) {
    let threshold = 0.06 + input.mystery * 0.45;
    let last = spans.len().saturating_sub(1);
    for (index, span) in spans.iter_mut().enumerate() {
        if index == last {
            continue;
        }
        let draw = chance(input.seed, "cloud", input.phrase, index as u32);
        if draw >= threshold {
            continue;
        }
        let kind = if input.open_color {
            ChordKind::Open5
        } else if draw < threshold / 2.0 {
            ChordKind::Sus2
        } else {
            ChordKind::Sus4
        };
        if kind.fits(span.chord.degree, input.intervals) {
            span.chord.kind = kind;
        }
    }
}

/// A 4–3 suspension: the dominant before a closing tonic enters with its
/// fourth held over and resolves to the third a beat later.
fn suspend_cadence(spans: &mut Vec<ChordSpan>, beat: u32, dominant: i32) {
    let Some(index) = spans.len().checked_sub(2) else {
        return;
    };
    let (penultimate, last) = (spans[index], spans[index + 1]);
    if penultimate.chord.degree != dominant
        || last.chord.degree != 0
        || penultimate.chord.kind != ChordKind::Triad
        || penultimate.length < beat * 2
    {
        return;
    }
    spans[index] = ChordSpan {
        length: beat,
        chord: Chord {
            degree: dominant,
            kind: ChordKind::Sus4,
        },
        ..penultimate
    };
    spans.insert(
        index + 1,
        ChordSpan {
            start: penultimate.start + beat,
            length: penultimate.length - beat,
            chord: penultimate.chord,
        },
    );
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

/// The pitch of `degree` nearest `target` inside `minimum..=maximum`.
pub(super) fn place_degree(
    tonic_pitch_class: i32,
    degree: i32,
    intervals: &[i32],
    target: i32,
    minimum: i32,
    maximum: i32,
) -> i32 {
    nearest_scale_pitch(
        tonic_pitch_class,
        degree,
        intervals,
        target,
        minimum,
        maximum,
    )
}

/// Voice a chord in three ordered voices: of every spacing of its tones in
/// the accompaniment register, the one whose voices move least (squared, so
/// no single voice leaps) from the previous voicing.
pub(super) fn voice_chord(
    tonic_pitch_class: i32,
    chord: Chord,
    intervals: &[i32],
    previous: Option<[i32; 3]>,
) -> [i32; 3] {
    const CEILINGS: [i32; 3] = [60, 72, 84];
    let targets = previous.unwrap_or([50, 58, 65]);
    let candidates = |voice: usize, offset: i32| -> Vec<i32> {
        let base = scale_pitch(60 + tonic_pitch_class, chord.degree + offset, intervals);
        (-3..=2)
            .map(|octave| base + octave * 12)
            .filter(|pitch| (40..=CEILINGS[voice]).contains(pitch))
            .collect()
    };
    let [a, b, c] = chord.kind.offsets();
    let mut best: Option<(i32, [i32; 3])> = None;
    for low in candidates(0, a) {
        for middle in candidates(1, b) {
            for high in candidates(2, c) {
                if middle < low + 3 || high < middle + 3 {
                    continue;
                }
                let voicing = [low, middle, high];
                let motion: i32 = voicing
                    .iter()
                    .zip(targets)
                    .map(|(p, t)| (p - t).pow(2))
                    .sum();
                if best.is_none_or(|(least, _)| motion < least) {
                    best = Some((motion, voicing));
                }
            }
        }
    }
    best.expect("three chord tones always fit the accompaniment register")
        .1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::mode_intervals;

    const MODES: [&str; 6] = [
        "ionian",
        "dorian",
        "phrygian",
        "lydian",
        "mixolydian",
        "aeolian",
    ];

    fn triad_quality(mode: &str, degree: i32) -> &'static str {
        let intervals = mode_intervals(mode);
        let root = intervals[degree.rem_euclid(7) as usize];
        let third = intervals[(degree + 2).rem_euclid(7) as usize];
        let fifth = intervals[(degree + 4).rem_euclid(7) as usize];
        match ((third - root).rem_euclid(12), (fifth - root).rem_euclid(12)) {
            (4, 7) => "major",
            (3, 7) => "minor",
            (3, 6) => "diminished",
            _ => "other",
        }
    }

    fn plans(mode: &str) -> Vec<(PhraseKind, Pace, Vec<ChordSpan>)> {
        let mut out = Vec::new();
        for kind in [
            PhraseKind::Antecedent,
            PhraseKind::Consequent,
            PhraseKind::Development,
            PhraseKind::Return,
            PhraseKind::Cadence,
        ] {
            for pace in [Pace::Calm, Pace::Walking, Pace::Driving] {
                for seed in 0..12 {
                    for mystery in [0.0, 0.5, 1.0] {
                        let spans = phrase_harmony(&PhraseHarmonyInput {
                            mode,
                            intervals: &mode_intervals(mode),
                            kind,
                            pace,
                            phrase_start: 0,
                            bar_ticks: 3840,
                            statement_variant: seed as usize,
                            seed,
                            phrase: seed,
                            danger: f64::from(seed % 3) / 2.0,
                            mystery,
                            open_color: seed % 2 == 0,
                            suspends: seed % 3 == 0,
                        });
                        out.push((kind, pace, spans));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn plans_tile_the_phrase_and_never_plant_a_diminished_root() {
        for mode in MODES {
            for (kind, pace, spans) in plans(mode) {
                let mut cursor = 0;
                for span in &spans {
                    assert_eq!(span.start, cursor, "{mode} {kind:?} {pace:?} gap");
                    cursor += span.length;
                    assert_ne!(
                        triad_quality(mode, span.chord.degree),
                        "diminished",
                        "{mode} {kind:?} planted a diminished root"
                    );
                }
                assert_eq!(cursor, 4 * 3840, "{mode} {kind:?} {pace:?} span");
            }
        }
    }

    #[test]
    fn phrases_arrive_where_their_kind_promises() {
        for mode in MODES {
            for (kind, _, spans) in plans(mode) {
                let last = spans.last().unwrap().chord;
                assert_eq!(
                    last.kind,
                    ChordKind::Triad,
                    "{mode} {kind:?} arrival clouded"
                );
                let expected = match kind {
                    PhraseKind::Antecedent | PhraseKind::Development => dominant_degree(mode),
                    _ => 0,
                };
                assert_eq!(last.degree, expected, "{mode} {kind:?} arrival");
            }
        }
    }

    #[test]
    fn warm_modes_arrive_on_major_chords() {
        for mode in ["ionian", "lydian", "mixolydian"] {
            for (kind, _, spans) in plans(mode) {
                let arrival = spans.last().unwrap().chord.degree;
                assert_eq!(triad_quality(mode, arrival), "major", "{mode} {kind:?}");
            }
        }
    }

    #[test]
    fn calm_phases_change_chords_less_often_than_driving_phases() {
        for mode in MODES {
            let count = |pace: Pace| {
                plans(mode)
                    .into_iter()
                    .filter(|(_, p, _)| *p == pace)
                    .map(|(_, _, spans)| spans.len())
                    .sum::<usize>()
            };
            assert!(count(Pace::Calm) < count(Pace::Walking), "{mode}");
            assert!(count(Pace::Walking) < count(Pace::Driving), "{mode}");
        }
    }

    #[test]
    fn mystery_clouds_more_chords() {
        for mode in MODES {
            let clouded = |mystery: f64| {
                (0..40)
                    .map(|seed| {
                        phrase_harmony(&PhraseHarmonyInput {
                            mode,
                            intervals: &mode_intervals(mode),
                            kind: PhraseKind::Consequent,
                            pace: Pace::Walking,
                            phrase_start: 0,
                            bar_ticks: 3840,
                            statement_variant: 0,
                            seed,
                            phrase: 1,
                            danger: 0.5,
                            mystery,
                            open_color: false,
                            suspends: false,
                        })
                        .iter()
                        .filter(|span| span.chord.kind != ChordKind::Triad)
                        .count()
                    })
                    .sum::<usize>()
            };
            assert!(clouded(0.1) < clouded(0.9), "{mode}");
        }
    }

    #[test]
    fn a_suspension_resolves_inside_the_dominant() {
        let spans = phrase_harmony(&PhraseHarmonyInput {
            mode: "ionian",
            intervals: &mode_intervals("ionian"),
            kind: PhraseKind::Cadence,
            pace: Pace::Walking,
            phrase_start: 0,
            bar_ticks: 3840,
            statement_variant: 0,
            seed: 3,
            phrase: 0,
            danger: 0.0,
            mystery: 0.0,
            open_color: false,
            suspends: true,
        });
        let sus = spans
            .iter()
            .position(|span| span.chord.kind == ChordKind::Sus4)
            .expect("the cadence carries a 4-3 suspension");
        assert_eq!(spans[sus + 1].chord, Chord::triad(4));
        assert_eq!(spans[sus + 2].chord, Chord::triad(0));
    }

    #[test]
    fn voicings_are_ordered_and_move_by_small_steps() {
        for tonic in [0, 2, 3, 5, 7, 9, 10] {
            for mode in MODES {
                let intervals = mode_intervals(mode);
                let mut previous = None;
                for (degree, kind) in [
                    (0, ChordKind::Triad),
                    (3, ChordKind::Sus4),
                    (1, ChordKind::Triad),
                    (4, ChordKind::Sus2),
                    (0, ChordKind::Open5),
                    (5, ChordKind::Triad),
                    (4, ChordKind::Triad),
                    (0, ChordKind::Triad),
                ] {
                    let chord = voice_chord(tonic, Chord { degree, kind }, &intervals, previous);
                    assert!(chord[0] < chord[1] && chord[1] < chord[2]);
                    if let Some(old) = previous {
                        for voice in 0..3 {
                            assert!(
                                (chord[voice] - old[voice]).abs() <= 9,
                                "{mode} {degree} voice {voice} leapt {} -> {}",
                                old[voice],
                                chord[voice]
                            );
                        }
                    }
                    previous = Some(chord);
                }
            }
        }
    }
}
