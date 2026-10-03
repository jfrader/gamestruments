//! Shared phase pool + seeded composer for the Racing composed arrangement.
//!
//! Every section the Racing generator produces is described once here by role
//! and energy, derived from the same authored [`PLANS`](crate::racing::PLANS)
//! table that generates the sections (see [`racing_phase_role`] and
//! [`racing_phase_energy`]). Composing a Racing song is then choosing a
//! [`SongForm`] over that pool; section generation is untouched.

use crate::composer::{self, PhaseMeta, PhasePool};
use crate::racing::{racing_phase_energy, racing_phase_role, RacingPhaseRole};
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::SongForm;

/// Maximum allowed energy step between adjacent composed phases (0-100 scale).
pub const MAX_ENERGY_DELTA: u32 = 35;

/// The shared pool for the seeded composer and all-phases tour: six base
/// sections, six authored phases, and the drumless breather.
pub const RACING_SECTION_IDS: [&str; 13] = [
    "garage",
    "ignition",
    "grid",
    "breather",
    "cruise",
    "switchback",
    "slipstream",
    "attack",
    "redline",
    "open-road",
    "final-lap",
    "victory",
    "cooldown",
];

/// Game-signal sections carried by every composed score so the game can cue
/// them, but never part of the composed song form: the loss outro (`defeat`)
/// and the two race incidents (`recovery`, `wrong-way`).
pub const RACING_SIGNAL_IDS: [&str; 3] = ["defeat", "recovery", "wrong-way"];

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

impl PhaseMeta for RacingPhaseSpec {
    fn id(&self) -> &'static str {
        self.id
    }

    fn one_shot(&self) -> bool {
        self.one_shot
    }

    fn is_groove(&self) -> bool {
        self.role == RacingPhaseRole::Groove
    }

    fn is_outro(&self) -> bool {
        // Both the win outro (victory) and the post-outro release (cooldown)
        // are terminal: either may close a form, and neither appears elsewhere.
        matches!(
            self.role,
            RacingPhaseRole::Outro | RacingPhaseRole::PostOutro
        )
    }
}

/// The phase spec for a Racing section id, if it is part of the pool.
pub fn racing_phase_spec(id: &str) -> Option<RacingPhaseSpec> {
    let pool_id = RACING_SECTION_IDS
        .iter()
        .copied()
        .find(|candidate| *candidate == id)?;
    // The breather, the four authored phases and the post-outro have no base
    // plan (they are built over a source section), so their role, energy and
    // authored length live here rather than being derived from `PLANS`.
    match pool_id {
        "breather" => Some(RacingPhaseSpec {
            id: "breather",
            role: RacingPhaseRole::Breather,
            energy: 60,
            bars: 4,
            one_shot: false,
        }),
        "ignition" => Some(RacingPhaseSpec {
            id: "ignition",
            role: RacingPhaseRole::Build,
            energy: 62,
            bars: 8,
            one_shot: false,
        }),
        "slipstream" => Some(RacingPhaseSpec {
            id: "slipstream",
            role: RacingPhaseRole::Groove,
            energy: 78,
            bars: 16,
            one_shot: false,
        }),
        "redline" => Some(RacingPhaseSpec {
            id: "redline",
            role: RacingPhaseRole::Peak,
            energy: 98,
            bars: 16,
            one_shot: false,
        }),
        "cooldown" => Some(RacingPhaseSpec {
            id: "cooldown",
            role: RacingPhaseRole::PostOutro,
            energy: 55,
            bars: 8,
            one_shot: false,
        }),
        "switchback" => Some(RacingPhaseSpec {
            id: "switchback",
            role: RacingPhaseRole::Groove,
            energy: 79,
            bars: 8,
            one_shot: false,
        }),
        "open-road" => Some(RacingPhaseSpec {
            id: "open-road",
            role: RacingPhaseRole::Groove,
            energy: 68,
            bars: 16,
            one_shot: false,
        }),
        _ => Some(RacingPhaseSpec {
            id: pool_id,
            role: racing_phase_role(id)?,
            energy: racing_phase_energy(id)?,
            bars: 4,
            one_shot: matches!(id, "garage" | "victory"),
        }),
    }
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
    composer::compose::<RacingPool>(seed, ())
}

