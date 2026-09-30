//! Cozy life-sim score: a day in the village, from first coffee to lanterns
//! out, for farming, café, crafting and other slow-living games.
//!
//! Eight sections follow the clock (dawn, morning, noon, evening, night) and
//! the place (market, festival), with rain as its own weather. One seeded
//! theme and key carry through the day over jazz-pop seventh-chord harmony,
//! in three ensembles: acoustic, lo-fi and bossa.

mod arrangement;
mod composition;
mod harmony;
mod pool;

pub use arrangement::{generate_cozy_arrangement, CozyArrangement};

use crate::rng::hash_text;
use crate::score::{
    AdaptiveCondition, AdaptiveRule, CozyState, PortableScore, SCORE_SCHEMA_VERSION,
};
use crate::theory::NOTE_NAMES;

pub const GENERATOR_VERSION: &str = "1.0.0";
pub const DNA_SEED_VERSION: &str = "1.0.0";

/// The first hour of each part of the day. Night wraps past midnight.
const DAWN_FROM: f64 = 5.0;
const MORNING_FROM: f64 = 8.0;
const NOON_FROM: f64 = 11.0;
const EVENING_FROM: f64 = 17.0;
const NIGHT_FROM: f64 = 21.0;
/// Rain at or above this strength sets the rain section.
const RAIN_FROM: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CozyStyle {
    Acoustic,
    Lofi,
    Bossa,
}

