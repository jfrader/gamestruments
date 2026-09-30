//! Shared phase pool + seeded composer for the Suspense recipe.
//!
//! Every section the Suspense generator can already produce (the base
//! `generate_suspense` sections plus the Extended-only `scan-ii` / `breach-ii` /
//! `anomaly` phases) is described once here as a [`PhaseSpec`] with a role, an
//! energy, and a one-shot flag. "Composing a song" is then nothing more than
//! choosing a [`SongForm`] over that pool — section generation is untouched.
//!
//! Energy is stored on a 0-100 integer scale so the bounded-delta rule is an
//! exact integer comparison (no floating-point boundary surprises).

use crate::composer::{self, PhaseMeta, PhasePool};
use crate::rng::{hash_text, DeterministicRandom};
use crate::score::SongForm;

/// Maximum allowed energy step between adjacent phases (0-100 scale), except
/// the intentional build -> peak gesture.
pub const MAX_ENERGY_DELTA: u32 = 35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseRole {
    Intro,
    Groove,
    Break,
    Bridge,
    Build,
    Peak,
    Loop,
    Outro,
}

#[derive(Clone, Copy, Debug)]
pub struct FigureSpec {
    pub id: &'static str,
    pub steps: &'static [u8],
    pub subdivision: u8,
    pub syncopation: u8,
    pub sustain_tolerance: u8,
    pub register: &'static str,
    pub energy: (u32, u32),
    pub roles: &'static [PhaseRole],
}

pub const FIGURE_POOL: &[FigureSpec; 10] = &[
    FigureSpec {
        id: "straight-8",
        steps: &[0, 2, 4, 6, 8, 10, 12, 14],
        subdivision: 16,
        syncopation: 0,
        sustain_tolerance: 5,
        register: "mid",
        energy: (0, 100),
        roles: &[
            PhaseRole::Groove,
            PhaseRole::Build,
            PhaseRole::Peak,
            PhaseRole::Bridge,
            PhaseRole::Intro,
            PhaseRole::Outro,
            PhaseRole::Loop,
        ],
    },
    FigureSpec {
        id: "half_time",
        steps: &[0, 4, 8, 12],
        subdivision: 8,
        syncopation: 0,
        sustain_tolerance: 8,
        register: "low",
        energy: (0, 55),
        roles: &[PhaseRole::Groove, PhaseRole::Break, PhaseRole::Bridge],
    },
    FigureSpec {
        id: "tresillo",
        steps: &[0, 3, 6, 8, 11, 14],
        subdivision: 16,
        syncopation: 2,
        sustain_tolerance: 4,
        register: "mid",
        energy: (25, 75),
        roles: &[PhaseRole::Groove, PhaseRole::Build, PhaseRole::Peak],
    },
    FigureSpec {
        id: "broken-beat",
        steps: &[0, 2, 5, 7, 9, 11, 14],
        subdivision: 16,
        syncopation: 3,
        sustain_tolerance: 3,
        register: "mid",
        energy: (40, 80),
        roles: &[PhaseRole::Groove, PhaseRole::Peak, PhaseRole::Bridge],
    },
    FigureSpec {
        id: "heavy-sync",
        steps: &[0, 4, 6, 9, 11, 13, 15],
        subdivision: 16,
        syncopation: 3,
        sustain_tolerance: 2,
        register: "high",
        energy: (45, 90),
        roles: &[PhaseRole::Groove, PhaseRole::Build, PhaseRole::Peak],
    },
    FigureSpec {
        id: "polyrhythm",
        steps: &[0, 2, 3, 5, 7, 8, 10, 12, 13, 15],
        subdivision: 16,
        syncopation: 2,
        sustain_tolerance: 3,
        register: "mid",
        energy: (35, 85),
        roles: &[PhaseRole::Groove, PhaseRole::Build, PhaseRole::Peak],
    },
    FigureSpec {
        id: "phasing",
        steps: &[0, 1, 4, 5, 8, 9, 12, 13],
        subdivision: 16,
        syncopation: 1,
        sustain_tolerance: 4,
        register: "high",
        energy: (30, 70),
        roles: &[PhaseRole::Groove, PhaseRole::Bridge],
    },
    FigureSpec {
        id: "augmentation",
        steps: &[0, 8],
        subdivision: 8,
        syncopation: 0,
        sustain_tolerance: 9,
        register: "low",
        energy: (0, 45),
        roles: &[
            PhaseRole::Groove,
            PhaseRole::Break,
            PhaseRole::Intro,
            PhaseRole::Outro,
            PhaseRole::Bridge,
        ],
    },
    FigureSpec {
        id: "double-time",
        steps: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        subdivision: 16,
        syncopation: 1,
        sustain_tolerance: 1,
        register: "high",
        energy: (55, 100),
        roles: &[PhaseRole::Groove, PhaseRole::Build, PhaseRole::Peak],
    },
    FigureSpec {
        id: "sparse",
        steps: &[0, 6],
        subdivision: 8,
        syncopation: 0,
        sustain_tolerance: 7,
        register: "mid",
        energy: (0, 35),
        roles: &[
            PhaseRole::Groove,
            PhaseRole::Break,
            PhaseRole::Intro,
            PhaseRole::Outro,
            PhaseRole::Bridge,
        ],
    },
];

