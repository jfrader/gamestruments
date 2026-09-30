//! The Cozy phase pool for the shared seeded composer: a day's sections by
//! role and energy, so a seed composes its own day.

use crate::composer::{self, PhaseMeta, PhasePool};
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::SongForm;

use super::composition::{cozy_phase_energy, cozy_phase_role, CozyPhaseRole, SECTION_PLANS};

/// Largest energy step between neighbouring phases, except into and out of
/// the festival and down into the night, which may jump.
const MAX_ENERGY_DELTA: u32 = 36;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CozyPhaseSpec {
    pub(super) id: &'static str,
    pub(super) role: CozyPhaseRole,
    pub(super) energy: u32,
    pub(super) bars: u32,
}

impl PhaseMeta for CozyPhaseSpec {
    fn id(&self) -> &'static str {
        self.id
    }

    /// The grooves (the working day) may come back; everything else frames it.
    fn one_shot(&self) -> bool {
        self.role != CozyPhaseRole::Groove
    }

    fn is_groove(&self) -> bool {
        self.role == CozyPhaseRole::Groove
    }

    fn is_outro(&self) -> bool {
        self.role == CozyPhaseRole::Outro
    }
}

pub(super) fn cozy_phase_spec(id: &str) -> Option<CozyPhaseSpec> {
    let plan = SECTION_PLANS.iter().find(|plan| plan.id == id)?;
    Some(CozyPhaseSpec {
        id: plan.id,
        role: cozy_phase_role(plan.scene),
        energy: cozy_phase_energy(plan.scene),
        bars: plan.bars,
    })
}

pub(super) fn cozy_compose_seed(secret: &str, seed: &str) -> u32 {
    hash_text(&format!("{secret}\0{seed}\0cozy-compose-1"))
}

/// Compose a day from the pool: dawn opens, grooves carry the working day
/// (possibly broken by rain or lifted by the festival), and night closes it.
pub(super) fn cozy_compose(seed: u32) -> SongForm {
    composer::compose::<CozyPool>(seed, ())
}

pub(super) fn cozy_canonical_form() -> SongForm {
    composer::canonical_form::<CozyPool>()
}

struct CozyPool;

impl PhasePool for CozyPool {
    type Spec = CozyPhaseSpec;
    type Request = ();

    fn pool() -> Vec<CozyPhaseSpec> {
        SECTION_PLANS
            .iter()
            .filter_map(|plan| cozy_phase_spec(plan.id))
            .collect()
    }

    fn role_legal(prev: CozyPhaseSpec, next: CozyPhaseSpec) -> bool {
        use CozyPhaseRole::*;
        match prev.role {
            Intro => next.role == Groove,
            Groove => matches!(next.role, Groove | Peak | Break | Outro),
            Peak => matches!(next.role, Groove | Outro),
            Break => matches!(next.role, Groove | Outro),
            Outro => false,
        }
    }

    fn energy_legal(prev: CozyPhaseSpec, next: CozyPhaseSpec) -> bool {
        let gesture = prev.role == CozyPhaseRole::Peak
            || next.role == CozyPhaseRole::Peak
            || next.role == CozyPhaseRole::Outro;
        gesture || prev.energy.abs_diff(next.energy) <= MAX_ENERGY_DELTA
    }

    fn ending_legal(spec: CozyPhaseSpec, _want_outro: bool) -> bool {
        spec.role == CozyPhaseRole::Outro
    }

    fn count(_request: (), rng: &mut DeterministicRandom) -> usize {
        (6 + rng.integer(4)) as usize // 6..=9 steps
    }

    fn want_outro(_request: (), _rng: &mut DeterministicRandom) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_seed_composes_a_legal_day() {
        let mut forms = HashSet::new();
        for index in 0..300 {
            let form = cozy_compose(cozy_compose_seed("s", &format!("day-{index}")));
            let steps: Vec<&str> = form
                .steps
                .iter()
                .map(|step| step.section.as_str())
                .collect();
            assert_eq!(steps.first(), Some(&"dawn"), "{steps:?}");
            assert_eq!(steps.last(), Some(&"night"), "{steps:?}");
            let loop_from = form.loop_from.expect("a day loops") as usize;
            assert!(cozy_phase_spec(steps[loop_from]).unwrap().is_groove());
            for pair in form.steps.windows(2) {
                let (a, b) = (
                    cozy_phase_spec(&pair[0].section).unwrap(),
                    cozy_phase_spec(&pair[1].section).unwrap(),
                );
                assert!(
                    CozyPool::role_legal(a, b) && CozyPool::energy_legal(a, b),
                    "{steps:?}"
                );
            }
            forms.insert(steps.join(","));
        }
        assert!(forms.len() > 20, "only {} distinct days", forms.len());
    }
}
