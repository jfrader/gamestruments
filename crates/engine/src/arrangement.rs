use crate::racing_arrangement::EXTENDED_SECTION_ORDER;
use crate::score::{FormOrigin, PortableScore, SongForm, SongFormStep};

const AUTOPLAY_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrangementRecipe {
    Racing,
    RacingExtended,
    Adventure,
    Cozy,
}

#[derive(Clone, Copy)]
struct TourStep {
    section: &'static str,
    repeats: u32,
}

struct Tour {
    steps: &'static [TourStep],
    loop_from: u32,
}

const RACING_TOUR: Tour = Tour {
    steps: &[
        TourStep {
            section: "garage",
            repeats: 1,
        },
        TourStep {
            section: "grid",
            repeats: 1,
        },
        TourStep {
            section: "cruise",
            repeats: 2,
        },
        TourStep {
            section: "attack",
            repeats: 1,
        },
        TourStep {
            section: "final-lap",
            repeats: 1,
        },
        TourStep {
            section: "victory",
            repeats: 1,
        },
    ],
    loop_from: 1,
};

const fn racing_extended_steps() -> [TourStep; EXTENDED_SECTION_ORDER.len()] {
    let mut steps = [TourStep {
        section: "",
        repeats: 1,
    }; EXTENDED_SECTION_ORDER.len()];
    let mut index = 0;
    while index < EXTENDED_SECTION_ORDER.len() {
        steps[index].section = EXTENDED_SECTION_ORDER[index];
        index += 1;
    }
    steps
}

static RACING_EXTENDED_STEPS: [TourStep; EXTENDED_SECTION_ORDER.len()] = racing_extended_steps();

const RACING_EXTENDED_TOUR: Tour = Tour {
    steps: &RACING_EXTENDED_STEPS,
    loop_from: 2,
};

const ADVENTURE_TOUR: Tour = Tour {
    steps: &[
        TourStep {
            section: "camp",
            repeats: 1,
        },
        TourStep {
            section: "explore",
            repeats: 1,
        },
        TourStep {
            section: "town",
            repeats: 1,
        },
        TourStep {
            section: "dungeon",
            repeats: 1,
        },
        TourStep {
            section: "combat",
            repeats: 1,
        },
        TourStep {
            section: "boss",
            repeats: 1,
        },
        TourStep {
            section: "sanctuary",
            repeats: 1,
        },
        TourStep {
            section: "victory",
            repeats: 1,
        },
    ],
    loop_from: 1,
};

/// A day in the village: first light, the working day, golden hour and night,
/// looping back into the morning.
const COZY_TOUR: Tour = Tour {
    steps: &[
        TourStep {
            section: "dawn",
            repeats: 1,
        },
        TourStep {
            section: "morning",
            repeats: 1,
        },
        TourStep {
            section: "noon",
            repeats: 1,
        },
        TourStep {
            section: "market",
            repeats: 1,
        },
        TourStep {
            section: "evening",
            repeats: 1,
        },
        TourStep {
            section: "night",
            repeats: 1,
        },
    ],
    loop_from: 1,
};