/// The Racing pool as the shared composer sees it: role/energy metadata derived
/// from the authored plans, the Racing role grammar, and the fixed 6..=12-step
/// band with an outro-or-post-outro ending.
struct RacingPool;

impl PhasePool for RacingPool {
    type Spec = RacingPhaseSpec;
    type Request = ();

    fn pool() -> Vec<RacingPhaseSpec> {
        RACING_SECTION_IDS
            .iter()
            .filter_map(|&id| racing_phase_spec(id))
            .collect()
    }

    fn role_legal(prev: RacingPhaseSpec, next: RacingPhaseSpec) -> bool {
        use RacingPhaseRole::*;
        match prev.role {
            Intro => matches!(next.role, Build | Groove),
            Build => matches!(next.role, Groove | Breather),
            Breather => matches!(next.role, Groove),
            Groove => matches!(next.role, Groove | Build | Peak | Breather),
            Peak => matches!(next.role, Peak | Groove | Outro | PostOutro),
            Outro => false,
            PostOutro => false,
        }
    }

    fn energy_legal(prev: RacingPhaseSpec, next: RacingPhaseSpec) -> bool {
        // The post-climax release (Peak -> PostOutro) may drop the whole band:
        // a resolved-down cooldown is a deliberate large step, not an error.
        if prev.role == RacingPhaseRole::Peak && next.role == RacingPhaseRole::PostOutro {
            return true;
        }
        i32::abs(prev.energy as i32 - next.energy as i32) <= MAX_ENERGY_DELTA as i32
    }

    fn ending_legal(spec: RacingPhaseSpec, _want_outro: bool) -> bool {
        // A composed song ends on the win outro or the post-outro release.
        matches!(
            spec.role,
            RacingPhaseRole::Outro | RacingPhaseRole::PostOutro
        )
    }

    fn count(_request: (), rng: &mut DeterministicRandom) -> usize {
        (6 + rng.integer(7)) as usize // 6..=12 steps, so the full pool is reachable
    }

    fn want_outro(_request: (), _rng: &mut DeterministicRandom) -> bool {
        true
    }
}

/// Role-aware length for the composed path: intros, outros and breathers stay
/// at the authored four bars, grooves and builds land at four or eight, and
/// peaks stretch to eight or twelve so the climax actually climbs. Bars are
/// always a multiple of the authored four-bar block, so a section re-times by
/// tiling.
///
/// The `energy` trait (0..1) biases the stretch band continuously: at the
/// extremes it forces the short or the long band, and in the middle it leaves
/// the seeded choice in place, so the knob moves the section lengths instead of
/// re-rolling them.
pub fn racing_phase_bars(spec: &RacingPhaseSpec, seed: u32, energy: f64) -> u32 {
    let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:bars", spec.id)));
    let authored = spec.bars.max(4);
    // The authored multi-block phases (ignition/slipstream/redline/cooldown)
    // already carry deliberate lengths; the composed path honours them as-is
    // rather than stretching an already-long phase out further.
    if authored > 4 {
        return authored;
    }
    let band = if energy >= 0.66 {
        1
    } else if energy < 0.33 {
        0
    } else {
        rng.integer(2)
    };
    match spec.role {
        RacingPhaseRole::Intro
        | RacingPhaseRole::Outro
        | RacingPhaseRole::PostOutro
        | RacingPhaseRole::Breather => authored,
        RacingPhaseRole::Build | RacingPhaseRole::Groove => authored + authored * band,
        RacingPhaseRole::Peak => authored * 2 + authored * band,
    }
}

