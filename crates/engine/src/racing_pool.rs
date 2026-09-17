//! Shared phase pool + seeded composer for the Racing composed arrangement.
//!
//! Every section the Racing generator produces is described once here by role
//! and energy, derived from the same authored [`PLANS`](crate::racing::PLANS)
//! table that generates the sections (see [`racing_phase_role`] and
//! [`racing_phase_energy`]). Composing a Racing song is then choosing a
//! [`SongForm`] over that pool; section generation is untouched.

use crate::racing::{racing_phase_energy, racing_phase_role, RacingPhaseRole};
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{SongForm, SongFormStep};

/// Maximum allowed energy step between adjacent composed phases (0-100 scale).
pub const MAX_ENERGY_DELTA: u32 = 35;

/// The Racing phase pool in natural race order.
pub const RACING_SECTION_IDS: [&str; 6] = [
    "garage",
    "grid",
    "cruise",
    "attack",
    "final-lap",
    "victory",
];

/// A Racing phase as the composer sees it: its role and energy band, derived
/// from the authored plan, plus the authored length (every Racing section is
/// four bars) and whether it may appear only once in a form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RacingPhaseSpec {
    pub id: &'static str,
    pub role: RacingPhaseRole,
    pub energy: u32,
    pub bars: u32,
    pub one_shot: bool,
}

/// The phase spec for a Racing section id, if it is part of the pool.
pub fn racing_phase_spec(id: &str) -> Option<RacingPhaseSpec> {
    let pool_id = RACING_SECTION_IDS
        .iter()
        .copied()
        .find(|candidate| *candidate == id)?;
    Some(RacingPhaseSpec {
        id: pool_id,
        role: racing_phase_role(id)?,
        energy: racing_phase_energy(id)?,
        bars: 4,
        one_shot: matches!(id, "garage" | "victory"),
    })
}

/// The compose seed for a Racing level: a hash of the project secret and level
/// seed under a domain unique to the composed Racing path. Racing has no reel
/// concept, so the seed is not index-addressed.
pub fn racing_compose_seed(secret: &str, seed: &str) -> u32 {
    hash_text(&format!("{secret}\0{seed}\0racing-compose-1"))
}

/// Compose a Racing song from the pool. The seed decides the step count, the
/// order, and the loop point (always a groove, so the loop keeps the race
/// moving). Intro and Outro are one-shot; they frame the form exactly once.
pub fn racing_compose(seed: u32) -> SongForm {
    let mut rng = DeterministicRandom::new(seed);
    let count = (6 + rng.integer(4)) as usize; // 6..=9 steps
    let mut chosen: Vec<RacingPhaseSpec> = Vec::with_capacity(count);
    let mut used_one_shot: Vec<&'static str> = Vec::new();
    let found = racing_search(&mut rng, &mut chosen, &mut used_one_shot, count);
    if !found {
        // Unreachable for this pool (the grammar is live); retried so a future
        // pool edit prefers any legal form over the canonical fallback.
        chosen.clear();
        used_one_shot.clear();
        racing_search(&mut rng, &mut chosen, &mut used_one_shot, count);
    }
    let groove_indices: Vec<u32> = chosen
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.role == RacingPhaseRole::Groove)
        .map(|(index, _)| index as u32)
        .collect();
    if chosen.len() < count || groove_indices.is_empty() {
        // Guard: never emit a short or grooveless form; degrade to the
        // canonical tour (every pool phase once, looping from the groove).
        return racing_canonical_form();
    }
    let loop_from = Some(groove_indices[rng.integer(groove_indices.len() as u32) as usize]);
    let steps = chosen.into_iter().map(racing_step).collect();
    SongForm {
        steps,
        loop_from,
        origin: None,
    }
}

/// The canonical Racing form: every pool phase once, in natural race order,
/// looping from the first groove (cruise). The seeded search can only fall
/// short if a future pool edit breaks the live grammar, so this is the
/// guaranteed-valid fallback.
fn racing_canonical_form() -> SongForm {
    let steps = RACING_SECTION_IDS
        .iter()
        .map(|id| SongFormStep {
            section: (*id).to_string(),
            repeats: 1,
        })
        .collect();
    let loop_from = RACING_SECTION_IDS
        .iter()
        .position(|id| {
            racing_phase_spec(id).is_some_and(|spec| spec.role == RacingPhaseRole::Groove)
        })
        .map(|index| index as u32);
    SongForm {
        steps,
        loop_from,
        origin: None,
    }
}

