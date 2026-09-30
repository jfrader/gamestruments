//! The Cozy arrangement selector: the state-driven default, the whole day in
//! clock order, and the seeded path that composes a day and lets each
//! section's layers enter and leave.

use crate::development::develop_section;
use crate::score::{MusicEvent, PortableScore};

use super::composition::CozyPhaseRole;
use super::pool::{cozy_canonical_form, cozy_compose, cozy_compose_seed, cozy_phase_spec};
use super::{generate_cozy, CozyInput};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CozyArrangement {
    /// The sections with no form: the game's state picks them.
    #[default]
    Original,
    /// Every section once in clock order, looping back to the morning.
    AllPhases,
    /// A seed-composed day with development inside each section.
    Seeded,
}

impl CozyArrangement {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" => Ok(Self::Original),
            "all-phases" => Ok(Self::AllPhases),
            "seeded" => Ok(Self::Seeded),
            other => Err(format!("Unknown cozy arrangement: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::AllPhases => "all-phases",
            Self::Seeded => "seeded",
        }
    }
}

pub fn generate_cozy_arrangement(
    input: &CozyInput,
    arrangement: CozyArrangement,
) -> Result<PortableScore, String> {
    let mut score = generate_cozy(input)?;
    match arrangement {
        CozyArrangement::Original => return Ok(score),
        CozyArrangement::AllPhases => {
            score.form = Some(cozy_canonical_form());
            score.id.push_str("-all-phases");
            score.title.push_str(" — All phases");
        }
        CozyArrangement::Seeded => {
            let seed = cozy_compose_seed(&input.secret, &input.seed);
            score.form = Some(cozy_compose(seed));
            let bar = score.bar_ticks();
            for section in &mut score.sections {
                let Some(spec) = cozy_phase_spec(&section.id) else {
                    continue;
                };
                develop_section(
                    section,
                    bar,
                    arc_for_role(spec.role),
                    layer_rank,
                    seed,
                    true,
                    0,
                    1,
                );
            }
            score.id.push_str("-seeded");
            score.title.push_str(" — Seeded");
        }
    }
    score.validate()?;
    Ok(score)
}

/// The pad is the bed and never leaves; bass and comp hold the groove; drums
/// and the answering voice come and go; the melody enters last.
fn layer_rank(event: &MusicEvent) -> u8 {
    match event {
        MusicEvent::Note { lane, .. } => match lane.as_str() {
            "pad" => 0,
            "bass" | "comp" => 1,
            "melody" => 3,
            _ => 2,
        },
        MusicEvent::Percussion { .. } => 2,
    }
}

fn arc_for_role(role: CozyPhaseRole) -> &'static [u8] {
    match role {
        CozyPhaseRole::Intro => &[1, 2, 3, 3],
        CozyPhaseRole::Groove => &[2, 3, 4, 3],
        CozyPhaseRole::Peak => &[3, 4, 4, 4],
        CozyPhaseRole::Break => &[3, 2, 2, 3],
        CozyPhaseRole::Outro => &[3, 2, 2, 1],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cozy::CozyStyle;

    fn input(seed: &str) -> CozyInput {
        CozyInput {
            secret: String::new(),
            seed: seed.into(),
            style: CozyStyle::Bossa,
            warmth: 0.5,
            bustle: 0.5,
            jazz: 0.5,
            swing: 0.5,
        }
    }

    #[test]
    fn parse_contract() {
        for arrangement in [
            CozyArrangement::Original,
            CozyArrangement::AllPhases,
            CozyArrangement::Seeded,
        ] {
            assert_eq!(
                CozyArrangement::parse(arrangement.as_str()),
                Ok(arrangement)
            );
        }
        assert_eq!(CozyArrangement::parse(""), Ok(CozyArrangement::Original));
        assert!(CozyArrangement::parse("composed").is_err());
    }

    #[test]
    fn arrangements_share_the_sections_and_differ_in_form() {
        let original = generate_cozy_arrangement(&input("a"), CozyArrangement::Original).unwrap();
        let all = generate_cozy_arrangement(&input("a"), CozyArrangement::AllPhases).unwrap();
        let seeded = generate_cozy_arrangement(&input("a"), CozyArrangement::Seeded).unwrap();
        assert!(original.form.is_none());
        assert_eq!(
            serde_json::to_vec(&original.sections).unwrap(),
            serde_json::to_vec(&all.sections).unwrap()
        );
        let tour: Vec<&str> = all
            .form
            .as_ref()
            .unwrap()
            .steps
            .iter()
            .map(|s| s.section.as_str())
            .collect();
        assert_eq!(tour.first(), Some(&"dawn"));
        assert_eq!(all.form.as_ref().unwrap().loop_from, Some(1));
        assert!(seeded.id.ends_with("-seeded"));
        let events =
            |score: &PortableScore| score.sections.iter().map(|s| s.events.len()).sum::<usize>();
        assert!(
            events(&seeded) < events(&original),
            "development thins some blocks"
        );
    }
}
