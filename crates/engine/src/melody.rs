//! Melodic building blocks shared by the composed recipes. Lines live in
//! scale-degree space (degree 7 is the octave) on a sixteenth-note grid.

use crate::rng::DeterministicRandom;

/// One note of a line: onset and length in sixteenths from the phrase start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Tone {
    pub(crate) at: u32,
    pub(crate) length: u32,
    pub(crate) degree: i32,
}

/// A theme on one of `rhythms`, chosen from many candidate contours by how
/// well it sings: mostly steps, one climax, leaps recovered by step, a hummable
/// range, ending on an open degree (the fifth or the second).
pub(crate) fn compose_theme(
    rng: &mut DeterministicRandom,
    rhythms: &[&'static [(u32, u32)]],
) -> Vec<Tone> {
    let rhythm = *rng.pick(rhythms);
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

/// Metric weight of a sixteenth position within the bar: downbeat, half bar,
/// beats, off-beats, then the in-between sixteenths.
pub(crate) fn metric_accent(position: u32) -> f64 {
    match position % 16 {
        0 => 1.0,
        8 => 0.93,
        4 | 12 => 0.87,
        p if p % 2 == 0 => 0.8,
        _ => 0.74,
    }
}