/// The canonical all-phases tour: every composition-pool phase once, in pool
/// order, looping back to the first groove (`cruise`). Game-signal sections are
/// not toured.
pub fn racing_all_phases_form() -> SongForm {
    composer::canonical_form::<RacingPool>()
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

        // Intro first, terminal ending last, both once.
        if specs[0].role != RacingPhaseRole::Intro {
            return Err("form must begin with the Intro".into());
        }
        if !matches!(
            specs.last().map(|spec| spec.role),
            Some(RacingPhaseRole::Outro | RacingPhaseRole::PostOutro)
        ) {
            return Err("form must end with the Outro or the Post-outro".into());
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
            if matches!(
                spec.role,
                RacingPhaseRole::Outro | RacingPhaseRole::PostOutro
            ) && index != specs.len() - 1
            {
                return Err(format!("terminal {} is not last", spec.id));
            }
            if let Some(prev) = index.checked_sub(1).map(|i| specs[i]) {
                if !RacingPool::role_legal(prev, *spec) {
                    return Err(format!("illegal transition {} -> {}", prev.id, spec.id));
                }
                if !RacingPool::energy_legal(prev, *spec) {
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
            13
        );
        assert!(racing_phase_spec("garage").unwrap().one_shot);
        assert!(racing_phase_spec("victory").unwrap().one_shot);
        for id in [
            "ignition",
            "grid",
            "breather",
            "cruise",
            "switchback",
            "slipstream",
            "attack",
            "redline",
            "open-road",
            "final-lap",
            "cooldown",
        ] {
            assert!(!racing_phase_spec(id).unwrap().one_shot, "{id} repeats");
        }
        assert!(
            racing_phase_spec("attack").unwrap().energy
                > racing_phase_spec("cruise").unwrap().energy
        );
    }

    #[test]
    fn pool_specs_cover_the_new_phases_with_role_and_energy() {
        for (id, role, energy, bars) in [
            ("ignition", RacingPhaseRole::Build, 62, 8),
            ("slipstream", RacingPhaseRole::Groove, 78, 16),
            ("redline", RacingPhaseRole::Peak, 98, 16),
            ("cooldown", RacingPhaseRole::PostOutro, 55, 8),
            ("breather", RacingPhaseRole::Breather, 60, 4),
            ("switchback", RacingPhaseRole::Groove, 79, 8),
            ("open-road", RacingPhaseRole::Groove, 68, 16),
        ] {
            let spec = racing_phase_spec(id).unwrap();
            assert_eq!(spec.role, role, "{id} role");
            assert_eq!(spec.energy, energy, "{id} energy");
            assert_eq!(spec.bars, bars, "{id} bars");
            assert!(!spec.one_shot, "{id} one_shot");
        }
        // The post-outro is a terminal ending alongside victory, not a rival.
        let cooldown = racing_phase_spec("cooldown").unwrap();
        let victory = racing_phase_spec("victory").unwrap();
        assert!(RacingPool::ending_legal(cooldown, true));
        assert!(RacingPool::ending_legal(victory, true));
        assert!(RacingPool::role_legal(
            racing_phase_spec("final-lap").unwrap(),
            cooldown
        ));
        assert!(RacingPool::energy_legal(
            racing_phase_spec("final-lap").unwrap(),
            cooldown
        ));
    }

    #[test]
    fn breather_is_drumless_and_legal_before_the_flow() {
        let breather = racing_phase_spec("breather").expect("breather must be in the pool");
        let cruise = racing_phase_spec("cruise").expect("cruise must be in the pool");
        assert_eq!(breather.role, RacingPhaseRole::Breather);
        assert_eq!(cruise.role, RacingPhaseRole::Groove);
        assert!(!breather.one_shot, "breather may repeat");
        assert_eq!(breather.bars, 4, "breather stays at its authored four bars");
        // The breather is transition-legal into the flow phase (Groove).
        assert!(
            RacingPool::role_legal(breather, cruise),
            "breather must be legal before the flow phase"
        );
        // And it is reachable from the build and from the groove.
        assert!(RacingPool::role_legal(
            racing_phase_spec("grid").unwrap(),
            breather
        ));
        assert!(RacingPool::role_legal(cruise, breather));
        // Its energy sits within the step bound of its neighbours.
        for neighbour in ["grid", "cruise"] {
            assert!(
                RacingPool::energy_legal(breather, racing_phase_spec(neighbour).unwrap()),
                "breather -> {neighbour} must stay within the energy bound"
            );
        }
        // The composer can actually choose it: across many seeds at least one
        // form places the breather directly before the flow phase.
        let mut saw_breather_before_flow = false;
        for seed in 0..512u32 {
            let form = racing_compose(seed);
            let ids: Vec<&str> = form
                .steps
                .iter()
                .map(|step| step.section.as_str())
                .collect();
            if ids.windows(2).any(|w| w == ["breather", "cruise"]) {
                saw_breather_before_flow = true;
                break;
            }
        }
        assert!(
            saw_breather_before_flow,
            "the composer must be able to place breather before cruise"
        );
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
                (6..=12).contains(&ids.len()),
                "seed {seed}: count {}",
                ids.len()
            );
            assert!(form.loop_from.is_some(), "seed {seed}: missing loop point");
            distinct.insert(ids);
        }
        assert!(
            distinct.len() > 20,
            "only {} distinct forms",
            distinct.len()
        );
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
                if spec.bars > 4 {
                    // Multi-block authored phases keep their length composed.
                    assert_eq!(bars, spec.bars, "{id} bars {bars}");
                    continue;
                }
                match spec.role {
                    RacingPhaseRole::Intro
                    | RacingPhaseRole::Outro
                    | RacingPhaseRole::PostOutro
                    | RacingPhaseRole::Breather => {
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

    #[test]
    fn every_pool_phase_is_reachable_from_the_seeded_composer() {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for seed in 0..4096u32 {
            let form = racing_compose(0x4ace_0000 ^ seed);
            for step in &form.steps {
                seen.insert(step.section.clone());
            }
        }
        for id in RACING_SECTION_IDS {
            assert!(seen.contains(id), "composer never chose {id}");
        }
    }

    #[test]
    fn all_phases_form_tours_every_pool_phase() {
        let form = racing_all_phases_form();
        let ids: Vec<&str> = form
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert_eq!(ids, RACING_SECTION_IDS.to_vec());
        let loop_from = form.loop_from.unwrap() as usize;
        assert_eq!(form.steps[loop_from].section, "cruise");
        assert_eq!(
            racing_phase_spec(&form.steps[loop_from].section)
                .unwrap()
                .role,
            RacingPhaseRole::Groove
        );
    }

    #[test]
    fn contrasting_phases_have_legal_groove_transitions() {
        for id in ["switchback", "open-road"] {
            let spec = racing_phase_spec(id).expect("new phase must be in pool");
            assert_eq!(spec.role, RacingPhaseRole::Groove, "{id} role");
            assert!(!spec.one_shot, "{id} may repeat in forms");
        }
        let cruise = racing_phase_spec("cruise").unwrap();
        let switchback = racing_phase_spec("switchback").unwrap();
        let open_road = racing_phase_spec("open-road").unwrap();
        let attack = racing_phase_spec("attack").unwrap();
        assert!(RacingPool::role_legal(cruise, switchback));
        assert!(RacingPool::role_legal(switchback, cruise));
        assert!(RacingPool::role_legal(switchback, open_road));
        assert!(RacingPool::role_legal(open_road, attack));
        assert!(RacingPool::energy_legal(cruise, switchback));
        assert!(RacingPool::energy_legal(switchback, open_road));
        assert!(RacingPool::energy_legal(open_road, attack));
    }
}
