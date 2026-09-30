//! Adventure's melodic material: one quest theme per piece and the ways each
//! scene states, sequences, augments, fragments or quickens it.
//!
//! Lines are written in scale-degree space (`degree` 7 is the octave) on a
//! sixteenth-note grid, so a theme keeps its shape in every mode and the
//! caller maps it onto pitches once. Harmony fitting is the only place a line
//! bends: a strong beat lands on a chord tone, a weak beat may pass or turn by
//! step, and anything else is pulled to the nearest chord tone.

use super::composition::PhraseKind;
use super::harmony::{chord_at, Chord, ChordSpan};
use crate::rng::{keyed_unit, DeterministicRandom};

/// Sixteenths in a bar and in a four-bar phrase.
pub(super) const BAR: u32 = 16;
pub(super) const PHRASE: u32 = 4 * BAR;
/// The arrival note lands on the last bar's downbeat and holds three beats,
/// leaving the fourth beat for a breath or a pickup.
const ARRIVAL_AT: u32 = 3 * BAR;
const ARRIVAL_LENGTH: u32 = 12;

/// One note of a line: onset and length in sixteenths from the phrase start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Tone {
    pub(super) at: u32,
    pub(super) length: u32,
    pub(super) degree: i32,
}

/// Two-bar theme rhythms as `(onset, length)` in sixteenths. Each ends on a
/// long note so the theme reads as a question the phrase then answers.
const THEME_RHYTHMS: [&[(u32, u32)]; 8] = [
    // Heroic dotted call.
    &[
        (0, 6),
        (6, 2),
        (8, 4),
        (12, 4),
        (16, 4),
        (20, 2),
        (22, 2),
        (24, 8),
    ],
    // Ballad.
    &[(0, 4), (4, 4), (8, 6), (14, 2), (16, 8), (24, 8)],
    // Dance with running eighths.
    &[
        (0, 2),
        (2, 2),
        (4, 4),
        (8, 2),
        (10, 2),
        (12, 4),
        (16, 4),
        (20, 4),
        (24, 8),
    ],
    // Broad horn call.
    &[(0, 8), (8, 4), (12, 4), (16, 6), (22, 2), (24, 8)],
    // Scotch snap, a folk lilt.
    &[
        (0, 1),
        (1, 3),
        (4, 4),
        (8, 4),
        (12, 4),
        (16, 1),
        (17, 3),
        (20, 4),
        (24, 8),
    ],
    // March.
    &[
        (0, 4),
        (4, 2),
        (6, 2),
        (8, 4),
        (12, 4),
        (16, 4),
        (20, 2),
        (22, 2),
        (24, 8),
    ],
    // Lyrical syncopation.
    &[(0, 6), (6, 6), (12, 4), (16, 4), (20, 4), (24, 8)],
    // Rising run to a held note.
    &[
        (0, 2),
        (2, 2),
        (4, 2),
        (6, 2),
        (8, 8),
        (16, 4),
        (20, 4),
        (24, 8),
    ],
];

/// Two-bar continuation rhythms (bars three and four) from sparse to busy.
/// Each ends with the arrival on the fourth bar's downbeat.
const CONTINUATIONS: [&[(u32, u32)]; 6] = [
    &[(0, 8), (8, 4), (12, 4), (16, ARRIVAL_LENGTH)],
    &[(0, 6), (6, 2), (8, 8), (16, ARRIVAL_LENGTH)],
    &[(0, 4), (4, 4), (8, 4), (12, 4), (16, ARRIVAL_LENGTH)],
    &[(0, 6), (6, 2), (8, 4), (12, 4), (16, ARRIVAL_LENGTH)],
    &[
        (0, 3),
        (3, 1),
        (4, 4),
        (8, 3),
        (11, 1),
        (12, 4),
        (16, ARRIVAL_LENGTH),
    ],
    &[
        (0, 2),
        (2, 2),
        (4, 4),
        (8, 2),
        (10, 2),
        (12, 4),
        (16, ARRIVAL_LENGTH),
    ],
];