fn racing_step(spec: RacingPhaseSpec) -> SongFormStep {
    SongFormStep {
        section: spec.id.to_string(),
        repeats: 1,
    }
}

fn racing_search(
    rng: &mut DeterministicRandom,
    chosen: &mut Vec<RacingPhaseSpec>,
    used_one_shot: &mut Vec<&'static str>,
    count: usize,
) -> bool {
    if chosen.len() == count {
        return true;
    }
    let candidates = racing_legal_next(rng, chosen, used_one_shot, count);
    for candidate in candidates {
        chosen.push(candidate);
        if candidate.one_shot {
            used_one_shot.push(candidate.id);
        }
        if racing_search(rng, chosen, used_one_shot, count) {
            return true;
        }
        chosen.pop();
        if candidate.one_shot {
            used_one_shot.pop();
        }
    }
    false
}

fn racing_legal_next(
    rng: &mut DeterministicRandom,
    chosen: &[RacingPhaseSpec],
    used_one_shot: &[&'static str],
    count: usize,
) -> Vec<RacingPhaseSpec> {
    let is_last = chosen.len() + 1 == count;
    let candidates: Vec<_> = match chosen.last() {
        None => vec![racing_phase_spec("garage").expect("garage is in the pool")],
        Some(prev) => RACING_SECTION_IDS
            .iter()
            .filter_map(|&id| racing_phase_spec(id))
            .filter(|candidate| racing_role_legal(prev, candidate))
            .filter(|candidate| !(candidate.one_shot && used_one_shot.contains(&candidate.id)))
            .filter(|candidate| candidate.id != prev.id)
            .filter(|candidate| racing_energy_legal(prev, candidate))
            .filter(|candidate| {
                if is_last {
                    candidate.role == RacingPhaseRole::Outro
                } else {
                    candidate.role != RacingPhaseRole::Outro
                }
            })
            .collect(),
    };
    rng.shuffle(&candidates)
}

fn racing_role_legal(prev: &RacingPhaseSpec, next: &RacingPhaseSpec) -> bool {
    use RacingPhaseRole::*;
    match prev.role {
        Intro => matches!(next.role, Build | Groove),
        Build => next.role == Groove,
        Groove => matches!(next.role, Groove | Build | Peak),
        Peak => matches!(next.role, Peak | Groove | Outro),
        Outro => false,
    }
}

fn racing_energy_legal(prev: &RacingPhaseSpec, next: &RacingPhaseSpec) -> bool {
    i32::abs(prev.energy as i32 - next.energy as i32) <= MAX_ENERGY_DELTA as i32
}

/// Role-aware length for the composed path: intros and outros stay at the
/// authored four bars, grooves and builds land at four or eight, and peaks
/// stretch to eight or twelve so the climax actually climbs. Bars are always a
/// multiple of the authored four-bar block, so a section re-times by tiling.
///
/// The `energy` trait (0..1) biases the stretch band continuously: at the
/// extremes it forces the short or the long band, and in the middle it leaves
/// the seeded choice in place, so the knob moves the section lengths instead of
/// re-rolling them.
pub fn racing_phase_bars(spec: &RacingPhaseSpec, seed: u32, energy: f64) -> u32 {
    let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:bars", spec.id)));
    let authored = spec.bars.max(4);
    let band = if energy >= 0.66 {
        1
    } else if energy < 0.33 {
        0
    } else {
        rng.integer(2)
    };
    match spec.role {
        RacingPhaseRole::Intro | RacingPhaseRole::Outro => authored,
        RacingPhaseRole::Build | RacingPhaseRole::Groove => authored + authored * band,
        RacingPhaseRole::Peak => authored * 2 + authored * band,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validate the hard composition rules against a form's step ids (resolved
    /// against the pool), returning the first violation.
    fn validate_racing_form(ids: &[&str], loop_from: Option<u32>) -> Result<(), String> {
        if ids.is_empty() {
            return Err("racing form is empty".into());
        }
        let specs: Vec<RacingPhaseSpec> = ids
            .iter()
            .map(|id| racing_phase_spec(id).ok_or_else(|| format!("unknown phase {id}")))
            .collect::<Result<_, _>>()?;

        // Intro first, outro last, both once.
        if specs[0].role != RacingPhaseRole::Intro {
            return Err("form must begin with the Intro".into());
        }
        if specs.last().map(|spec| spec.role) != Some(RacingPhaseRole::Outro) {
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
            if spec.role == RacingPhaseRole::Intro && index != 0 {
                return Err(format!("Intro appears at step {index}, not first"));
            }
            if spec.role == RacingPhaseRole::Outro && index != specs.len() - 1 {
                return Err(format!("Outro {} is not last", spec.id));
            }
            if let Some(prev) = index.checked_sub(1).map(|i| specs[i]) {
                if !racing_role_legal(&prev, spec) {
                    return Err(format!("illegal transition {} -> {}", prev.id, spec.id));
                }
                if !racing_energy_legal(&prev, spec) {
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
            if specs[index].role != RacingPhaseRole::Groove {
                return Err(format!("loopFrom targets non-groove {}", specs[index].id));
            }
        }
        Ok(())
    }

    #[test]
    fn metadata_derives_role_and_energy_from_plans() {
        for (id, role, energy) in [
            ("garage", RacingPhaseRole::Intro, 55),
            ("grid", RacingPhaseRole::Build, 70),
            ("cruise", RacingPhaseRole::Groove, 82),
            ("attack", RacingPhaseRole::Peak, 95),
            ("final-lap", RacingPhaseRole::Peak, 100),
            ("victory", RacingPhaseRole::Outro, 78),
        ] {
            assert_eq!(racing_phase_role(id), Some(role), "role for {id}");
            assert_eq!(racing_phase_energy(id), Some(energy), "energy for {id}");
        }
        assert_eq!(racing_phase_role("not-a-section"), None);
        assert_eq!(racing_phase_energy("not-a-section"), None);
    }

    #[test]
    fn pool_declares_every_racing_section_with_one_shot_intro_and_outro() {
        assert_eq!(
            RACING_SECTION_IDS
                .iter()
                .filter_map(|&id| racing_phase_spec(id))
                .count(),
            6
        );
        assert!(racing_phase_spec("garage").unwrap().one_shot);
        assert!(racing_phase_spec("victory").unwrap().one_shot);
        for id in ["grid", "cruise", "attack", "final-lap"] {
            assert!(!racing_phase_spec(id).unwrap().one_shot, "{id} repeats");
        }
        assert!(racing_phase_spec("attack").unwrap().energy > racing_phase_spec("cruise").unwrap().energy);
    }

    #[test]
    fn composer_is_deterministic() {
        for seed in [0xdead_beef, 0, 42, 0x51ed_0000] {
            assert_eq!(racing_compose(seed), racing_compose(seed), "seed {seed}");
        }
    }

    #[test]
    fn composer_produces_rule_legal_forms_across_many_seeds() {
        let mut distinct = std::collections::HashSet::new();
        for seed in 0..250u32 {
            let form = racing_compose(0x5eed_0000 ^ seed);
            let ids: Vec<String> = form.steps.iter().map(|step| step.section.clone()).collect();
            let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
            validate_racing_form(&id_refs, form.loop_from)
                .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
            assert!(
                (6..=9).contains(&ids.len()),
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
            racing_compose_seed("project", "level-001"),
            racing_compose_seed("project", "level-002")
        );
        assert_ne!(
            racing_compose_seed("project", "level-001"),
            racing_compose_seed("other", "level-001")
        );
    }

    #[test]
    fn phase_bars_are_role_aware_and_tile_aligned() {
        for seed in 0..64u32 {
            for id in RACING_SECTION_IDS {
                let spec = racing_phase_spec(id).unwrap();
                let bars = racing_phase_bars(&spec, seed, 0.5);
                assert_eq!(bars % 4, 0, "{id} bars {bars} not tile-aligned");
                match spec.role {
                    RacingPhaseRole::Intro | RacingPhaseRole::Outro => {
                        assert_eq!(bars, 4, "{id}")
                    }
                    RacingPhaseRole::Build | RacingPhaseRole::Groove => {
                        assert!((4..=8).contains(&bars), "{id} bars {bars}")
                    }
                    RacingPhaseRole::Peak => {
                        assert!((8..=12).contains(&bars), "{id} bars {bars}")
                    }
                }
            }
        }
    }
}