pub fn figure_spec(id: &str) -> Option<&'static FigureSpec> {
    FIGURE_POOL.iter().find(|f| f.id == id)
}

#[derive(Clone, Copy, Debug)]
pub struct PhaseSpec {
    pub id: &'static str,
    pub role: PhaseRole,
    pub energy: u32,
    pub one_shot: bool,
    /// Authored length in bars (4, 8, 16, or 32). The composer ignores it — its
    /// energy/role grammar works purely on the metadata above — but the pool
    /// path honours it when it builds the actual section, so a song form can
    /// carry mixed-length phases.
    pub bars: u32,
    /// Preferred figure for this phase. Actual figure chosen by seeded
    /// selection among role+energy matches.
    pub figure: &'static str,
}

impl PhaseMeta for PhaseSpec {
    fn id(&self) -> &'static str {
        self.id
    }

    fn one_shot(&self) -> bool {
        self.one_shot
    }

    fn is_groove(&self) -> bool {
        self.role == PhaseRole::Groove
    }

    fn is_outro(&self) -> bool {
        self.role == PhaseRole::Outro
    }
}

/// The pool, in canonical play order (the natural song arc, Extended phases
/// slotted where `generate_extended` would place them, new grooves and
/// breaks/bridges interleaved by energy, then `outro` / `coda`).
pub const PHASE_POOL: &[PhaseSpec; 38] = &[
    PhaseSpec {
        id: "intro",
        role: PhaseRole::Intro,
        energy: 15,
        one_shot: true,
        bars: 8,
        figure: "sparse",
    },
    PhaseSpec {
        id: "verse",
        role: PhaseRole::Groove,
        energy: 45,
        one_shot: false,
        bars: 16,
        figure: "straight-8",
    },
    PhaseSpec {
        id: "half-time",
        role: PhaseRole::Groove,
        energy: 38,
        one_shot: false,
        bars: 8,
        figure: "half_time",
    },
    PhaseSpec {
        id: "scan-ii",
        role: PhaseRole::Groove,
        energy: 45,
        one_shot: false,
        bars: 16,
        figure: "straight-8",
    },
    PhaseSpec {
        id: "sparse",
        role: PhaseRole::Groove,
        energy: 22,
        one_shot: false,
        bars: 4,
        figure: "sparse",
    },
    PhaseSpec {
        id: "sub-groove",
        role: PhaseRole::Groove,
        energy: 50,
        one_shot: false,
        bars: 8,
        figure: "broken-beat",
    },
    PhaseSpec {
        id: "pre-chorus",
        role: PhaseRole::Build,
        energy: 60,
        one_shot: false,
        bars: 8,
        figure: "tresillo",
    },
    PhaseSpec {
        id: "chorus",
        role: PhaseRole::Peak,
        energy: 80,
        one_shot: false,
        bars: 16,
        figure: "double-time",
    },
    PhaseSpec {
        id: "breach-ii",
        role: PhaseRole::Peak,
        energy: 85,
        one_shot: false,
        bars: 16,
        figure: "heavy-sync",
    },
    PhaseSpec {
        id: "syncopated",
        role: PhaseRole::Groove,
        energy: 55,
        one_shot: false,
        bars: 8,
        figure: "heavy-sync",
    },
    PhaseSpec {
        id: "break",
        role: PhaseRole::Break,
        energy: 20,
        one_shot: false,
        bars: 4,
        figure: "augmentation",
    },
    PhaseSpec {
        id: "drum-break",
        role: PhaseRole::Break,
        energy: 45,
        one_shot: false,
        bars: 4,
        figure: "sparse",
    },
    PhaseSpec {
        id: "verse-b",
        role: PhaseRole::Groove,
        energy: 50,
        one_shot: false,
        bars: 8,
        figure: "straight-8",
    },
    PhaseSpec {
        id: "drive",
        role: PhaseRole::Groove,
        energy: 68,
        one_shot: false,
        bars: 8,
        figure: "double-time",
    },
    PhaseSpec {
        id: "post-chorus",
        role: PhaseRole::Groove,
        energy: 55,
        one_shot: false,
        bars: 8,
        figure: "polyrhythm",
    },
    PhaseSpec {
        id: "false-stop",
        role: PhaseRole::Break,
        energy: 12,
        one_shot: false,
        bars: 4,
        figure: "augmentation",
    },
    PhaseSpec {
        id: "interlude",
        role: PhaseRole::Groove,
        energy: 30,
        one_shot: false,
        bars: 4,
        figure: "half_time",
    },
    PhaseSpec {
        id: "filter-break",
        role: PhaseRole::Break,
        energy: 28,
        one_shot: false,
        bars: 8,
        figure: "sparse",
    },
    PhaseSpec {
        id: "bridge",
        role: PhaseRole::Bridge,
        energy: 50,
        one_shot: false,
        bars: 16,
        figure: "phasing",
    },
    PhaseSpec {
        id: "harmonic-bridge",
        role: PhaseRole::Bridge,
        energy: 62,
        one_shot: false,
        bars: 8,
        figure: "phasing",
    },
    PhaseSpec {
        id: "bridge-b",
        role: PhaseRole::Bridge,
        energy: 55,
        one_shot: false,
        bars: 8,
        figure: "tresillo",
    },
    PhaseSpec {
        id: "step-up-bridge",
        role: PhaseRole::Bridge,
        energy: 70,
        one_shot: false,
        bars: 8,
        figure: "polyrhythm",
    },
    PhaseSpec {
        id: "solo",
        role: PhaseRole::Build,
        energy: 70,
        one_shot: false,
        bars: 8,
        figure: "heavy-sync",
    },
    PhaseSpec {
        id: "anomaly",
        role: PhaseRole::Build,
        energy: 60,
        one_shot: true,
        bars: 8,
        figure: "broken-beat",
    },
    PhaseSpec {
        id: "chorus-final",
        role: PhaseRole::Peak,
        energy: 90,
        one_shot: false,
        bars: 8,
        figure: "double-time",
    },
    PhaseSpec {
        id: "theme-ride",
        role: PhaseRole::Groove,
        energy: 68,
        one_shot: false,
        bars: 8,
        figure: "straight-8",
    },
    // The match phases (see `match_phases`): a strategy game's arc.
    PhaseSpec {
        id: "build",
        role: PhaseRole::Groove,
        energy: 40,
        one_shot: false,
        bars: 32,
        figure: "straight-8",
    },
    PhaseSpec {
        id: "scout",
        role: PhaseRole::Groove,
        energy: 35,
        one_shot: false,
        bars: 16,
        figure: "sparse",
    },
    PhaseSpec {
        id: "expand",
        role: PhaseRole::Groove,
        energy: 55,
        one_shot: false,
        bars: 32,
        figure: "straight-8",
    },
    PhaseSpec {
        id: "research",
        role: PhaseRole::Build,
        energy: 50,
        one_shot: true,
        bars: 16,
        figure: "double-time",
    },
    PhaseSpec {
        id: "raid",
        role: PhaseRole::Groove,
        energy: 65,
        one_shot: false,
        bars: 16,
        figure: "broken-beat",
    },
    PhaseSpec {
        id: "tension",
        role: PhaseRole::Break,
        energy: 30,
        one_shot: true,
        bars: 16,
        figure: "sparse",
    },
    PhaseSpec {
        id: "siege",
        role: PhaseRole::Bridge,
        energy: 60,
        one_shot: true,
        bars: 32,
        figure: "heavy-sync",
    },
    PhaseSpec {
        id: "battle",
        role: PhaseRole::Peak,
        energy: 90,
        one_shot: true,
        bars: 32,
        figure: "double-time",
    },
    PhaseSpec {
        id: "victory",
        role: PhaseRole::Outro,
        energy: 40,
        one_shot: true,
        bars: 16,
        figure: "half_time",
    },
    PhaseSpec {
        id: "defeat",
        role: PhaseRole::Outro,
        energy: 15,
        one_shot: true,
        bars: 16,
        figure: "sparse",
    },
    PhaseSpec {
        id: "outro",
        role: PhaseRole::Outro,
        energy: 20,
        one_shot: true,
        bars: 4,
        figure: "augmentation",
    },
    PhaseSpec {
        id: "coda",
        role: PhaseRole::Outro,
        energy: 20,
        one_shot: true,
        bars: 4,
        figure: "sparse",
    },
];