pub fn apply_automatic_arrangement(
    mut score: PortableScore,
    recipe: ArrangementRecipe,
    autoplay: bool,
) -> Result<PortableScore, String> {
    if !autoplay {
        return Ok(score);
    }
    if score.form.is_some() {
        return Err("automatic arrangement requires a score without an existing form".into());
    }

    let tour = match recipe {
        ArrangementRecipe::Racing => &RACING_TOUR,
        ArrangementRecipe::RacingExtended => &RACING_EXTENDED_TOUR,
        ArrangementRecipe::Adventure => &ADVENTURE_TOUR,
        ArrangementRecipe::Cozy => &COZY_TOUR,
    };
    for step in tour.steps {
        if score.section(step.section).is_none() {
            return Err(format!(
                "automatic {:?} arrangement requires section {}",
                recipe, step.section
            ));
        }
    }

    score.form = Some(SongForm {
        steps: tour
            .steps
            .iter()
            .map(|step| SongFormStep {
                section: step.section.into(),
                repeats: step.repeats,
            })
            .collect(),
        loop_from: Some(tour.loop_from),
        origin: Some(FormOrigin::TransitionStart),
    });
    score.id.push_str(&format!("-autoplay-v{AUTOPLAY_VERSION}"));
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use super::{apply_automatic_arrangement, ArrangementRecipe};
    use crate::{
        generate_adventure, generate_racing, generate_racing_arrangement, AdventureInput,
        AdventureStyle, GenerateInput, InstrumentPalette, RacingArrangement, Style,
    };

    fn racing_score() -> crate::PortableScore {
        generate_racing(&GenerateInput {
            secret: "arrangement-test".into(),
            seed: "tour".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.6,
            complexity: 0.5,
            brightness: 0.5,
            syncopation: 0.6,
        })
        .expect("racing arrangement fixture must validate")
    }

    fn adventure_score() -> crate::PortableScore {
        generate_adventure(&AdventureInput {
            secret: "arrangement-test".into(),
            seed: "tour".into(),
            style: AdventureStyle::Folk,
            wonder: 0.6,
            danger: 0.5,
            mystery: 0.6,
            motion: 0.58,
        })
        .expect("adventure arrangement fixture must validate")
    }

    #[test]
    fn disabled_adapter_preserves_the_racing_score_byte_for_byte() {
        let score = racing_score();
        let expected = serde_json::to_vec(&score).expect("fixture must serialize");
        let unchanged = apply_automatic_arrangement(score, ArrangementRecipe::Racing, false)
            .expect("disabled arrangement must succeed");

        assert!(unchanged.form.is_none());
        assert_eq!(
            serde_json::to_vec(&unchanged).expect("unchanged score must serialize"),
            expected
        );
    }

    #[test]
    fn racing_tour_uses_the_canonical_order_and_skips_the_garage_when_looping() {
        let original = racing_score();
        let original_id = original.id.clone();
        let score = apply_automatic_arrangement(original, ArrangementRecipe::Racing, true)
            .expect("racing tour must validate");
        let form = score.form.expect("autoplay score must have a form");

        assert_eq!(
            form.steps
                .iter()
                .map(|step| (step.section.as_str(), step.repeats))
                .collect::<Vec<_>>(),
            [
                ("garage", 1),
                ("grid", 1),
                ("cruise", 2),
                ("attack", 1),
                ("final-lap", 1),
                ("victory", 1),
            ]
        );
        assert_eq!(form.loop_from, Some(1));
        assert_eq!(form.origin, Some(crate::score::FormOrigin::TransitionStart));
        assert_eq!(score.id, format!("{original_id}-autoplay-v1"));
    }

    #[test]
    fn adventure_tour_references_every_quest_phase_and_validates_actual_lengths() {
        let score =
            apply_automatic_arrangement(adventure_score(), ArrangementRecipe::Adventure, true)
                .expect("adventure tour must validate");
        let form = score
            .form
            .as_ref()
            .expect("autoplay score must have a form");

        assert_eq!(
            form.steps
                .iter()
                .map(|step| step.section.as_str())
                .collect::<Vec<_>>(),
            [
                "camp",
                "explore",
                "town",
                "dungeon",
                "combat",
                "boss",
                "sanctuary",
                "victory",
            ]
        );
        assert_eq!(form.loop_from, Some(1));
        for step in &form.steps {
            assert!(score.section(&step.section).is_some());
        }
        score.validate().expect("arranged score must validate");
    }

    #[test]
    fn incomplete_scores_fail_instead_of_producing_a_partial_tour() {
        let mut score = racing_score();
        score.sections.retain(|section| section.id != "victory");
        let error = apply_automatic_arrangement(score, ArrangementRecipe::Racing, true)
            .expect_err("missing tour sections must fail");
        assert_eq!(
            error,
            "automatic Racing arrangement requires section victory"
        );
    }

    #[test]
    fn racing_extended_tour_plays_all_ten_sections_in_order_and_loops_to_grid() {
        let original = generate_racing_arrangement(
            &GenerateInput {
                secret: "arrangement-test".into(),
                seed: "tour".into(),
                style: Style::Funk,
                palette: InstrumentPalette::default(),
                energy: 0.6,
                complexity: 0.5,
                brightness: 0.5,
                syncopation: 0.6,
            },
            RacingArrangement::Extended,
        )
        .expect("extended racing arrangement fixture must validate");
        let original_id = original.id.clone();
        let score = apply_automatic_arrangement(original, ArrangementRecipe::RacingExtended, true)
            .expect("racing extended tour must validate");
        let form = score
            .form
            .as_ref()
            .expect("autoplay score must have a form");

        assert_eq!(
            form.steps
                .iter()
                .map(|step| (step.section.as_str(), step.repeats))
                .collect::<Vec<_>>(),
            [
                ("garage", 1),
                ("ignition", 1),
                ("grid", 1),
                ("cruise", 1),
                ("slipstream", 1),
                ("attack", 1),
                ("redline", 1),
                ("final-lap", 1),
                ("victory", 1),
                ("cooldown", 1),
            ]
        );
        assert_eq!(form.loop_from, Some(2));
        assert_eq!(form.origin, Some(crate::score::FormOrigin::TransitionStart));
        assert_eq!(score.id, format!("{original_id}-autoplay-v1"));
        score
            .validate()
            .expect("arranged extended score must validate");
    }

    #[test]
    fn cozy_tour_walks_the_day_and_loops_into_the_morning() {
        let score = crate::generate_cozy(&crate::CozyInput {
            secret: "arrangement-test".into(),
            seed: "tour".into(),
            style: crate::CozyStyle::Acoustic,
            warmth: 0.5,
            bustle: 0.5,
            jazz: 0.5,
            swing: 0.5,
        })
        .expect("cozy fixture must validate");
        let form = apply_automatic_arrangement(score, ArrangementRecipe::Cozy, true)
            .expect("cozy tour must validate")
            .form
            .expect("autoplay score must have a form");
        let steps: Vec<&str> = form
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert_eq!(
            steps,
            ["dawn", "morning", "noon", "market", "evening", "night"]
        );
        assert_eq!(form.loop_from, Some(1));
    }
}
