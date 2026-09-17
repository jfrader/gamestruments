//! Shared seeded composer over a phase pool.
//!
//! The three composed recipes (Suspense, Racing, Adventure) each describe
//! their sections as a flat pool with a role, an energy and a one-shot flag,
//! then hand that pool to this backtracker to choose a legal order and a
//! groove loop point. The recipes differ only in the data they feed in — the
//! role grammar, the energy bound, the seeded step-count band, whether they
//! want an outro ending, and an optional candidate refinement — the search
//! itself is one implementation, so the drift between the three copy-pasted
//! composers cannot rot.

use crate::rng::DeterministicRandom;
use crate::score::{SongForm, SongFormStep};

/// The composer's view of one pool phase.
pub(crate) trait PhaseMeta: Copy {
    fn id(&self) -> &'static str;
    fn one_shot(&self) -> bool;
    /// Whether the phase's role is a groove a loop point may target.
    fn is_groove(&self) -> bool;
    /// Whether the phase's role is the terminal outro.
    fn is_outro(&self) -> bool;
}

/// A recipe's phase pool: everything the shared composer needs to pick a
/// legal order and a loop point.
pub(crate) trait PhasePool: Sized {
    type Spec: PhaseMeta;
    /// Recipe-specific compose request (Suspense's intent, the unit type where
    /// there is none).
    type Request: Copy;

    /// The pool in canonical play order — the candidate enumeration order and
    /// the fallback tour.
    fn pool() -> Vec<Self::Spec>;
    /// Whether `next` may follow `prev` by role.
    fn role_legal(prev: Self::Spec, next: Self::Spec) -> bool;
    /// Whether `next` may follow `prev` by energy.
    fn energy_legal(prev: Self::Spec, next: Self::Spec) -> bool;
    /// Whether `spec` is a legal final phase given `want_outro`.
    fn ending_legal(spec: Self::Spec, want_outro: bool) -> bool;
    /// The seeded step count.
    fn count(request: Self::Request, rng: &mut DeterministicRandom) -> usize;
    /// Whether this compose prefers an outro ending.
    fn want_outro(request: Self::Request, rng: &mut DeterministicRandom) -> bool;
    /// The ending preference for the guaranteed-reachable retry after a failed
    /// search. Suspense falls back to a groove ending; Racing and Adventure
    /// keep the outro ending (always reachable for their live grammar).
    fn retry_want_outro() -> bool {
        true
    }
    /// Optional recipe-specific candidate refinement before the seeded shuffle
    /// (Suspense's figure-contrast budgets).
    fn refine(
        _chosen: &[Self::Spec],
        candidates: Vec<Self::Spec>,
        _root_seed: u32,
    ) -> Vec<Self::Spec> {
        candidates
    }
    /// The canonical fallback form: every pool phase once, in pool order,
    /// looping from the first groove.
    fn canonical() -> SongForm {
        canonical_form::<Self>()
    }
}

/// Compose a song form over the pool. The seed decides the step count, the
/// role selection, the order, and the loop point.
pub(crate) fn compose<P: PhasePool>(seed: u32, request: P::Request) -> SongForm {
    let pool = P::pool();
    let mut rng = DeterministicRandom::new(seed);
    let count = P::count(request, &mut rng);
    let want_outro = P::want_outro(request, &mut rng);
    let mut chosen: Vec<P::Spec> = Vec::with_capacity(count);
    let mut used_one_shot: Vec<&'static str> = Vec::new();
    let found = search::<P>(
        &mut rng,
        &pool,
        &mut chosen,
        &mut used_one_shot,
        count,
        want_outro,
        seed,
    );
    if !found {
        // An outro ending can be unreachable for the shortest forms (Suspense's
        // count-5 Surprise has no room for the required groove before the
        // outro), so retry with the recipe's guaranteed-reachable ending.
        chosen.clear();
        used_one_shot.clear();
        search::<P>(
            &mut rng,
            &pool,
            &mut chosen,
            &mut used_one_shot,
            count,
            P::retry_want_outro(),
            seed,
        );
    }
    let groove_indices: Vec<u32> = chosen
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.is_groove())
        .map(|(index, _)| index as u32)
        .collect();
    if chosen.len() < count || groove_indices.is_empty() {
        // Guard: never emit a short or grooveless form; degrade to the
        // canonical tour (every pool phase once, looping from the first groove).
        return P::canonical();
    }
    let loop_from = Some(groove_indices[rng.integer(groove_indices.len() as u32) as usize]);
    let steps = chosen.into_iter().map(step).collect();
    SongForm {
        steps,
        loop_from,
        origin: None,
    }
}

fn step<S: PhaseMeta>(spec: S) -> SongFormStep {
    SongFormStep {
        section: spec.id().to_string(),
        repeats: 1,
    }
}

/// Backtracking search over concrete pool phases. The role grammar is
/// deliberately "live": from any non-terminal state there is always at least
/// one legal continuation, so the search finds a solution for every reachable
/// count.
fn search<P: PhasePool>(
    rng: &mut DeterministicRandom,
    pool: &[P::Spec],
    chosen: &mut Vec<P::Spec>,
    used_one_shot: &mut Vec<&'static str>,
    count: usize,
    want_outro: bool,
    root_seed: u32,
) -> bool {
    if chosen.len() == count {
        return true;
    }
    let candidates = legal_next::<P>(rng, pool, chosen, used_one_shot, count, want_outro, root_seed);
    for candidate in candidates {
        chosen.push(candidate);
        if candidate.one_shot() {
            used_one_shot.push(candidate.id());
        }
        if search::<P>(rng, pool, chosen, used_one_shot, count, want_outro, root_seed) {
            return true;
        }
        chosen.pop();
        if candidate.one_shot() {
            used_one_shot.pop();
        }
    }
    false
}

fn legal_next<P: PhasePool>(
    rng: &mut DeterministicRandom,
    pool: &[P::Spec],
    chosen: &[P::Spec],
    used_one_shot: &[&'static str],
    count: usize,
    want_outro: bool,
    root_seed: u32,
) -> Vec<P::Spec> {
    let is_last = chosen.len() + 1 == count;
    let candidates: Vec<P::Spec> = match chosen.last() {
        None => vec![pool[0]],
        Some(prev) => pool
            .iter()
            .copied()
            .filter(|candidate| P::role_legal(*prev, *candidate))
            .filter(|candidate| !(candidate.one_shot() && used_one_shot.contains(&candidate.id())))
            .filter(|candidate| candidate.id() != prev.id())
            .filter(|candidate| P::energy_legal(*prev, *candidate))
            .filter(|candidate| {
                if is_last {
                    P::ending_legal(*candidate, want_outro)
                } else {
                    !candidate.is_outro()
                }
            })
            .collect(),
    };
    let refined = P::refine(chosen, candidates, root_seed);
    rng.shuffle(&refined)
}

/// The canonical tour: every pool phase once, in pool order, looping from the
/// first groove. The seeded search can only fall short if a future pool edit
/// breaks the live grammar, so this is the guaranteed-valid fallback.
pub(crate) fn canonical_form<P: PhasePool>() -> SongForm {
    let pool = P::pool();
    let steps = pool
        .iter()
        .map(|spec| SongFormStep {
            section: spec.id().to_string(),
            repeats: 1,
        })
        .collect();
    let loop_from = pool
        .iter()
        .position(|spec| spec.is_groove())
        .map(|index| index as u32);
    SongForm {
        steps,
        loop_from,
        origin: None,
    }
}