/// Derive the take seed for a reel position: an index-addressed hash of the
/// project secret and level seed. Not a chained hash, so any take is reachable
/// directly (a caller can jump straight to take 7). The `domain` must be unique
/// to this derivation — never a seed domain used anywhere else in the engine.
pub fn take_seed(secret: &str, seed: &str, domain: &str, index: u32) -> u32 {
    hash_text(&format!("{secret}\0{seed}\0{domain}\0{index}"))
}

pub fn phase_spec(id: &str) -> Option<&'static PhaseSpec> {
    PHASE_POOL.iter().find(|spec| spec.id == id)
}

/// The composer intent: selects the shape (section count and ending).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    Loop,
    Arc,
    Long,
    Surprise,
}

impl Intent {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "arc" => Ok(Self::Arc),
            "loop" => Ok(Self::Loop),
            "long" => Ok(Self::Long),
            "surprise" => Ok(Self::Surprise),
            other => Err(format!("Unknown suspense intent: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Arc => "arc",
            Self::Long => "long",
            Self::Surprise => "surprise",
        }
    }

    fn count_range(self) -> (u32, u32) {
        match self {
            Self::Loop => (5, 7),
            Self::Arc => (8, 10),
            Self::Long => (11, 14),
            Self::Surprise => (5, 14),
        }
    }
}

