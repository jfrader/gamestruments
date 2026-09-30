//! Voice leading shared by the composed recipes.

/// Of every ordered voicing that takes one pitch from each voice's
/// `candidates` (lowest voice first) with at least `min_gap` semitones between
/// neighbours, the one whose voices move least from `targets` — squared, so no
/// single voice leaps. Ties keep the first voicing in candidate order.
pub(crate) fn least_motion_voicing(
    candidates: &[Vec<i32>],
    targets: &[i32],
    min_gap: i32,
) -> Option<Vec<i32>> {
    fn search(
        candidates: &[Vec<i32>],
        targets: &[i32],
        min_gap: i32,
        chosen: &mut Vec<i32>,
        best: &mut Option<(i32, Vec<i32>)>,
    ) {
        let voice = chosen.len();
        if voice == candidates.len() {
            let motion = chosen
                .iter()
                .zip(targets)
                .map(|(p, t)| (p - t).pow(2))
                .sum();
            if best.as_ref().is_none_or(|(least, _)| motion < *least) {
                *best = Some((motion, chosen.clone()));
            }
            return;
        }
        for &pitch in &candidates[voice] {
            if chosen.last().is_some_and(|below| pitch < below + min_gap) {
                continue;
            }
            chosen.push(pitch);
            search(candidates, targets, min_gap, chosen, best);
            chosen.pop();
        }
    }
    let mut best = None;
    search(
        candidates,
        targets,
        min_gap,
        &mut Vec::with_capacity(candidates.len()),
        &mut best,
    );
    best.map(|(_, voicing)| voicing)
}