/// Driving continuations: a bar of running figures into the arrival.
const DRIVING_CONTINUATIONS: [&[(u32, u32)]; 3] = [
    &[
        (0, 2),
        (2, 2),
        (4, 2),
        (6, 2),
        (8, 2),
        (10, 2),
        (12, 4),
        (16, ARRIVAL_LENGTH),
    ],
    &[
        (0, 3),
        (3, 1),
        (4, 2),
        (6, 2),
        (8, 3),
        (11, 1),
        (12, 4),
        (16, ARRIVAL_LENGTH),
    ],
    &[
        (0, 2),
        (2, 1),
        (3, 1),
        (4, 4),
        (8, 2),
        (10, 1),
        (11, 1),
        (12, 4),
        (16, ARRIVAL_LENGTH),
    ],
];

/// The piece's quest theme, chosen from many candidate contours by how well it
/// sings: mostly steps, one climax, leaps recovered by step, a hummable range.
pub(super) fn compose_theme(rng: &mut DeterministicRandom) -> Vec<Tone> {
    let rhythm = *rng.pick(&THEME_RHYTHMS);
    let mut best: Option<(f64, Vec<i32>)> = None;
    for _ in 0..48 {
        let degrees = candidate_contour(rng, rhythm.len());
        let score = theme_quality(&degrees);
        if best.as_ref().is_none_or(|(top, _)| score > *top) {
            best = Some((score, degrees));
        }
    }
    let (_, degrees) = best.expect("at least one candidate");
    rhythm
        .iter()
        .zip(degrees)
        .map(|(&(at, length), degree)| Tone { at, length, degree })
        .collect()
}

fn candidate_contour(rng: &mut DeterministicRandom, notes: usize) -> Vec<i32> {
    let mut degrees = vec![*rng.pick(&[0, 2, 4, 4, 2])];
    let mut last_move: i32 = 0;
    for _ in 1..notes - 1 {
        let current = *degrees.last().unwrap();
        // Lean back toward the middle of the range so lines do not wander off.
        let upward = rng.next() < 0.5 + f64::from(2 - current.clamp(-2, 6)) * 0.08;
        let direction = if upward { 1 } else { -1 };
        let step = if last_move.abs() >= 3 {
            -last_move.signum()
        } else {
            let draw = rng.next();
            if draw < 0.56 {
                direction
            } else if draw < 0.66 {
                0
            } else if draw < 0.88 {
                direction * 2
            } else {
                direction * *rng.pick(&[3, 4])
            }
        };
        degrees.push(current + step);
        last_move = step;
    }
    // End on an open degree (the fifth or the second) near the last note.
    let current = *degrees.last().unwrap();
    let ending = [4i32, 1, 11, 8, -3, -6]
        .into_iter()
        .min_by_key(|degree| ((degree - current).abs(), *degree))
        .unwrap();
    degrees.push(ending);
    degrees
}

fn theme_quality(degrees: &[i32]) -> f64 {
    let moves: Vec<i32> = degrees.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let steps = moves.iter().filter(|m| m.abs() == 1).count() as f64;
    let repeats = moves.iter().filter(|m| **m == 0).count() as f64;
    let mut score = steps / moves.len() as f64 * 4.0 - repeats * 0.6;
    for pair in moves.windows(2) {
        if pair[0].abs() >= 3 && !(pair[1].abs() <= 2 && pair[1].signum() == -pair[0].signum()) {
            score -= 1.5;
        }
    }
    let high = *degrees.iter().max().unwrap();
    let low = *degrees.iter().min().unwrap();
    let range = f64::from(high - low);
    score -= (range - 5.5).abs() * 0.4;
    if range > 8.0 {
        score -= 3.0;
    }
    let peak_count = degrees.iter().filter(|d| **d == high).count();
    if peak_count == 1 {
        score += 1.5;
        let peak = degrees.iter().position(|d| *d == high).unwrap();
        if peak * 3 >= degrees.len() && peak < degrees.len() - 1 {
            score += 0.5;
        }
    }
    if moves.iter().any(|m| *m > 0) && moves.iter().any(|m| *m < 0) {
        score += 0.5;
    }
    score
}

