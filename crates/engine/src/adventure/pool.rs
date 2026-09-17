//! Shared phase pool + seeded composer for the Adventure composed arrangement.
//!
//! Every section the Adventure generator produces is described once here by
//! role and energy, derived from the same authored
//! [`SECTION_PLANS`](super::composition::SECTION_PLANS) table that generates
//! the sections (see [`adventure_phase_role`] and [`adventure_phase_energy`]).
//! Composing an Adventure song is then choosing a [`SongForm`] over that pool;
//! section generation is untouched.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{SongForm, SongFormStep};

use super::composition::{
    adventure_phase_energy, adventure_phase_role, AdventurePhaseRole, Scene, SECTION_PLANS,
};

/// Maximum allowed energy step between adjacent composed phases (0-100 scale),
/// except the intentional gestures (the battle erupts Build → Peak, and the
/// post-climax respite Peak → Break).
pub(super) const MAX_ENERGY_DELTA: u32 = 40;

/// The Adventure phase pool in natural quest order.
pub(super) const ADVENTURE_SECTION_IDS: [&str; 8] = [
    "camp",
    "explore",
    "town",
    "dungeon",
    "combat",
    "boss",
    "sanctuary",
    "victory",
];

/// An Adventure section as the composer sees it: its role and energy band,
/// derived from the authored plan, plus the authored length (16 or 32 bars) and
/// whether it may appear only once in a form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AdventurePhaseSpec {
    pub(super) id: &'static str,
    pub(super) role: AdventurePhaseRole,
    pub(super) energy: u32,
    pub(super) bars: u32,
    pub(super) one_shot: bool,
}

/// The phase spec for an Adventure section id, if it is part of the pool.
pub(super) fn adventure_phase_spec(id: &str) -> Option<AdventurePhaseSpec> {
    let plan = SECTION_PLANS.iter().find(|plan| plan.id == id)?;
    Some(AdventurePhaseSpec {
        id: plan.id,
        role: adventure_phase_role(plan.scene),
        energy: adventure_phase_energy(plan.scene),
        bars: plan.bars,
        one_shot: !matches!(plan.scene, Scene::Explore | Scene::Town | Scene::Combat),
    })
}

/// The compose seed for an Adventure level: a hash of the project secret and
/// level seed under a domain unique to the composed Adventure path. Adventure
/// has no reel concept, so the seed is not index-addressed.
pub(super) fn adventure_compose_seed(secret: &str, seed: &str) -> u32 {
    hash_text(&format!("{secret}\0{seed}\0adventure-compose-1"))
}

/// Compose an Adventure song from the pool. The seed decides the step count,
/// the order, and the loop point (always a groove, so the loop keeps the quest
/// moving). The Intro (camp) and Outro (victory) are one-shot; they frame the
/// form exactly once.
pub(super) fn adventure_compose(seed: u32) -> SongForm {
    let mut rng = DeterministicRandom::new(seed);
    let count = (8 + rng.integer(5)) as usize; // 8..=12 steps
    let mut chosen: Vec<AdventurePhaseSpec> = Vec::with_capacity(count);
    let mut used_one_shot: Vec<&'static str> = Vec::new();
    let found = adventure_search(&mut rng, &mut chosen, &mut used_one_shot, count);
    if !found {
        // Unreachable for this pool (the grammar is live); kept as a guard so a
        // future pool edit degrades to the canonical quest arc instead of
        // looping forever.
        chosen = ADVENTURE_SECTION_IDS
            .iter()
            .filter_map(|&id| adventure_phase_spec(id))
            .collect();
    }
    let groove_indices: Vec<u32> = chosen
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.role == AdventurePhaseRole::Groove)
        .map(|(index, _)| index as u32)
        .collect();
    let loop_from = if groove_indices.is_empty() {
        None
    } else {
        Some(groove_indices[rng.integer(groove_indices.len() as u32) as usize])
    };
    let steps = chosen.into_iter().map(adventure_step).collect();
    SongForm {
        steps,
        loop_from,
        origin: None,
    }
}

fn adventure_step(spec: AdventurePhaseSpec) -> SongFormStep {
    SongFormStep {
        section: spec.id.to_string(),
        repeats: 1,
    }
}

fn adventure_search(
    rng: &mut DeterministicRandom,
    chosen: &mut Vec<AdventurePhaseSpec>,
    used_one_shot: &mut Vec<&'static str>,
    count: usize,
) -> bool {
    if chosen.len() == count {
        return true;
    }
    let candidates = adventure_legal_next(rng, chosen, used_one_shot, count);
    for candidate in candidates {
        chosen.push(candidate);
        if candidate.one_shot {
            used_one_shot.push(candidate.id);
        }
        if adventure_search(rng, chosen, used_one_shot, count) {
            return true;
        }
        chosen.pop();
        if candidate.one_shot {
            used_one_shot.pop();
        }
    }
    false
}