/// Every pool phase exactly once, in canonical order, looping back to the first
/// Groove.
pub fn all_phases_form() -> SongForm {
    composer::canonical_form::<SuspensePool>()
}

/// Compose a song from the pool. The seed decides the section count, the role
/// selection, the order, and the loop point.
pub fn compose(seed: u32, intent: Intent) -> SongForm {
    composer::compose::<SuspensePool>(seed, intent)
}

/// The Suspense pool as the shared composer sees it: role/energy metadata over
/// [`PHASE_POOL`], the Suspense role grammar, and the intent-driven count band
/// and outro preference.
struct SuspensePool;

impl PhasePool for SuspensePool {
    type Spec = PhaseSpec;
    type Request = Intent;

    fn pool() -> Vec<PhaseSpec> {
        PHASE_POOL.to_vec()
    }

    fn role_legal(prev: PhaseSpec, next: PhaseSpec) -> bool {
        use PhaseRole::*;
        match prev.role {
            Intro => next.role == Groove,
            Groove => matches!(next.role, Groove | Break | Bridge | Build | Outro),
            Break | Bridge => next.role == Groove,
            Build => next.role == Peak,
            Peak => matches!(next.role, Groove | Outro),
            Outro | Loop => false,
        }
    }

    fn energy_legal(prev: PhaseSpec, next: PhaseSpec) -> bool {
        if prev.role == PhaseRole::Build && next.role == PhaseRole::Peak {
            return true; // the intentional build -> peak gesture
        }
        i32::abs(prev.energy as i32 - next.energy as i32) <= MAX_ENERGY_DELTA as i32
    }

