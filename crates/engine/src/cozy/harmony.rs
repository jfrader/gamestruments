//! Cozy harmony: seventh chords in a major key, borrowed and secondary
//! colours, and rootless voicings that move by the smallest steps.
//!
//! Chords are pitch-class sets over the key's tonic, not scale degrees, so
//! the jazz-pop vocabulary a cozy game wants (secondary dominants, the
//! borrowed minor iv, the backdoor bVII7) is spelled exactly.

use crate::rng::keyed_unit;
use crate::voicing::least_motion_voicing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Quality {
    Major7,
    Minor7,
    Dominant7,
    /// A dominant with its fourth held instead of the third (V7sus4).
    Suspended7,
    /// The borrowed minor iv with a major sixth: the "nostalgic" colour.
    Minor6,
}

impl Quality {
    /// Semitones above the root.
    fn intervals(self) -> [i32; 4] {
        match self {
            Self::Major7 => [0, 4, 7, 11],
            Self::Minor7 => [0, 3, 7, 10],
            Self::Dominant7 => [0, 4, 7, 10],
            Self::Suspended7 => [0, 5, 7, 10],
            Self::Minor6 => [0, 3, 7, 9],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Chord {
    /// Root in semitones above the key's tonic.
    pub(super) root: i32,
    pub(super) quality: Quality,
    /// Whether the ninth colours the chord.
    pub(super) ninth: bool,
}

impl Chord {
    const fn new(root: i32, quality: Quality) -> Self {
        Self {
            root,
            quality,
            ninth: false,
        }
    }

    /// Pitch classes above the tonic, root first.
    pub(super) fn pitch_classes(self) -> Vec<i32> {
        let mut classes: Vec<i32> = self
            .quality
            .intervals()
            .iter()
            .map(|interval| (self.root + interval).rem_euclid(12))
            .collect();
        if self.ninth {
            classes.push((self.root + 2).rem_euclid(12));
        }
        classes
    }

    /// Whether pitch class `class` (above the tonic) is a tone of the chord.
    pub(super) fn contains(self, class: i32) -> bool {
        self.pitch_classes().contains(&class.rem_euclid(12))
    }

    /// A chord tone the melody can rest on: in the chord and a semitone from
    /// none of its other tones (not the root under a major seventh, nor the
    /// minor third under a ninth).
    pub(super) fn rests_on(self, class: i32) -> bool {
        self.contains(class) && !self.rubs(class)
    }

    /// Whether `class` sits a semitone from any chord tone.
    pub(super) fn rubs(self, class: i32) -> bool {
        self.pitch_classes()
            .iter()
            .any(|tone| matches!((class - tone).rem_euclid(12), 1 | 11))
    }
}

const I: Chord = Chord::new(0, Quality::Major7);
const II: Chord = Chord::new(2, Quality::Minor7);
const III: Chord = Chord::new(4, Quality::Minor7);
const IV: Chord = Chord::new(5, Quality::Major7);
const V: Chord = Chord::new(7, Quality::Dominant7);
const V_SUS: Chord = Chord::new(7, Quality::Suspended7);
const VI: Chord = Chord::new(9, Quality::Minor7);
/// Borrowed from the parallel minor.
const IV_MINOR: Chord = Chord::new(5, Quality::Minor6);
/// The backdoor dominant: bVII7 falls home by a whole step.
const BACKDOOR: Chord = Chord::new(10, Quality::Dominant7);
/// Secondary dominants: of ii (VI7) and of vi (III7).
const V_OF_II: Chord = Chord::new(9, Quality::Dominant7);
const V_OF_VI: Chord = Chord::new(4, Quality::Dominant7);

/// Which harmonic world a section lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HarmonicColor {
    /// Slow, open, two bars per chord.
    Still,
    /// Diatonic pop loops.
    Bright,
    /// Secondary dominants for a bustling scene.
    Busy,
    /// The key's relative minor, for rain.
    Wistful,
    /// Plagal and borrowed colours for the evening.
    Golden,
}

/// Four-bar loops per colour, one chord per bar.
fn bank(color: HarmonicColor) -> &'static [[Chord; 4]] {
    match color {
        HarmonicColor::Still => &[[I, I, IV, IV], [IV, IV, I, I], [I, I, BACKDOOR, BACKDOOR]],
        HarmonicColor::Bright => &[[I, VI, II, V], [I, III, IV, V_SUS], [IV, V, III, VI]],
        HarmonicColor::Busy => &[
            [I, V_OF_II, II, V],
            [I, IV, V_OF_VI, VI],
            [II, V, I, V_OF_II],
        ],
        HarmonicColor::Wistful => &[[VI, IV, II, V_OF_VI], [VI, II, VI, III], [IV, III, II, VI]],
        HarmonicColor::Golden => &[[IV, IV_MINOR, I, VI], [II, V, I, IV], [IV, III, VI, V]],
    }
}

/// A chord sounding from `start` (section ticks) for `length`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ChordSpan {
    pub(super) start: u32,
    pub(super) length: u32,
    pub(super) chord: Chord,
}