fn adventure_legal_next(
    rng: &mut DeterministicRandom,
    chosen: &[AdventurePhaseSpec],
    used_one_shot: &[&'static str],
    count: usize,
) -> Vec<AdventurePhaseSpec> {
    let is_last = chosen.len() + 1 == count;
    let candidates: Vec<_> = match chosen.last() {
        None => vec![adventure_phase_spec("camp").expect("camp is in the pool")],
        Some(prev) => ADVENTURE_SECTION_IDS
            .iter()
            .filter_map(|&id| adventure_phase_spec(id))
            .filter(|candidate| adventure_role_legal(prev, candidate))
            .filter(|candidate| !(candidate.one_shot && used_one_shot.contains(&candidate.id)))
            .filter(|candidate| candidate.id != prev.id)
            .filter(|candidate| adventure_energy_legal(prev, candidate))
            .filter(|candidate| {
                if is_last {
                    candidate.role == AdventurePhaseRole::Outro
                } else {
                    candidate.role != AdventurePhaseRole::Outro
                }
            })
            .collect(),
    };
    rng.shuffle(&candidates)
}

fn adventure_role_legal(prev: &AdventurePhaseSpec, next: &AdventurePhaseSpec) -> bool {
    use AdventurePhaseRole::*;
    match prev.role {
        Intro => next.role == Groove,
        Groove => matches!(next.role, Groove | Build | Break),
        Build => next.role == Peak,
        Peak => matches!(next.role, Peak | Break | Outro),
        Break => next.role == Outro,
        Outro => false,
    }
}

fn adventure_energy_legal(prev: &AdventurePhaseSpec, next: &AdventurePhaseSpec) -> bool {
    // The intentional gestures: the battle erupts (Build -> Peak) and the
    // post-climax respite (Peak -> Break) may jump the band.
    if prev.role == AdventurePhaseRole::Build && next.role == AdventurePhaseRole::Peak {
        return true;
    }
    if prev.role == AdventurePhaseRole::Peak && next.role == AdventurePhaseRole::Break {
        return true;
    }
    i32::abs(prev.energy as i32 - next.energy as i32) <= MAX_ENERGY_DELTA as i32
}

/// Role-aware length for the composed path: intros, breaks and outros stay at
/// the authored length, while builds, grooves and peaks may stretch one extra
/// 16-bar movement so the journey has room to breathe. Bars are always a
/// multiple of the authored 16-bar block, so a section re-times by tiling.
pub(super) fn adventure_phase_bars(spec: &AdventurePhaseSpec, seed: u32) -> u32 {
    let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:bars", spec.id)));
    let authored = spec.bars;
    match spec.role {
        AdventurePhaseRole::Intro | AdventurePhaseRole::Break | AdventurePhaseRole::Outro => {
            authored
        }
        AdventurePhaseRole::Build | AdventurePhaseRole::Groove | AdventurePhaseRole::Peak => {
            authored + 16 * rng.integer(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Validate the hard composition rules against a form's step ids (resolved
    /// against the pool), returning the first violation.
    fn validate_adventure_form(ids: &[&str], loop_from: Option<u32>) -> Result<(), String> {
        if ids.is_empty() {
            return Err("adventure form is empty".into());
        }
        let specs: Vec<AdventurePhaseSpec> = ids
            .iter()
            .map(|id| adventure_phase_spec(id).ok_or_else(|| format!("unknown phase {id}")))
            .collect::<Result<_, _>>()?;

        // Intro first, outro last, both once.
        if specs[0].role != AdventurePhaseRole::Intro {
            return Err("form must begin with the Intro".into());
        }
        if specs.last().map(|spec| spec.role) != Some(AdventurePhaseRole::Outro) {
            return Err("form must end with the Outro".into());
        }
        let mut seen_one_shot: Vec<&str> = Vec::new();
        for spec in &specs {
            if spec.one_shot {
                if seen_one_shot.contains(&spec.id) {
                    return Err(format!("one-shot phase {} repeats", spec.id));
                }
                seen_one_shot.push(spec.id);
            }
        }
        for (index, spec) in specs.iter().enumerate() {
            if spec.role == AdventurePhaseRole::Intro && index != 0 {
                return Err(format!("Intro appears at step {index}, not first"));
            }
            if spec.role == AdventurePhaseRole::Outro && index != specs.len() - 1 {
                return Err(format!("Outro {} is not last", spec.id));
            }
            if let Some(prev) = index.checked_sub(1).map(|i| specs[i]) {
                if !adventure_role_legal(&prev, spec) {
                    return Err(format!("illegal transition {} -> {}", prev.id, spec.id));
                }
                if !adventure_energy_legal(&prev, spec) {
                    return Err(format!(
                        "energy jump {} -> {} exceeds bound",
                        prev.id, spec.id
                    ));
                }
            }
        }
        if let Some(loop_from) = loop_from {
            let index = loop_from as usize;
            if index >= specs.len() {
                return Err(format!("loopFrom {loop_from} is out of range"));
            }
            if specs[index].role != AdventurePhaseRole::Groove {
                return Err(format!("loopFrom targets non-groove {}", specs[index].id));
            }
        }
        Ok(())
    }

    #[test]
    fn metadata_derives_role_and_energy_from_plans() {
        for (id, role, energy, bars, one_shot) in [
            ("camp", AdventurePhaseRole::Intro, 25, 16, true),
            ("explore", AdventurePhaseRole::Groove, 48, 32, false),
            ("town", AdventurePhaseRole::Groove, 58, 32, false),
            ("dungeon", AdventurePhaseRole::Build, 20, 16, true),
            ("combat", AdventurePhaseRole::Peak, 82, 32, false),
            ("boss", AdventurePhaseRole::Peak, 90, 16, true),
            ("sanctuary", AdventurePhaseRole::Break, 34, 16, true),
            ("victory", AdventurePhaseRole::Outro, 70, 32, true),
        ] {
            let spec = adventure_phase_spec(id).unwrap();
            assert_eq!(spec.role, role, "role for {id}");
            assert_eq!(spec.energy, energy, "energy for {id}");
            assert_eq!(spec.bars, bars, "bars for {id}");
            assert_eq!(spec.one_shot, one_shot, "one_shot for {id}");
        }
        assert_eq!(adventure_phase_spec("not-a-section"), None);
    }

    #[test]
    fn pool_declares_every_adventure_section() {
        assert_eq!(
            ADVENTURE_SECTION_IDS
                .iter()
                .filter_map(|&id| adventure_phase_spec(id))
                .count(),
            8
        );
    }

    #[test]
    fn composer_is_deterministic() {
        for seed in [0xdead_beef, 0, 42, 0x51ed_0000] {
            assert_eq!(adventure_compose(seed), adventure_compose(seed), "seed {seed}");
        }
    }

    #[test]
    fn composer_produces_rule_legal_forms_across_many_seeds() {
        let mut distinct = HashSet::new();
        for seed in 0..250u32 {
            let form = adventure_compose(0x5eed_0000 ^ seed);
            let ids: Vec<String> = form.steps.iter().map(|step| step.section.clone()).collect();
            let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
            validate_adventure_form(&id_refs, form.loop_from)
                .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
            assert!(
                (8..=12).contains(&ids.len()),
                "seed {seed}: count {}",
                ids.len()
            );
            assert!(form.loop_from.is_some(), "seed {seed}: missing loop point");
            distinct.insert(ids);
        }
        assert!(distinct.len() > 20, "only {} distinct forms", distinct.len());
    }

    #[test]
    fn compose_seed_differs_by_secret_and_seed() {
        assert_ne!(
            adventure_compose_seed("project", "level-001"),
            adventure_compose_seed("project", "level-002")
        );
        assert_ne!(
            adventure_compose_seed("project", "level-001"),
            adventure_compose_seed("other", "level-001")
        );
    }

    #[test]
    fn phase_bars_are_role_aware_and_phrase_aligned() {
        for seed in 0..64u32 {
            for id in ADVENTURE_SECTION_IDS {
                let spec = adventure_phase_spec(id).unwrap();
                let bars = adventure_phase_bars(&spec, seed);
                assert_eq!(bars % 16, 0, "{id} bars {bars} not block-aligned");
                match spec.role {
                    AdventurePhaseRole::Intro
                    | AdventurePhaseRole::Break
                    | AdventurePhaseRole::Outro => {
                        assert_eq!(bars, spec.bars, "{id}")
                    }
                    AdventurePhaseRole::Build | AdventurePhaseRole::Peak => {
                        assert!(
                            (spec.bars..=spec.bars + 16).contains(&bars),
                            "{id} bars {bars}"
                        );
                    }
                    AdventurePhaseRole::Groove => {
                        assert!(
                            (spec.bars..=spec.bars + 16).contains(&bars),
                            "{id} bars {bars}"
                        );
                    }
                }
            }
        }
    }
}