/// How a scene states the theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Treatment {
    /// The theme as written, answered by a continuation.
    Statement,
    /// The theme's head at half speed: broad and still.
    Augmented,
    /// Only the head, in the dark, with long silences.
    Fragment,
    /// The whole theme in one bar, then sequenced: urgent.
    Diminution,
}

/// Everything a phrase melody needs besides the theme itself.
pub(super) struct PhraseMelodyInput<'a> {
    pub(super) theme: &'a [Tone],
    pub(super) intervals: &'a [i32],
    pub(super) kind: PhraseKind,
    pub(super) treatment: Treatment,
    pub(super) dominant: i32,
    /// The phrase's harmony in section ticks, starting at `phrase_start`.
    pub(super) chords: &'a [ChordSpan],
    pub(super) phrase_start: u32,
    pub(super) sixteenth: u32,
    pub(super) seed: u32,
    pub(super) phrase: u32,
    pub(super) motion: f64,
    pub(super) mystery: f64,
    /// A pickup leads into the next phrase's theme unless the section ends.
    pub(super) pickup: bool,
}

/// The melody of one four-bar phrase in degree space.
pub(super) fn phrase_melody(input: &PhraseMelodyInput) -> Vec<Tone> {
    let mut tones = opening(input);
    let from = tones
        .last()
        .map_or(input.theme[0].degree, |tone| tone.degree);
    let target = match input.kind {
        PhraseKind::Antecedent | PhraseKind::Development => input.dominant,
        PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Cadence => 0,
    };
    tones.extend(continuation(input, from, target));
    fit_to_harmony(&mut tones, input);
    thin_for_mystery(&mut tones, input);
    if input.pickup {
        add_pickup(&mut tones, input);
    }
    tones
}

/// Bars one and two: the theme, transformed by treatment and phrase role.
fn opening(input: &PhraseMelodyInput) -> Vec<Tone> {
    let theme = input.theme;
    let head: Vec<Tone> = theme.iter().copied().filter(|tone| tone.at < BAR).collect();
    let tail: Vec<Tone> = theme
        .iter()
        .copied()
        .filter(|tone| tone.at >= BAR)
        .collect();
    let rise = if keyed_unit(input.seed, "sequence", input.phrase, 0) < 0.6 {
        1
    } else {
        2
    };
    match (input.treatment, input.kind) {
        (Treatment::Statement, PhraseKind::Development) => {
            let mut tones = shift(&head, 0, rise);
            tones.extend(shift(&head, BAR, 2 * rise));
            tones
        }
        (Treatment::Statement, PhraseKind::Cadence) => {
            let mut tones = shift(&tail, 0, 1)
                .into_iter()
                .map(|t| Tone {
                    at: t.at - BAR,
                    ..t
                })
                .collect::<Vec<_>>();
            tones.extend(shift(&tail, 0, 0));
            tones
        }
        (Treatment::Statement, _) => theme.to_vec(),
        (Treatment::Augmented, kind) => {
            let (source, transpose) = match kind {
                PhraseKind::Development => (&tail, rise),
                PhraseKind::Cadence => (&head, -1),
                _ => (&head, 0),
            };
            let origin = source.first().map_or(0, |tone| tone.at);
            source
                .iter()
                .map(|tone| Tone {
                    at: (tone.at - origin) * 2,
                    length: (tone.length * 2).min(2 * BAR - (tone.at - origin) * 2),
                    degree: tone.degree + transpose,
                })
                .collect()
        }
        (Treatment::Fragment, kind) => {
            let fragment: Vec<Tone> = head.iter().copied().take(3).collect();
            let transpose = match kind {
                PhraseKind::Development => -1,
                PhraseKind::Cadence => -2,
                _ => 0,
            };
            let mut tones = shift(&fragment, 0, transpose);
            // An echo a step lower answers the head across the silence.
            if matches!(
                kind,
                PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Development
            ) {
                tones.extend(shift(&fragment, BAR, transpose - 1));
            } else if let Some(last) = fragment.last() {
                tones.push(Tone {
                    at: BAR + 4,
                    length: 8,
                    degree: last.degree + transpose - 1,
                });
            }
            tones
        }
        (Treatment::Diminution, kind) => {
            let quick = diminish(theme);
            let answer = match kind {
                PhraseKind::Antecedent | PhraseKind::Return => 0,
                PhraseKind::Consequent => rise,
                PhraseKind::Development => 2 * rise,
                PhraseKind::Cadence => -1,
            };
            let mut tones = quick.clone();
            tones.extend(shift(&quick, BAR, answer));
            tones
        }
    }
}