impl CozyStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "acoustic" => Ok(Self::Acoustic),
            "lofi" => Ok(Self::Lofi),
            "bossa" => Ok(Self::Bossa),
            other => Err(format!("Unknown cozy style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Acoustic => "acoustic",
            Self::Lofi => "lofi",
            Self::Bossa => "bossa",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Acoustic => "Acoustic",
            Self::Lofi => "Lo-fi",
            Self::Bossa => "Bossa",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CozyInput {
    pub secret: String,
    pub seed: String,
    pub style: CozyStyle,
    /// Brightness: register, sparkle and a softer, fuller comp.
    pub warmth: f64,
    /// Energy: tempo, drum ghosts and busier parts.
    pub bustle: f64,
    /// Complexity: ninths, ii–V substitutions and chromatic bass approaches.
    pub jazz: f64,
    /// Syncopation: swing and anticipated melody notes.
    pub swing: f64,
}

#[derive(Clone, Copy)]
struct NormalizedTraits {
    warmth: f64,
    bustle: f64,
    jazz: f64,
    swing: f64,
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn normalize(input: &CozyInput) -> NormalizedTraits {
    NormalizedTraits {
        warmth: clamp_unit(input.warmth),
        bustle: clamp_unit(input.bustle),
        jazz: clamp_unit(input.jazz),
        swing: clamp_unit(input.swing),
    }
}

fn subseed(secret: &str, seed: &str, domain: &str) -> u32 {
    let mut value = format!("{DNA_SEED_VERSION}\0string:{seed}\0cozy:{domain}");
    if !secret.is_empty() {
        value.push('\0');
        value.push_str(secret);
    }
    hash_text(&value)
}

fn score_id(secret: &str, seed: &str, style: CozyStyle, traits: NormalizedTraits) -> String {
    let identity = hash_text(&format!(
        "{secret}\0{seed}\0{}\0{:?}\0{GENERATOR_VERSION}",
        style.as_str(),
        [traits.warmth, traits.bustle, traits.jazz, traits.swing]
    ));
    format!(
        "cozy-generated-v{}-{identity:08x}",
        GENERATOR_VERSION.replace('.', "-")
    )
}

/// Serialized selection rules for the portable score, mirroring
/// [`select_cozy_section`] exactly.
pub fn default_rules() -> Vec<AdaptiveRule> {
    fn rule(
        target: &str,
        priority: i32,
        place: Option<&str>,
        numeric: &[(&str, Option<f64>, Option<f64>)],
    ) -> AdaptiveRule {
        let mut ranges = serde_json::Map::new();
        for (name, min, max) in numeric {
            let mut range = serde_json::Map::new();
            if let Some(min) = min {
                range.insert("min".into(), serde_json::json!(min));
            }
            if let Some(max) = max {
                range.insert("max".into(), serde_json::json!(max));
            }
            ranges.insert((*name).into(), serde_json::Value::Object(range));
        }
        let mut categorical = serde_json::Map::new();
        if let Some(place) = place {
            categorical.insert("place".into(), serde_json::json!(place));
        }
        AdaptiveRule {
            target: target.to_string(),
            priority,
            when: AdaptiveCondition {
                numeric: serde_json::Value::Object(ranges),
                categorical: serde_json::Value::Object(categorical),
            },
            hold: None,
        }
    }

    vec![
        rule("festival", 100, Some("festival"), &[]),
        rule("rain", 90, None, &[("rain", Some(RAIN_FROM), None)]),
        rule("night", 80, None, &[("hour", Some(NIGHT_FROM), None)]),
        rule("night", 80, None, &[("hour", None, Some(DAWN_FROM))]),
        rule(
            "dawn",
            70,
            None,
            &[("hour", Some(DAWN_FROM), Some(MORNING_FROM))],
        ),
        rule(
            "evening",
            70,
            None,
            &[("hour", Some(EVENING_FROM), Some(NIGHT_FROM))],
        ),
        rule("market", 65, Some("town"), &[]),
        rule(
            "morning",
            60,
            None,
            &[("hour", Some(MORNING_FROM), Some(NOON_FROM))],
        ),
        rule(
            "noon",
            50,
            None,
            &[("hour", Some(NOON_FROM), Some(EVENING_FROM))],
        ),
    ]
}

/// Pick the section for the village's state. Deterministic and total; ranges
/// include both ends and the higher priority wins a shared boundary, exactly
/// as the serialized rules resolve. Anything unmatched opens at dawn.
pub fn select_cozy_section(state: &CozyState) -> &'static str {
    let hour = state.hour;
    let within = |from: f64, to: f64| (from..=to).contains(&hour);
    if state.place == "festival" {
        "festival"
    } else if state.rain >= RAIN_FROM {
        "rain"
    } else if hour >= NIGHT_FROM || hour <= DAWN_FROM {
        "night"
    } else if within(DAWN_FROM, MORNING_FROM) {
        "dawn"
    } else if within(EVENING_FROM, NIGHT_FROM) {
        "evening"
    } else if state.place == "town" {
        "market"
    } else if within(MORNING_FROM, NOON_FROM) {
        "morning"
    } else if within(NOON_FROM, EVENING_FROM) {
        "noon"
    } else {
        "dawn"
    }
}

fn compose_cozy(input: &CozyInput) -> PortableScore {
    let traits = normalize(input);
    let dna = composition::PieceDna::new(subseed(&input.secret, &input.seed, "piece"));
    let ticks_per_beat = 960;
    PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: score_id(&input.secret, &input.seed, input.style, traits),
        title: format!(
            "{} {} Days",
            input.style.display(),
            NOTE_NAMES[dna.tonic_pitch_class as usize].to_uppercase()
        ),
        bpm: composition::tempo(input.style, traits),
        beats_per_bar: 4,
        ticks_per_beat,
        crossfade_bars: 2.0,
        default_section: "dawn".to_string(),
        sections: composition::build_sections(
            input.style,
            traits,
            &dna,
            ticks_per_beat,
            |section| subseed(&input.secret, &input.seed, section),
        ),
        rules: default_rules(),
        form: None,
    }
}

pub fn generate_cozy(input: &CozyInput) -> Result<PortableScore, String> {
    let score = compose_cozy(input);
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::score::MusicEvent;

    const STYLES: [CozyStyle; 3] = [CozyStyle::Acoustic, CozyStyle::Lofi, CozyStyle::Bossa];

    fn sample(seed: &str, style: CozyStyle) -> CozyInput {
        CozyInput {
            secret: "cozy-test".into(),
            seed: seed.into(),
            style,
            warmth: 0.6,
            bustle: 0.5,
            jazz: 0.5,
            swing: 0.5,
        }
    }

    fn state(hour: f64, place: &str, rain: f64) -> CozyState {
        CozyState {
            hour,
            place: place.into(),
            rain,
        }
    }

    #[test]
    fn style_contract_is_exact() {
        for style in STYLES {
            assert_eq!(CozyStyle::parse(style.as_str()), Ok(style));
        }
        for rejected in ["Lofi", "lo-fi", "jazz", ""] {
            assert!(CozyStyle::parse(rejected).is_err());
        }
    }

    #[test]
    fn a_day_follows_the_clock() {
        for (hour, expected) in [
            (0.0, "night"),
            (4.9, "night"),
            (5.0, "night"),
            (5.1, "dawn"),
            (8.0, "dawn"),
            (9.5, "morning"),
            (11.0, "morning"),
            (13.0, "noon"),
            (17.0, "evening"),
            (20.9, "evening"),
            (21.0, "night"),
            (23.9, "night"),
        ] {
            assert_eq!(
                select_cozy_section(&state(hour, "home", 0.0)),
                expected,
                "{hour}"
            );
        }
        assert_eq!(select_cozy_section(&state(10.0, "town", 0.0)), "market");
        assert_eq!(select_cozy_section(&state(18.0, "town", 0.0)), "evening");
        assert_eq!(select_cozy_section(&state(22.0, "town", 0.0)), "night");
        assert_eq!(select_cozy_section(&state(13.0, "fields", 0.7)), "rain");
        assert_eq!(
            select_cozy_section(&state(22.0, "festival", 0.9)),
            "festival"
        );
    }

    #[test]
    fn selector_and_serialized_rules_agree_everywhere() {
        let rules = default_rules();
        for tenth in 0..=240 {
            let hour = f64::from(tenth) / 10.0;
            for place in ["home", "fields", "town", "festival", "river"] {
                for rain in [0.0, 0.49, 0.5, 1.0] {
                    let game = state(hour, place, rain);
                    let ruled = rules
                        .iter()
                        .filter(|rule| {
                            let place_ok = rule
                                .when
                                .categorical
                                .get("place")
                                .is_none_or(|expected| expected == place);
                            let numeric_ok = rule.when.numeric.as_object().is_none_or(|ranges| {
                                ranges.iter().all(|(name, range)| {
                                    let value = if name == "hour" { hour } else { rain };
                                    range
                                        .get("min")
                                        .and_then(serde_json::Value::as_f64)
                                        .is_none_or(|min| value >= min)
                                        && range
                                            .get("max")
                                            .and_then(serde_json::Value::as_f64)
                                            .is_none_or(|max| value <= max)
                                })
                            });
                            place_ok && numeric_ok
                        })
                        .max_by_key(|rule| rule.priority)
                        .map_or("dawn", |rule| rule.target.as_str());
                    assert_eq!(select_cozy_section(&game), ruled, "{hour} {place} {rain}");
                }
            }
        }
    }

    #[test]
    fn generates_eight_sections_that_validate_for_every_style() {
        for style in STYLES {
            let score = generate_cozy(&sample("day-1", style)).unwrap();
            assert_eq!(score.default_section, "dawn");
            assert!(score.form.is_none());
            let ids: Vec<&str> = score.sections.iter().map(|s| s.id.as_str()).collect();
            assert_eq!(
                ids,
                ["dawn", "morning", "market", "noon", "rain", "evening", "festival", "night"]
            );
            let mut seen = HashSet::new();
            for section in &score.sections {
                assert!(
                    !section.events.is_empty(),
                    "{style:?} {} is empty",
                    section.id
                );
                for event in &section.events {
                    let id = match event {
                        MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
                    };
                    assert!(seen.insert(id.clone()), "duplicate id {id}");
                    assert!(event.start_tick() + event.duration_ticks() <= section.length_ticks);
                    if let Some(pitch) = event.pitch() {
                        assert!((28..=90).contains(&pitch), "{} pitch {pitch}", section.id);
                    }
                }
            }
        }
    }

    #[test]
    fn deterministic_and_seeded() {
        let first = generate_cozy(&sample("day-1", CozyStyle::Lofi)).unwrap();
        let again = generate_cozy(&sample("day-1", CozyStyle::Lofi)).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&again).unwrap()
        );
        let other = generate_cozy(&sample("day-2", CozyStyle::Lofi)).unwrap();
        assert_ne!(first.id, other.id);
        assert_ne!(
            serde_json::to_vec(&first.sections).unwrap(),
            serde_json::to_vec(&other.sections).unwrap()
        );
    }

    #[test]
    fn quiet_scenes_are_quieter_than_the_festival() {
        for style in STYLES {
            let score = generate_cozy(&sample("density", style)).unwrap();
            let per_bar = |id: &str| {
                let section = score.section(id).unwrap();
                section.events.len() as f64 / f64::from(section.length_ticks / score.bar_ticks())
            };
            for quiet in ["dawn", "night", "rain"] {
                assert!(per_bar(quiet) < per_bar("festival"), "{style:?} {quiet}");
            }
            let drums = |id: &str| {
                score
                    .section(id)
                    .unwrap()
                    .events
                    .iter()
                    .filter(|e| matches!(e, MusicEvent::Percussion { .. }))
                    .count()
            };
            assert_eq!(drums("dawn"), 0);
            assert_eq!(drums("night"), 0);
        }
    }

    #[test]
    fn the_melody_strong_beats_sit_on_the_harmony() {
        for style in STYLES {
            let score = generate_cozy(&sample("harmony", style)).unwrap();
            let beat = score.ticks_per_beat;
            let (mut strong, mut clean) = (0, 0);
            for section in &score.sections {
                let accompaniment: Vec<&MusicEvent> = section
                    .events
                    .iter()
                    .filter(|e| matches!(e, MusicEvent::Note { lane, .. } if lane == "comp" || lane == "pad"))
                    .collect();
                for note in section.events.iter().filter(|e| e.is_melody()) {
                    if note.start_tick() % (2 * beat) != 0 {
                        continue;
                    }
                    let pitch = i32::from(note.pitch().unwrap());
                    let rub = accompaniment.iter().any(|a| {
                        a.start_tick() <= note.start_tick()
                            && note.start_tick() < a.start_tick() + a.duration_ticks()
                            && matches!(
                                (pitch - i32::from(a.pitch().unwrap())).rem_euclid(12),
                                1 | 11
                            )
                    });
                    strong += 1;
                    clean += usize::from(!rub);
                }
            }
            assert!(
                clean * 100 / strong.max(1) >= 97,
                "{style:?}: {clean}/{strong}"
            );
        }
    }

    #[test]
    fn traits_move_the_music_monotonically() {
        let count = |set: fn(&mut CozyInput, f64), filter: fn(&MusicEvent) -> bool| {
            [0.1, 0.5, 0.9].map(|value| {
                let mut input = sample("traits", CozyStyle::Lofi);
                set(&mut input, value);
                generate_cozy(&input)
                    .unwrap()
                    .sections
                    .iter()
                    .flat_map(|s| s.events.iter())
                    .filter(|e| filter(e))
                    .count()
            })
        };
        let drums = count(
            |i, v| i.bustle = v,
            |e| matches!(e, MusicEvent::Percussion { .. }),
        );
        assert!(
            drums[0] < drums[1] && drums[1] < drums[2],
            "bustle drums {drums:?}"
        );
        let bells = count(|i, v| i.warmth = v, |e| e.voice() == "bell");
        assert!(
            bells[0] <= bells[1] && bells[1] <= bells[2] && bells[0] < bells[2],
            "warmth {bells:?}"
        );
        let walks = count(
            |i, v| i.jazz = v,
            |e| matches!(e, MusicEvent::Note { lane, .. } if lane == "bass"),
        );
        assert!(
            walks[0] <= walks[1] && walks[1] <= walks[2] && walks[0] < walks[2],
            "jazz {walks:?}"
        );
    }
}