pub(super) fn chord_at(spans: &[ChordSpan], tick: u32) -> Option<Chord> {
    spans
        .iter()
        .find(|span| (span.start..span.start + span.length).contains(&tick))
        .map(|span| span.chord)
}

pub(super) struct PhraseHarmonyInput {
    pub(super) color: HarmonicColor,
    /// Which loop of the colour's bank this phrase plays.
    pub(super) loop_index: usize,
    pub(super) phrase_start: u32,
    pub(super) bar_ticks: u32,
    pub(super) seed: u32,
    pub(super) phrase: u32,
    /// 0..1: ninths and ii–V substitutions become likelier.
    pub(super) jazz: f64,
}

/// One phrase's four bars of harmony. Jazz adds ninths and turns a bar that
/// lands on the dominant into a ii–V; every choice is a keyed draw against a
/// jazz-scaled threshold, so more jazz only ever adds colour.
pub(super) fn phrase_harmony(input: &PhraseHarmonyInput) -> Vec<ChordSpan> {
    let loops = bank(input.color);
    let chords = loops[input.loop_index % loops.len()];
    let half = input.bar_ticks / 2;
    let mut spans = Vec::with_capacity(6);
    for (bar, chord) in chords.into_iter().enumerate() {
        let start = input.phrase_start + bar as u32 * input.bar_ticks;
        let draw = |tag: &str| keyed_unit(input.seed, tag, input.phrase, bar as u32);
        let mut chord = chord;
        chord.ninth = chord.quality != Quality::Minor6 && draw("ninth") < 0.15 + input.jazz * 0.7;
        let split = chord.quality == Quality::Dominant7
            && input.color != HarmonicColor::Still
            && draw("two-five") < input.jazz * 0.6;
        if split {
            let two = Chord {
                root: (chord.root + 7).rem_euclid(12),
                quality: Quality::Minor7,
                ninth: chord.ninth,
            };
            spans.push(ChordSpan {
                start,
                length: half,
                chord: two,
            });
            spans.push(ChordSpan {
                start: start + half,
                length: half,
                chord,
            });
        } else {
            spans.push(ChordSpan {
                start,
                length: input.bar_ticks,
                chord,
            });
        }
    }
    merge_repeats(&mut spans);
    spans
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

/// A rootless voicing (the bass carries the root) in the keyboard register,
/// in whichever inversion moves least from the previous voicing.
pub(super) fn voice_chord(tonic: i32, chord: Chord, previous: Option<&[i32]>) -> Vec<i32> {
    const LOW: i32 = 52;
    const HIGH: i32 = 76;
    let mut classes: Vec<i32> = chord.pitch_classes().into_iter().skip(1).collect();
    if classes.len() > 3 && chord.ninth {
        // With the ninth in, the fifth is the dispensable tone.
        let fifth = (chord.root + 7).rem_euclid(12);
        classes.retain(|class| *class != fifth);
    }
    let default_targets: Vec<i32> = (0..classes.len() as i32).map(|v| 57 + v * 4).collect();
    let targets = previous
        .filter(|p| p.len() == classes.len())
        .map_or(default_targets, <[i32]>::to_vec);
    let pitches_of = |class: i32| -> Vec<i32> {
        (LOW..=HIGH)
            .filter(|pitch| (pitch - tonic - class).rem_euclid(12) == 0)
            .collect()
    };
    let mut best: Option<(i32, Vec<i32>)> = None;
    for rotation in 0..classes.len() {
        let candidates: Vec<Vec<i32>> = (0..classes.len())
            .map(|voice| pitches_of(classes[(voice + rotation) % classes.len()]))
            .collect();
        let Some(voicing) = least_motion_voicing(&candidates, &targets, 2) else {
            continue;
        };
        let motion: i32 = voicing
            .iter()
            .zip(&targets)
            .map(|(p, t)| (p - t).pow(2))
            .sum();
        if best.as_ref().is_none_or(|(least, _)| motion < *least) {
            best = Some((motion, voicing));
        }
    }
    best.expect("a rootless voicing always fits two octaves").1
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLORS: [HarmonicColor; 5] = [
        HarmonicColor::Still,
        HarmonicColor::Bright,
        HarmonicColor::Busy,
        HarmonicColor::Wistful,
        HarmonicColor::Golden,
    ];

    fn plan(color: HarmonicColor, seed: u32, jazz: f64) -> Vec<ChordSpan> {
        phrase_harmony(&PhraseHarmonyInput {
            color,
            loop_index: seed as usize,
            phrase_start: 0,
            bar_ticks: 3840,
            seed,
            phrase: seed % 4,
            jazz,
        })
    }

    #[test]
    fn plans_tile_four_bars() {
        for color in COLORS {
            for seed in 0..30 {
                let spans = plan(color, seed, f64::from(seed % 3) / 2.0);
                let mut cursor = 0;
                for span in &spans {
                    assert_eq!(span.start, cursor);
                    cursor += span.length;
                }
                assert_eq!(cursor, 4 * 3840, "{color:?}");
            }
        }
    }

    #[test]
    fn jazz_only_adds_colour() {
        for color in COLORS {
            let count = |jazz: f64| {
                (0..60)
                    .map(|seed| {
                        plan(color, seed, jazz)
                            .iter()
                            .map(|span| usize::from(span.chord.ninth) + 1)
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            };
            assert!(count(0.1) < count(0.9), "{color:?}");
        }
    }

    #[test]
    fn a_two_five_resolves_its_dominant() {
        let spans = (0..80)
            .flat_map(|seed| plan(HarmonicColor::Bright, seed, 1.0))
            .collect::<Vec<_>>();
        let pairs = spans
            .windows(2)
            .filter(|pair| {
                pair[1].chord.quality == Quality::Dominant7
                    && pair[0].chord.quality == Quality::Minor7
                    && (pair[0].chord.root - pair[1].chord.root).rem_euclid(12) == 7
            })
            .count();
        assert!(pairs > 0, "jazz must write ii–V pairs");
    }

    #[test]
    fn voicings_stay_rootless_in_register_and_move_smoothly() {
        let mut previous: Option<Vec<i32>> = None;
        for (index, chord) in [I, VI, II, V, IV, IV_MINOR, I, BACKDOOR, V_OF_II, III]
            .into_iter()
            .enumerate()
        {
            let chord = Chord {
                ninth: index % 2 == 0,
                ..chord
            };
            let voicing = voice_chord(2, chord, previous.as_deref());
            assert!(voicing.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(voicing.iter().all(|pitch| (52..=76).contains(pitch)));
            for pitch in &voicing {
                assert!(chord.contains(pitch - 2), "{pitch} not in {chord:?}");
            }
            if let Some(old) = &previous {
                if old.len() == voicing.len() {
                    let motion: i32 = old.iter().zip(&voicing).map(|(a, b)| (a - b).abs()).sum();
                    assert!(motion <= 12, "{chord:?} moved {motion}");
                }
            }
            previous = Some(voicing);
        }
    }
}