/// The theme at double speed in one bar. Onsets that collapse onto the same
/// sixteenth keep the first note, and each note lasts until the next.
fn diminish(theme: &[Tone]) -> Vec<Tone> {
    let mut quick: Vec<Tone> = Vec::with_capacity(theme.len());
    for tone in theme {
        let at = tone.at / 2;
        if quick.last().is_some_and(|previous| previous.at == at) {
            continue;
        }
        quick.push(Tone {
            at,
            length: (tone.length / 2).max(1),
            degree: tone.degree,
        });
    }
    for index in 1..quick.len() {
        let next_at = quick[index].at;
        let previous = &mut quick[index - 1];
        previous.length = previous.length.min(next_at - previous.at);
    }
    quick
}

fn shift(tones: &[Tone], offset: u32, transpose: i32) -> Vec<Tone> {
    tones
        .iter()
        .map(|tone| Tone {
            at: tone.at + offset,
            length: tone.length,
            degree: tone.degree + transpose,
        })
        .collect()
}

/// Bars three and four: a line from where the opening stopped to a stepwise
/// approach and the arrival. Tonic arrivals descend onto home (2 → 1), open
/// arrivals rise into the dominant, so question and answer have opposite arcs.
fn continuation(input: &PhraseMelodyInput, from: i32, target: i32) -> Vec<Tone> {
    let bank: &[&[(u32, u32)]] = if input.treatment == Treatment::Diminution {
        &DRIVING_CONTINUATIONS
    } else {
        &CONTINUATIONS
    };
    let draw = keyed_unit(input.seed, "continuation", input.phrase, 0);
    let index = ((draw * 0.55 + input.motion * 0.45) * bank.len() as f64) as usize;
    let rhythm = bank[index.min(bank.len() - 1)];

    let arrival = nearest_octave(target, from);
    let approach = if target == 0 {
        arrival + 1
    } else {
        arrival - 1
    };
    let moving = rhythm.len() - 1;
    let distance = approach - from;
    // When the gap is small the line still travels: it arches over the top
    // into a home arrival and dips beneath an open one.
    let spare = (moving as i32 - 1 - distance.abs()).max(0);
    let bump = (spare / 2).min(2) * if target == 0 { 1 } else { -1 };
    let mut tones: Vec<Tone> = rhythm
        .iter()
        .take(moving)
        .enumerate()
        .map(|(index, &(at, length))| {
            let progress = (index + 1) as f64 / moving as f64;
            let straight = f64::from(from) + f64::from(distance) * progress;
            let arch = (std::f64::consts::PI * progress).sin() * f64::from(bump);
            Tone {
                at: 2 * BAR + at,
                length,
                degree: (straight + arch).round() as i32,
            }
        })
        .collect();
    if let Some(last) = tones.last_mut() {
        last.degree = approach;
    }
    let (at, length) = rhythm[moving];
    tones.push(Tone {
        at: 2 * BAR + at,
        length,
        degree: arrival,
    });
    tones
}

fn nearest_octave(degree: i32, near: i32) -> i32 {
    (-3..=3)
        .map(|octave| degree + octave * 7)
        .min_by_key(|candidate| ((candidate - near).abs(), *candidate))
        .unwrap()
}

fn chord_for(input: &PhraseMelodyInput, at: u32) -> Option<Chord> {
    chord_at(input.chords, input.phrase_start + at * input.sixteenth)
}