    fn ending_legal(spec: PhaseSpec, want_outro: bool) -> bool {
        if want_outro {
            spec.role == PhaseRole::Outro
        } else {
            spec.role == PhaseRole::Groove
        }
    }

    fn count(request: Intent, rng: &mut DeterministicRandom) -> usize {
        let (low, high) = request.count_range();
        (low + rng.integer(high - low + 1)) as usize
    }

    fn want_outro(request: Intent, rng: &mut DeterministicRandom) -> bool {
        match request {
            Intent::Arc | Intent::Long => true,
            Intent::Loop => false,
            Intent::Surprise => rng.integer(2) == 0,
        }
    }

    fn retry_want_outro() -> bool {
        // An outro ending is unreachable for the shortest Surprise forms (e.g.
        // count 5 has no room for the required groove before the outro), so
        // retry with a groove ending, which is always reachable.
        false
    }

    fn refine(
        chosen: &[PhaseSpec],
        candidates: Vec<PhaseSpec>,
        root_seed: u32,
    ) -> Vec<PhaseSpec> {
        // Budgeted only: contrast/groove rules. If empty for a state the
        // backtracker will not take the path; grammar + bands keep at least one
        // always reachable.
        candidates
            .into_iter()
            .filter(|c| groove_run_allows_change(chosen, *c, root_seed))
            .filter(|c| build_legal_allows_figure(chosen, *c, root_seed))
            .collect()
    }
}

fn figure_family(id: &str) -> &'static str {
    match id {
        "straight-8" | "augmentation" => "even",
        "half_time" => "half",
        "double-time" => "dense",
        "tresillo" | "polyrhythm" | "phasing" => "poly",
        "broken-beat" | "heavy-sync" => "sync",
        "sparse" => "sparse",
        _ => "other",
    }
}

fn density_band(onsets: usize) -> u8 {
    match onsets {
        0..=2 => 0,
        3..=5 => 1,
        6..=8 => 2,
        _ => 3,
    }
}

pub(crate) fn figure_for_composition(phase_id: &str, root_seed: u32) -> &'static FigureSpec {
    let spec = phase_spec(phase_id).expect("phase exists");
    let mut rng = DeterministicRandom::new(root_seed ^ hash_text(&format!("{}:figure", phase_id)));
    let cands: Vec<_> = FIGURE_POOL
        .iter()
        .filter(|f| {
            f.roles.contains(&spec.role) && spec.energy >= f.energy.0 && spec.energy <= f.energy.1
        })
        .collect();
    if cands.is_empty() {
        // Fallback to preferred if defined, else first matching role
        if let Some(pref) = figure_spec(spec.figure) {
            if pref.roles.contains(&spec.role) {
                return pref;
            }
        }
        return FIGURE_POOL
            .iter()
            .find(|f| f.roles.contains(&spec.role))
            .unwrap_or(&FIGURE_POOL[0]);
    }
    let idx = rng.integer(cands.len() as u32) as usize;
    cands[idx]
}

pub(crate) fn contrast_attrs_differ(a: &FigureSpec, b: &FigureSpec) -> usize {
    let mut d = 0;
    if figure_family(a.id) != figure_family(b.id) {
        d += 1;
    }
    if a.subdivision != b.subdivision {
        d += 1;
    }
    if density_band(a.steps.len()) != density_band(b.steps.len()) {
        d += 1;
    }
    if a.register != b.register {
        d += 1;
    }
    d
}

fn groove_run_allows_change(
    chosen: &[PhaseSpec],
    candidate: PhaseSpec,
    root_seed: u32,
) -> bool {
    if candidate.role != PhaseRole::Groove {
        return true;
    }
    let mut recent_figs: Vec<&'static str> = Vec::new();
    for spec in chosen.iter().rev() {
        if spec.role == PhaseRole::Groove {
            recent_figs.push(figure_for_composition(spec.id, root_seed).id);
        } else if !recent_figs.is_empty() {
            break;
        }
    }
    recent_figs.reverse();
    let cand_fig = figure_for_composition(candidate.id, root_seed).id;
    let would: Vec<_> = recent_figs
        .iter()
        .chain(std::iter::once(&cand_fig))
        .copied()
        .collect();
    if would.len() < 2 {
        return true;
    }
    let changes = would.windows(2).filter(|w| w[0] != w[1]).count();
    // ≤ 1 figure change per 2 phases in groove run
    changes <= would.len() / 2
}

fn build_legal_allows_figure(
    chosen: &[PhaseSpec],
    candidate: PhaseSpec,
    root_seed: u32,
) -> bool {
    let prev = chosen.last().copied().unwrap_or(candidate);
    let prev_fig = figure_for_composition(prev.id, root_seed);
    let cand_fig = figure_for_composition(candidate.id, root_seed);
    // Build/Peak inherit subdivision from preceding Groove
    if matches!(candidate.role, PhaseRole::Build | PhaseRole::Peak) {
        if let Some(gs) = last_groove_subdivision(chosen, root_seed) {
            if gs != cand_fig.subdivision {
                return false;
            }
        }
    }
    // Contrast budget only enforced on groove-to-groove (and build/peak inherit); breaks/bridges foreign ok, other transits loose
    let groove_to_groove = prev.role == PhaseRole::Groove && candidate.role == PhaseRole::Groove;
    if groove_to_groove && contrast_attrs_differ(prev_fig, cand_fig) > 1 {
        return false;
    }
    true
}

fn last_groove_subdivision(chosen: &[PhaseSpec], root_seed: u32) -> Option<u8> {
    for spec in chosen.iter().rev() {
        if spec.role == PhaseRole::Groove {
            return Some(figure_for_composition(spec.id, root_seed).subdivision);
        }
    }
    None
}