/// Bend the line onto the harmony: strong beats and long notes are chord
/// tones; a weak note may stay only as a stepwise passing or neighbour tone,
/// and one that rubs a semitone against the chord only as a short off-beat
/// note passing through. The arrival is never moved — it is the chord tone the
/// harmony was built for.
fn fit_to_harmony(tones: &mut [Tone], input: &PhraseMelodyInput) {
    for index in 0..tones.len() {
        let tone = tones[index];
        if tone.at == ARRIVAL_AT {
            continue;
        }
        let Some(chord) = chord_for(input, tone.at) else {
            continue;
        };
        if chord.contains(tone.degree) {
            continue;
        }
        let strong = tone.at.is_multiple_of(8) || tone.length >= 6;
        let previous = index.checked_sub(1).map(|i| tones[i].degree);
        let next = tones.get(index + 1).map(|t| t.degree);
        let by_step = |other: Option<i32>| other.is_some_and(|o| (o - tone.degree).abs() <= 1);
        let allowed = if chord.rubs(tone.degree, input.intervals) {
            !tone.at.is_multiple_of(4) && tone.length <= 2 && by_step(previous) && by_step(next)
        } else {
            !strong && (by_step(previous) || by_step(next))
        };
        if allowed {
            continue;
        }
        let reference = previous.unwrap_or(tone.degree);
        let rising = tone.degree >= reference;
        tones[index].degree = [1, -1, 2, -2]
            .into_iter()
            .map(|offset| tone.degree + offset)
            .filter(|candidate| chord.contains(*candidate))
            .min_by_key(|candidate| {
                let keeps_contour = (*candidate >= reference) == rising;
                (
                    i32::from(!keeps_contour),
                    (candidate - tone.degree).abs(),
                    *candidate,
                )
            })
            .unwrap_or(tone.degree);
    }
}

/// Mystery opens silences in the continuation: short weak notes drop away as
/// the knob climbs. The theme, downbeats and the arrival always remain.
fn thin_for_mystery(tones: &mut Vec<Tone>, input: &PhraseMelodyInput) {
    let threshold = (input.mystery - 0.4).max(0.0) * 0.55;
    let mut index = 0u32;
    tones.retain(|tone| {
        index += 1;
        let structural =
            tone.at < 2 * BAR || tone.at % BAR == 0 || tone.at == ARRIVAL_AT || tone.length >= 4;
        structural || keyed_unit(input.seed, "mystery-rest", input.phrase, index) >= threshold
    });
}

/// One or two notes on the last beat that step into the next phrase's first
/// note, so the loop and the phrase seams sing through instead of stopping.
/// The pickup sounds over the held arrival chord, so it never rubs it: it
/// approaches from below, or from above when the step below would rub.
fn add_pickup(tones: &mut Vec<Tone>, input: &PhraseMelodyInput) {
    let Some(arrival) = tones.iter().find(|tone| tone.at == ARRIVAL_AT) else {
        return;
    };
    let Some(chord) = chord_for(input, PHRASE - 4) else {
        return;
    };
    let target = nearest_octave(input.theme[0].degree, arrival.degree);
    let clean = |degree: i32| !chord.rubs(degree, input.intervals);
    let Some(side) = [-1, 1].into_iter().find(|side| clean(target + side)) else {
        return;
    };
    let run = keyed_unit(input.seed, "pickup", input.phrase, 0) >= 0.5 && clean(target + 2 * side);
    if run {
        tones.push(Tone {
            at: PHRASE - 4,
            length: 2,
            degree: target + 2 * side,
        });
    }
    tones.push(Tone {
        at: PHRASE - 2,
        length: 2,
        degree: target + side,
    });
}