/// Validate the hard composition rules against a form's step ids (resolved
/// against the pool). Returns an error describing the first violation.
#[cfg(test)]
pub(crate) fn validate_form_rules(sections: &[&str], loop_from: Option<u32>) -> Result<(), String> {
    if sections.is_empty() {
        return Err("song form is empty".into());
    }
    let specs: Vec<&'static PhaseSpec> = sections
        .iter()
        .map(|id| phase_spec(id).ok_or_else(|| format!("unknown phase {id}")))
        .collect::<Result<_, _>>()?;

    // Intro only first.
    if specs[0].role != PhaseRole::Intro {
        return Err("form must begin with Intro".into());
    }
    for (index, spec) in specs.iter().enumerate().skip(1) {
        if spec.role == PhaseRole::Intro {
            return Err(format!("Intro appears at step {index}, not first"));
        }
    }

    // One-shot phases appear at most once.
    let mut seen_one_shot: Vec<&str> = Vec::new();
    for spec in &specs {
        if spec.one_shot {
            if seen_one_shot.contains(&spec.id) {
                return Err(format!("one-shot phase {} repeats", spec.id));
            }
            seen_one_shot.push(spec.id);
        }
    }

    for index in 0..specs.len() {
        let spec = specs[index];
        let prev = index.checked_sub(1).map(|i| specs[i]);
        let next = specs.get(index + 1).copied();
        match spec.role {
            PhaseRole::Peak => match prev {
                Some(previous) if previous.role == PhaseRole::Build => {}
                _ => return Err(format!("Peak {} is not preceded by Build", spec.id)),
            },
            PhaseRole::Break | PhaseRole::Bridge => {
                let Some(previous) = prev else {
                    return Err(format!("{} cannot be first", spec.id));
                };
                let Some(following) = next else {
                    return Err(format!("{} cannot be last", spec.id));
                };
                if previous.role != PhaseRole::Groove || following.role != PhaseRole::Groove {
                    return Err(format!("{} must sit between Grooves", spec.id));
                }
            }
            PhaseRole::Outro => {
                if index != specs.len() - 1 {
                    return Err(format!("Outro {} is not last", spec.id));
                }
            }
            _ => {}
        }
        if let Some(previous) = prev {
            if !SuspensePool::energy_legal(*previous, *spec) {
                return Err(format!(
                    "energy jump {} -> {} exceeds bound",
                    previous.id, spec.id
                ));
            }
        }
    }

    if let Some(loop_from) = loop_from {
        let index = loop_from as usize;
        if index >= specs.len() {
            return Err(format!("loopFrom {loop_from} is out of range"));
        }
        if specs[index].role != PhaseRole::Groove || specs[index].one_shot {
            return Err(format!("loopFrom targets non-groove {}", specs[index].id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_phases_covers_the_pool_once_and_loops_from_the_first_groove() {
        let form = all_phases_form();
        let ids: Vec<&str> = form
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert_eq!(
            ids,
            PHASE_POOL.iter().map(|spec| spec.id).collect::<Vec<_>>()
        );
        let loop_from = form.loop_from.unwrap() as usize;
        assert_eq!(form.steps[loop_from].section, "verse");
        assert_eq!(PHASE_POOL[loop_from].role, PhaseRole::Groove);
    }

    #[test]
    fn take_seed_is_index_addressed_and_never_reuses_an_existing_domain() {
        // Same inputs -> same seed (deterministic, no hidden chaining).
        assert_eq!(
            take_seed("project", "level-001", "suspense-reel-compose-1", 0),
            take_seed("project", "level-001", "suspense-reel-compose-1", 0)
        );
        // Different index -> different seed, and jumping straight to a high
        // take is fine (no upper bound assumed).
        for index in [0u32, 1, 2, 7, 99, 1_000_000] {
            assert_ne!(
                take_seed("project", "level-001", "suspense-reel-compose-1", index),
                take_seed("project", "level-001", "suspense-reel-compose-1", index + 1),
                "take {index} must differ from take {}",
                index + 1
            );
        }
        // The secret participates: changing it changes every take.
        assert_ne!(
            take_seed("project", "level-001", "suspense-reel-compose-1", 7),
            take_seed("other", "level-001", "suspense-reel-compose-1", 7)
        );
        // Distinct domains for compose vs surface stay independent.
        assert_ne!(
            take_seed("project", "level-001", "suspense-reel-compose-1", 7),
            take_seed("project", "level-001", "suspense-reel-surface-1", 7)
        );
    }

    #[test]
    fn pool_declares_mixed_lengths_including_16_and_4_bars() {
        assert!(PHASE_POOL.iter().any(|spec| spec.bars == 16));
        assert!(PHASE_POOL.iter().any(|spec| spec.bars == 4));
        assert!(PHASE_POOL
            .iter()
            .all(|spec| matches!(spec.bars, 4 | 8 | 16 | 32)));
        for phase in &crate::match_phases::MATCH_PHASES {
            let spec = phase_spec(phase.id).expect("every match phase is in the pool");
            assert_eq!(spec.bars, phase.bars(), "{} length", phase.id);
        }
        assert!(PHASE_POOL.iter().all(|spec| spec.energy <= 100));
    }

    #[test]
    fn composer_is_deterministic() {
        for intent in [Intent::Loop, Intent::Arc, Intent::Long, Intent::Surprise] {
            let first = compose(0xdead_beef, intent);
            let second = compose(0xdead_beef, intent);
            assert_eq!(first, second, "{intent:?}");
        }
    }

    #[test]
    fn composer_produces_rule_legal_forms_across_many_seeds() {
        let mut counts = std::collections::HashSet::new();
        for intent in [Intent::Loop, Intent::Arc, Intent::Long, Intent::Surprise] {
            for seed in 0..250u32 {
                let form = compose(0x51ed_0000 ^ seed, intent);
                let ids: Vec<&str> = form
                    .steps
                    .iter()
                    .map(|step| step.section.as_str())
                    .collect();
                validate_form_rules(&ids, form.loop_from)
                    .unwrap_or_else(|error| panic!("{intent:?} seed {seed}: {error}"));
                let (low, high) = intent.count_range();
                assert!(
                    (low..=high).contains(&(ids.len() as u32)),
                    "{intent:?} seed {seed}: count {}",
                    ids.len()
                );
                assert!(
                    form.loop_from.is_some(),
                    "{intent:?} seed {seed}: missing loop point"
                );
                counts.insert(ids.iter().map(|id| (*id).to_string()).collect::<Vec<_>>());
            }
        }
        // Four intents, each with a wide variety of forms over 250 seeds.
        assert!(counts.len() > 200, "only {} distinct forms", counts.len());
    }

    /// The rhythm vocabulary has to be real: enough figures to avoid repeating
    /// transitions, all of them reachable, and no phase locked to one figure.
    #[test]
    fn figure_vocabulary_is_used_and_varies_per_seed() {
        assert!(FIGURE_POOL.len() >= 8, "only {} figures", FIGURE_POOL.len());
        let preferred: std::collections::BTreeSet<&str> =
            PHASE_POOL.iter().map(|spec| spec.figure).collect();
        assert!(
            preferred.len() >= 8,
            "only {} distinct preferred figures: {preferred:?}",
            preferred.len()
        );
        for id in ["verse", "chorus", "bridge", "drum-break"] {
            let picks: std::collections::BTreeSet<&str> = (0..16u32)
                .map(|seed| figure_for_composition(id, 0x9000_0000 ^ seed).id)
                .collect();
            assert!(picks.len() >= 2, "{id} always gets {picks:?}");
        }
    }

    /// `theme-ride` must be a real, reachable phase: a non-one-shot groove with
    /// legal predecessors and successors, so both the canonical tour and the
    /// seeded composer can place it.
    #[test]
    fn theme_ride_is_a_reachable_groove() {
        let ride = phase_spec("theme-ride").expect("theme-ride must be in the pool");
        assert_eq!(ride.role, PhaseRole::Groove);
        assert!(!ride.one_shot, "a ride the composer can loop must not be one-shot");
        assert_eq!(ride.bars, 8);
        assert!(ride.energy <= 100);

        let predecessors = PHASE_POOL
            .iter()
            .filter(|spec| {
                SuspensePool::role_legal(**spec, *ride) && SuspensePool::energy_legal(**spec, *ride)
            })
            .count();
        let successors = PHASE_POOL
            .iter()
            .filter(|spec| {
                SuspensePool::role_legal(*ride, **spec) && SuspensePool::energy_legal(*ride, **spec)
            })
            .count();
        assert!(predecessors >= 1, "no phase can precede theme-ride");
        assert!(successors >= 1, "theme-ride cannot precede any phase");

        // Its preferred figure must be a legal match for its role and energy.
        let figure = figure_for_composition("theme-ride", 0x51ed_0000);
        assert!(figure.roles.contains(&PhaseRole::Groove));
        assert!(ride.energy >= figure.energy.0 && ride.energy <= figure.energy.1);
    }

    /// Adjacent Grooves stay inside the contrast budget: at most one of
    /// {family, subdivision, density band, register} may change between them.
    #[test]
    fn adjacent_grooves_stay_within_the_contrast_budget() {
        for intent in [Intent::Loop, Intent::Arc, Intent::Long, Intent::Surprise] {
            for seed in 0..64u32 {
                let root_seed = 0x51ed_0000 ^ seed;
                let form = compose(root_seed, intent);
                let ids: Vec<&str> = form
                    .steps
                    .iter()
                    .map(|step| step.section.as_str())
                    .collect();
                for pair in ids.windows(2) {
                    let (Some(left), Some(right)) = (phase_spec(pair[0]), phase_spec(pair[1]))
                    else {
                        continue;
                    };
                    if left.role != PhaseRole::Groove || right.role != PhaseRole::Groove {
                        continue;
                    }
                    let differs = contrast_attrs_differ(
                        figure_for_composition(left.id, root_seed),
                        figure_for_composition(right.id, root_seed),
                    );
                    assert!(
                        differs <= 1,
                        "{} -> {} differs in {differs} attributes ({intent:?} seed {seed})",
                        left.id,
                        right.id
                    );
                }
            }
        }
    }
}