/// A second voice under the melody: mostly thirds and sixths below on the
/// strong beats, holding while the melody runs and moving while it holds.
/// `clashes` reports whether two degrees would sound a semitone apart.
pub(super) fn counter_line(
    melody: &[Tone],
    chords: &[ChordSpan],
    phrase_start: u32,
    sixteenth: u32,
    clashes: impl Fn(i32, i32) -> bool,
) -> Vec<Tone> {
    let mut line = Vec::new();
    for window in 0..8u32 {
        let start = window * 8;
        let end = start + 8;
        let Some(chord) = chord_at(chords, phrase_start + start * sixteenth) else {
            continue;
        };
        let reference = melody
            .iter()
            .filter(|tone| tone.at <= start && start < tone.at + tone.length)
            .chain(melody.iter().filter(|tone| (start..end).contains(&tone.at)))
            .next();
        let Some(reference) = reference else {
            continue;
        };
        let inside: Vec<&Tone> = melody
            .iter()
            .filter(|tone| tone.at < end && tone.at + tone.length > start)
            .collect();
        let safe = |degree: i32| {
            chord.contains(degree)
                && inside
                    .iter()
                    .all(|tone| tone.degree - degree >= 2 && !clashes(tone.degree, degree))
        };
        let Some(first) = [2, 5, 3, 4]
            .into_iter()
            .map(|below| reference.degree - below)
            .find(|degree| safe(*degree))
        else {
            continue;
        };
        let onsets = melody
            .iter()
            .filter(|tone| (start..end).contains(&tone.at))
            .count();
        if onsets >= 2 || window == 6 {
            line.push(Tone {
                at: start,
                length: if window == 6 { 12 } else { 8 },
                degree: first,
            });
            if window == 6 {
                break;
            }
        } else {
            line.push(Tone {
                at: start,
                length: 4,
                degree: first,
            });
            let second = [first + 2, first - 2, first + 1, first - 1]
                .into_iter()
                .find(|degree| safe(*degree))
                .unwrap_or(first);
            line.push(Tone {
                at: start + 4,
                length: 4,
                degree: second,
            });
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::composition::Pace;
    use crate::adventure::harmony::{phrase_harmony, Chord, PhraseHarmonyInput};

    fn themes() -> Vec<Vec<Tone>> {
        (0..200)
            .map(|seed| compose_theme(&mut DeterministicRandom::new(seed)))
            .collect()
    }

    #[test]
    fn themes_sing_mostly_by_step_within_a_hummable_range() {
        let mut steps = 0;
        let mut moves = 0;
        for theme in themes() {
            let high = theme.iter().map(|t| t.degree).max().unwrap();
            let low = theme.iter().map(|t| t.degree).min().unwrap();
            assert!(high - low <= 8, "range {} in {theme:?}", high - low);
            assert!(theme
                .windows(2)
                .all(|p| (p[1].degree - p[0].degree).abs() <= 4));
            assert!(matches!(theme.last().unwrap().degree.rem_euclid(7), 4 | 1));
            for pair in theme.windows(2) {
                moves += 1;
                steps += usize::from((pair[1].degree - pair[0].degree).abs() == 1);
            }
        }
        assert!(steps * 100 / moves >= 50, "only {steps}/{moves} steps");
    }

    #[test]
    fn themes_vary_across_seeds() {
        let distinct: std::collections::HashSet<_> = themes()
            .into_iter()
            .map(|theme| theme.iter().map(|t| (t.at, t.degree)).collect::<Vec<_>>())
            .collect();
        assert!(distinct.len() > 120, "{} distinct themes", distinct.len());
    }

    fn phrase(kind: PhraseKind, treatment: Treatment, seed: u32) -> (Vec<Tone>, Vec<ChordSpan>) {
        let theme = compose_theme(&mut DeterministicRandom::new(seed));
        let intervals = crate::theory::mode_intervals("dorian");
        let chords = phrase_harmony(&PhraseHarmonyInput {
            mode: "dorian",
            intervals: &intervals,
            kind,
            pace: Pace::Walking,
            phrase_start: 0,
            bar_ticks: 3840,
            statement_variant: seed as usize,
            seed,
            phrase: 1,
            danger: 0.5,
            mystery: 0.5,
            open_color: false,
            suspends: false,
        });
        let tones = phrase_melody(&PhraseMelodyInput {
            theme: &theme,
            intervals: &intervals,
            kind,
            treatment,
            dominant: 6,
            chords: &chords,
            phrase_start: 0,
            sixteenth: 240,
            seed,
            phrase: 1,
            motion: 0.5,
            mystery: 0.5,
            pickup: true,
        });
        (tones, chords)
    }

    const KINDS: [PhraseKind; 5] = [
        PhraseKind::Antecedent,
        PhraseKind::Consequent,
        PhraseKind::Development,
        PhraseKind::Return,
        PhraseKind::Cadence,
    ];
    const TREATMENTS: [Treatment; 4] = [
        Treatment::Statement,
        Treatment::Augmented,
        Treatment::Fragment,
        Treatment::Diminution,
    ];

    #[test]
    fn phrases_stay_in_order_inside_four_bars_and_arrive_on_the_downbeat() {
        for seed in 0..60 {
            for kind in KINDS {
                for treatment in TREATMENTS {
                    let (tones, _) = phrase(kind, treatment, seed);
                    for pair in tones.windows(2) {
                        assert!(
                            pair[0].at + pair[0].length <= pair[1].at,
                            "{kind:?} {treatment:?} overlap {pair:?}"
                        );
                    }
                    assert!(tones.last().unwrap().at + tones.last().unwrap().length <= PHRASE);
                    let arrival = tones.iter().find(|t| t.at == ARRIVAL_AT).expect("arrival");
                    let expected =
                        if matches!(kind, PhraseKind::Antecedent | PhraseKind::Development) {
                            6
                        } else {
                            0
                        };
                    assert_eq!(
                        arrival.degree.rem_euclid(7),
                        expected,
                        "{kind:?} {treatment:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn strong_beats_land_on_chord_tones() {
        let mut strong = 0;
        let mut fitted = 0;
        for seed in 0..60 {
            for kind in KINDS {
                for treatment in TREATMENTS {
                    let (tones, chords) = phrase(kind, treatment, seed);
                    for tone in tones.iter().filter(|t| t.at % 8 == 0) {
                        let chord: Chord = chord_at(&chords, tone.at * 240).unwrap();
                        strong += 1;
                        fitted += usize::from(chord.contains(tone.degree));
                    }
                }
            }
        }
        assert!(
            fitted * 100 / strong >= 97,
            "{fitted}/{strong} strong beats on chord tones"
        );
    }

    #[test]
    fn the_theme_is_recognisable_in_every_statement() {
        for seed in 0..40 {
            let theme = compose_theme(&mut DeterministicRandom::new(seed));
            let (tones, _) = phrase(PhraseKind::Antecedent, Treatment::Statement, seed);
            let opening: Vec<u32> = tones
                .iter()
                .filter(|t| t.at < 2 * BAR)
                .map(|t| t.at)
                .collect();
            let rhythm: Vec<u32> = theme.iter().map(|t| t.at).collect();
            assert_eq!(opening, rhythm, "statement keeps the theme rhythm");
        }
    }

    #[test]
    fn the_counter_line_sits_below_and_avoids_semitone_rubs() {
        let intervals = crate::theory::mode_intervals("dorian");
        let pitch = |degree: i32| crate::theory::scale_pitch(62, degree, &intervals);
        for seed in 0..40 {
            let (melody, chords) = phrase(PhraseKind::Consequent, Treatment::Statement, seed);
            let counter = counter_line(&melody, &chords, 0, 240, |a, b| {
                matches!((pitch(a) - pitch(b)).rem_euclid(12), 1 | 11)
            });
            assert!(!counter.is_empty());
            for tone in &counter {
                for m in melody
                    .iter()
                    .filter(|m| m.at < tone.at + tone.length && tone.at < m.at + m.length)
                {
                    assert!(m.degree - tone.degree >= 2, "counter crossed the melody");
                    assert!(!matches!(
                        (pitch(m.degree) - pitch(tone.degree)).rem_euclid(12),
                        1 | 11
                    ));
                }
            }
        }
    }
}
