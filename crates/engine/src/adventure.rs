//! Medieval-inspired adaptive adventure score.
//!
//! Eight long-form sections share one seeded modal identity while changing
//! phrase, orchestration, and pulse to follow a complete fantasy quest arc.

mod arrangement;
mod composition;
mod harmony;
mod pool;
mod theme;

pub use arrangement::{generate_adventure_arrangement, AdventureArrangement};

use crate::rng::hash_text;
use crate::score::{
    AdaptiveCondition, AdaptiveRule, AdventureState, PortableScore, SCORE_SCHEMA_VERSION,
};
use crate::theory::NOTE_NAMES;

pub const GENERATOR_VERSION: &str = "6.0.0";
pub const DNA_SEED_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdventureStyle {
    Folk,
    Dark,
    Orchestral,
}

impl AdventureStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "folk" => Ok(Self::Folk),
            "dark" => Ok(Self::Dark),
            "orchestral" => Ok(Self::Orchestral),
            other => Err(format!("Unknown adventure style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Folk => "folk",
            Self::Dark => "dark",
            Self::Orchestral => "orchestral",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Folk => "Folk",
            Self::Dark => "Dark",
            Self::Orchestral => "Orchestral",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdventureInput {
    pub secret: String,
    pub seed: String,
    pub style: AdventureStyle,
    pub wonder: f64,
    pub danger: f64,
    pub mystery: f64,
    pub motion: f64,
}

#[derive(Clone, Copy)]
struct NormalizedTraits {
    wonder: f64,
    danger: f64,
    mystery: f64,
    motion: f64,
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn normalize(input: &AdventureInput) -> NormalizedTraits {
    NormalizedTraits {
        wonder: clamp_unit(input.wonder),
        danger: clamp_unit(input.danger),
        mystery: clamp_unit(input.mystery),
        motion: clamp_unit(input.motion),
    }
}

pub(crate) fn subseed(secret: &str, seed: &str, domain: &str) -> u32 {
    let canonical = format!("string:{seed}");
    let mut value = format!("{DNA_SEED_VERSION}\0{canonical}\0{domain}");
    if !secret.is_empty() {
        value.push('\0');
        value.push_str(secret);
    }
    hash_text(&value)
}

/// The tonic pitch class for an Adventure piece, derived the same way the
/// generator does so the composed passes can anchor to Adventure's own key.
pub(crate) fn adventure_tonic_pitch_class(secret: &str, seed: &str) -> i32 {
    let piece_seed = subseed(secret, seed, "piece");
    composition::PieceDna::new(piece_seed).tonic_pitch_class
}

fn score_id(secret: &str, seed: &str, style: AdventureStyle, traits: NormalizedTraits) -> String {
    let trait_json = format!(
        r#"{{"wonder":{},"danger":{},"mystery":{},"motion":{}}}"#,
        traits.wonder, traits.danger, traits.mystery, traits.motion
    );
    let identity = hash_text(&format!(
        "{secret}\0{seed}\0{}\0{trait_json}\0{GENERATOR_VERSION}",
        style.as_str()
    ));
    format!(
        "adventure-generated-v{}-{identity:08x}",
        GENERATOR_VERSION.replace('.', "-")
    )
}

/// Serialized selection rules for the portable score, mirroring
/// [`select_adventure_section`] exactly.
pub fn default_rules() -> Vec<AdaptiveRule> {
    fn rule(
        target: &str,
        priority: i32,
        phase: Option<&str>,
        numeric: &[(&str, f64)],
    ) -> AdaptiveRule {
        let mut numeric_map = serde_json::Map::new();
        for (name, min) in numeric {
            numeric_map.insert((*name).into(), serde_json::json!({ "min": min }));
        }
        let mut categorical = serde_json::Map::new();
        if let Some(phase) = phase {
            categorical.insert("areaPhase".into(), serde_json::json!(phase));
        }
        AdaptiveRule {
            target: target.to_string(),
            priority,
            when: AdaptiveCondition {
                numeric: serde_json::Value::Object(numeric_map),
                categorical: serde_json::Value::Object(categorical),
            },
            hold: None,
        }
    }

    vec![
        rule("victory", 100, Some("victory"), &[]),
        rule("victory", 98, None, &[("questComplete", 1.0)]),
        rule("boss", 95, Some("boss"), &[]),
        rule("boss", 94, Some("combat"), &[("threat", 0.85)]),
        rule("combat", 85, Some("combat"), &[]),
        rule("combat", 80, None, &[("threat", 0.7)]),
        rule("sanctuary", 76, Some("sanctuary"), &[]),
        rule("sanctuary", 75, None, &[("discovery", 0.85)]),
        rule("dungeon", 70, Some("dungeon"), &[]),
        rule("town", 60, Some("town"), &[]),
        rule("explore", 50, Some("explore"), &[]),
        rule("explore", 45, None, &[("discovery", 0.3)]),
        rule("camp", 10, Some("camp"), &[]),
    ]
}

/// Pick the section for an adventure state. Deterministic and total; the
/// default is `camp`.
pub fn select_adventure_section(state: &AdventureState) -> &'static str {
    if state.quest_complete || state.area_phase == "victory" {
        "victory"
    } else if state.area_phase == "boss" || (state.area_phase == "combat" && state.threat >= 0.85) {
        "boss"
    } else if state.area_phase == "combat" || state.threat >= 0.7 {
        "combat"
    } else if state.discovery >= 0.85 || state.area_phase == "sanctuary" {
        "sanctuary"
    } else if state.area_phase == "dungeon" {
        "dungeon"
    } else if state.area_phase == "town" {
        "town"
    } else if state.area_phase == "explore" || state.discovery >= 0.3 {
        "explore"
    } else {
        "camp"
    }
}

fn compose_adventure(input: &AdventureInput) -> PortableScore {
    let traits = normalize(input);
    let piece_seed = subseed(&input.secret, &input.seed, "piece");
    let dna = composition::PieceDna::new(piece_seed);
    let ticks_per_beat = 960;
    let beats_per_bar = 4;

    PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: score_id(&input.secret, &input.seed, input.style, traits),
        title: format!(
            "{} {} Quest",
            input.style.display(),
            NOTE_NAMES[dna.tonic_pitch_class as usize].to_uppercase()
        ),
        bpm: composition::tempo(input.style, traits),
        beats_per_bar,
        ticks_per_beat,
        crossfade_bars: 2.0,
        default_section: "camp".to_string(),
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

pub fn generate_adventure(input: &AdventureInput) -> Result<PortableScore, String> {
    let score = compose_adventure(input);
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::composition::{mode_for, phrase_kind, PhraseKind, PieceDna, Scene, SECTION_PLANS};
    use super::harmony::dominant_degree;
    use super::{
        generate_adventure, select_adventure_section, subseed, AdventureInput, AdventureStyle,
        GENERATOR_VERSION,
    };
    use crate::score::{AdventureState, MusicEvent, PortableSection};
    use crate::theory::mode_intervals;

    fn sample(secret: &str, style: AdventureStyle) -> AdventureInput {
        AdventureInput {
            secret: secret.into(),
            seed: "trail-01".into(),
            style,
            wonder: 0.6,
            danger: 0.5,
            mystery: 0.6,
            motion: 0.58,
        }
    }

    fn event_id(event: &MusicEvent) -> &str {
        match event {
            MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
        }
    }

    fn musical_signature(section: &PortableSection, offset: u32, span: u32) -> Vec<String> {
        section
            .events
            .iter()
            .filter(|event| (offset..offset + span).contains(&event.start_tick()))
            .map(|event| match event {
                MusicEvent::Note {
                    lane,
                    start_tick,
                    duration_ticks,
                    pitch,
                    voice,
                    ..
                } => format!(
                    "n:{lane}:{}:{duration_ticks}:{pitch}:{voice}",
                    start_tick - offset
                ),
                MusicEvent::Percussion {
                    lane,
                    start_tick,
                    duration_ticks,
                    voice,
                    ..
                } => format!("p:{lane}:{}:{duration_ticks}:{voice}", start_tick - offset),
            })
            .collect()
    }

    #[test]
    fn style_contract_is_exact() {
        for (name, style) in [
            ("folk", AdventureStyle::Folk),
            ("dark", AdventureStyle::Dark),
            ("orchestral", AdventureStyle::Orchestral),
        ] {
            assert_eq!(AdventureStyle::parse(name), Ok(style));
            assert_eq!(style.as_str(), name);
        }
        for rejected in ["campfire", "court", "chapel", "wilds", "Folk"] {
            assert!(AdventureStyle::parse(rejected).is_err());
        }
    }

    #[test]
    fn generates_the_fourteen_requested_long_sections_without_a_default_form() {
        let score = generate_adventure(&sample("lengths", AdventureStyle::Folk)).unwrap();
        let bar_ticks = score.bar_ticks();
        assert_eq!(score.default_section, "camp");
        assert!(score.form.is_none());
        let expected = [
            ("camp", 16),
            ("explore", 32),
            ("town", 32),
            ("dungeon", 16),
            ("combat", 32),
            ("boss", 16),
            ("sanctuary", 16),
            ("victory", 32),
            ("skirmish", 16),
            ("assault", 16),
            ("chase", 32),
            ("festival", 32),
            ("reunion", 32),
            ("dawn", 16),
        ];
        assert_eq!(score.sections.len(), expected.len());
        for (id, bars) in expected {
            let section = score.section(id).expect("section must exist");
            assert_eq!(section.length_ticks, bars * bar_ticks, "{id}");
            assert!(!section.events.is_empty(), "{id} must contain music");
        }
    }

    #[test]
    fn generation_is_deterministic_and_seeds_change_musical_content() {
        let input = sample("deterministic", AdventureStyle::Folk);
        let first = generate_adventure(&input).unwrap();
        let second = generate_adventure(&input).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );

        let mut changed = input;
        changed.seed = "trail-02".into();
        let changed = generate_adventure(&changed).unwrap();
        assert_ne!(first.id, changed.id);
        assert_ne!(
            musical_signature(
                first.section("explore").unwrap(),
                0,
                first.section("explore").unwrap().length_ticks
            ),
            musical_signature(
                changed.section("explore").unwrap(),
                0,
                changed.section("explore").unwrap().length_ticks
            ),
            "a seed must alter notes or rhythms, not only the score id"
        );
    }

    #[test]
    fn traits_are_normalized_before_identity_and_composition() {
        let mut unusual = sample("normalize", AdventureStyle::Dark);
        unusual.wonder = f64::NAN;
        unusual.danger = -4.0;
        unusual.mystery = f64::INFINITY;
        unusual.motion = 3.0;
        let mut expected = unusual.clone();
        expected.wonder = 0.5;
        expected.danger = 0.0;
        expected.mystery = 0.5;
        expected.motion = 1.0;
        assert_eq!(
            serde_json::to_vec(&generate_adventure(&unusual).unwrap()).unwrap(),
            serde_json::to_vec(&generate_adventure(&expected).unwrap()).unwrap()
        );
    }

    #[test]
    fn fine_trait_differences_affect_score_identity() {
        // json_num used to round normalized traits to two decimals, so tuples
        // that differed only past the hundredths (motion 0.449 vs 0.451, both
        // serialized as "0.45") shared one id even though composition reads the
        // exact value. The identity now round-trips the full f64, so distinct
        // normalized tuples can no longer collide.
        let input = |motion| AdventureInput {
            secret: "identity".into(),
            seed: "trail-01".into(),
            style: AdventureStyle::Folk,
            wonder: 0.6,
            danger: 0.5,
            mystery: 0.6,
            motion,
        };
        let low = generate_adventure(&input(0.449)).unwrap();
        let high = generate_adventure(&input(0.451)).unwrap();

        assert_ne!(low.id, high.id, "0.449 and 0.451 must not share an id");

        let low_explore = low.section("explore").unwrap();
        let high_explore = high.section("explore").unwrap();
        assert_ne!(
            musical_signature(low_explore, 0, low_explore.length_ticks),
            musical_signature(high_explore, 0, high_explore.length_ticks),
            "motion crossing the 0.45 percussion threshold must alter the music, not only the id"
        );
    }

    #[test]
    fn events_use_acoustic_palette_safe_registers_and_unique_ids() {
        let allowed_notes = HashSet::from(["harp", "recorder", "vielle", "bell", "horn", "timpani"]);
        let allowed_percussion = HashSet::from(["frame-drum", "tambourine"]);
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let score = generate_adventure(&sample("bounds", style)).unwrap();
            let mut ids = HashSet::new();
            for section in &score.sections {
                for event in &section.events {
                    assert!(ids.insert(event_id(event)), "duplicate event id");
                    assert!(event.velocity().is_finite());
                    assert!((0.0..=1.0).contains(&event.velocity()));
                    assert!(
                        event.start_tick() + event.duration_ticks() <= section.length_ticks,
                        "{} event crosses section boundary",
                        section.id
                    );
                    match event {
                        MusicEvent::Note {
                            pitch, voice, role, ..
                        } => {
                            assert!(allowed_notes.contains(voice.as_str()), "{voice}");
                            assert!((36..=84).contains(pitch), "unsafe pitch {pitch}");
                            if role.as_deref() == Some("melody") {
                                assert!((55..=84).contains(pitch), "melody pitch {pitch}");
                            }
                        }
                        MusicEvent::Percussion { voice, .. } => {
                            assert!(allowed_percussion.contains(voice.as_str()), "{voice}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_layer_stays_in_the_section_mode_and_phrases_reach_home() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("harmony", style);
            let dna = PieceDna::new(subseed(&input.secret, &input.seed, "piece"));
            let tonic = dna.tonic_pitch_class;
            let score = generate_adventure(&input).unwrap();
            let bar_ticks = score.bar_ticks();
            for plan in SECTION_PLANS {
                let section = score.section(plan.id).unwrap();
                let pitch_classes: HashSet<i32> = mode_intervals(mode_for(style, plan.scene))
                    .into_iter()
                    .map(|interval| (tonic + interval).rem_euclid(12))
                    .collect();
                for pitch in section.events.iter().filter_map(MusicEvent::pitch) {
                    assert!(
                        pitch_classes.contains(&(i32::from(pitch) % 12)),
                        "{} pitch {pitch} left {}",
                        plan.id,
                        mode_for(style, plan.scene)
                    );
                }

                let phrase_count = plan.bars / 4;
                for phrase in 0..phrase_count {
                    let arrival_tick = (phrase * 4 + 3) * bar_ticks;
                    let arrival = section
                        .events
                        .iter()
                        .find(|event| event.is_melody() && event.start_tick() == arrival_tick)
                        .and_then(MusicEvent::pitch)
                        .unwrap_or_else(|| panic!("{} phrase {phrase} has no arrival", plan.id));
                    let expected_degree = match phrase_kind(phrase, phrase_count) {
                        PhraseKind::Antecedent | PhraseKind::Development => {
                            dominant_degree(mode_for(style, plan.scene))
                        }
                        PhraseKind::Consequent | PhraseKind::Return | PhraseKind::Cadence => 0,
                    };
                    let interval =
                        mode_intervals(mode_for(style, plan.scene))[expected_degree as usize];
                    assert_eq!(
                        i32::from(arrival) % 12,
                        (tonic + interval).rem_euclid(12),
                        "{} phrase {phrase} has no directed arrival",
                        plan.id
                    );
                }

                let final_bass = section
                    .events
                    .iter()
                    .filter(
                        |event| matches!(event, MusicEvent::Note { lane, .. } if lane == "bass"),
                    )
                    .max_by_key(|event| event.start_tick())
                    .and_then(MusicEvent::pitch)
                    .unwrap();
                assert_eq!(i32::from(final_bass) % 12, tonic);
            }
        }
    }

    #[test]
    fn styles_change_texture_and_rhythm_at_identical_traits() {
        let scores = [
            generate_adventure(&sample("styles", AdventureStyle::Folk)).unwrap(),
            generate_adventure(&sample("styles", AdventureStyle::Dark)).unwrap(),
            generate_adventure(&sample("styles", AdventureStyle::Orchestral)).unwrap(),
        ];
        for left in 0..scores.len() {
            for right in left + 1..scores.len() {
                let left_town = scores[left].section("town").unwrap();
                let right_town = scores[right].section("town").unwrap();
                let left_rhythm: Vec<_> = left_town
                    .events
                    .iter()
                    .map(|event| (event.voice(), event.start_tick(), event.duration_ticks()))
                    .collect();
                let right_rhythm: Vec<_> = right_town
                    .events
                    .iter()
                    .map(|event| (event.voice(), event.start_tick(), event.duration_ticks()))
                    .collect();
                assert_ne!(left_rhythm, right_rhythm, "style rhythm must be authored");

                let lane_counts = |section: &PortableSection| {
                    let mut counts = HashMap::new();
                    for event in &section.events {
                        let lane = match event {
                            MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. } => {
                                lane.as_str()
                            }
                        };
                        *counts.entry(lane.to_string()).or_insert(0usize) += 1;
                    }
                    counts
                };
                assert_ne!(
                    lane_counts(left_town),
                    lane_counts(right_town),
                    "style textures must use different layer activity"
                );
            }
        }
    }

    #[test]
    fn section_development_is_not_repeated_padding() {
        let score = generate_adventure(&sample("develop", AdventureStyle::Orchestral)).unwrap();
        for section in &score.sections {
            let half = section.length_ticks / 2;
            assert_ne!(
                musical_signature(section, 0, half),
                musical_signature(section, half, half),
                "{} halves repeat after normalizing ids and time offsets",
                section.id
            );
        }
    }

    #[test]
    fn cadences_breathe_and_resolve_to_the_tonic() {
        let input = sample("cadence", AdventureStyle::Folk);
        let dna = PieceDna::new(subseed(&input.secret, &input.seed, "piece"));
        let score = generate_adventure(&input).unwrap();
        let bar_ticks = score.bar_ticks();
        for section in &score.sections {
            let final_bar = section.length_ticks - bar_ticks;
            let previous_bar = final_bar - bar_ticks;
            let final_melody: Vec<_> = section
                .events
                .iter()
                .filter(|event| event.is_melody() && event.start_tick() >= final_bar)
                .collect();
            let previous_melody_count = section
                .events
                .iter()
                .filter(|event| {
                    event.is_melody() && (previous_bar..final_bar).contains(&event.start_tick())
                })
                .count();
            assert_eq!(final_melody.len(), 1, "{} final cadence", section.id);
            assert!(previous_melody_count > final_melody.len());
            assert_eq!(
                i32::from(final_melody[0].pitch().unwrap()) % 12,
                dna.tonic_pitch_class
            );
            assert!(
                final_melody[0].start_tick() + final_melody[0].duration_ticks()
                    < section.length_ticks,
                "{} must leave a breath after its final arrival",
                section.id
            );
        }
    }

    #[test]
    fn many_seeds_are_valid_and_varied() {
        assert_eq!(GENERATOR_VERSION, "6.0.0");
        let styles = [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ];
        let mut ids = HashSet::new();
        let mut openings = HashSet::new();
        for index in 0..48 {
            let input = AdventureInput {
                secret: "stress".into(),
                seed: format!("trail-{index}"),
                style: styles[index % styles.len()],
                wonder: f64::from((index % 17) as u32) / 16.0,
                danger: f64::from((index % 11) as u32) / 10.0,
                mystery: f64::from((index % 13) as u32) / 12.0,
                motion: f64::from((index % 19) as u32) / 18.0,
            };
            let score = generate_adventure(&input).expect("stress score must validate");
            assert!(ids.insert(score.id.clone()));
            let opening: Vec<_> = score
                .section("explore")
                .unwrap()
                .events
                .iter()
                .filter_map(MusicEvent::pitch)
                .take(24)
                .collect();
            openings.insert(opening);
        }
        assert_eq!(ids.len(), 48);
        assert!(
            openings.len() > 24,
            "seed variety must be audible in pitches"
        );
    }

    #[test]
    fn selector_and_serialized_rules_agree_across_states() {
        use super::default_rules;
        let phases = [
            "camp",
            "explore",
            "town",
            "dungeon",
            "combat",
            "boss",
            "sanctuary",
            "victory",
            "unknown",
        ];
        let readings = [
            (0.0, 0.0),
            (0.4, 0.2),
            (0.9, 0.2),
            (0.2, 0.8),
            (0.5, 0.9),
            (0.1, 0.95),
        ];
        for phase in phases {
            for (discovery, threat) in readings {
                for quest_complete in [false, true] {
                    let state = AdventureState {
                        area_phase: phase.into(),
                        discovery,
                        threat,
                        quest_complete,
                    };
                    let selected = select_adventure_section(&state);
                    let ruled = default_rules()
                        .into_iter()
                        .find(|rule| {
                            let phase_ok = rule
                                .when
                                .categorical
                                .get("areaPhase")
                                .is_none_or(|expected| expected == &state.area_phase);
                            let numeric_ok =
                                rule.when.numeric.as_object().is_none_or(|conditions| {
                                    conditions.iter().all(|(name, spec)| {
                                        let Some(min) =
                                            spec.get("min").and_then(serde_json::Value::as_f64)
                                        else {
                                            return true;
                                        };
                                        match name.as_str() {
                                            "discovery" => discovery >= min,
                                            "threat" => threat >= min,
                                            "questComplete" => {
                                                (if state.quest_complete { 1.0 } else { 0.0 })
                                                    >= min
                                            }
                                            _ => true,
                                        }
                                    })
                                });
                            phase_ok && numeric_ok
                        })
                        .map(|rule| rule.target)
                        .unwrap_or_else(|| "camp".to_string());
                    assert_eq!(selected, ruled);
                }
            }
        }
    }

    #[test]
    fn sanctuary_discovery_and_quest_priority_are_preserved() {
        let state = |phase: &str, discovery: f64, threat: f64, quest: bool| AdventureState {
            area_phase: phase.into(),
            discovery,
            threat,
            quest_complete: quest,
        };
        assert_eq!(
            select_adventure_section(&state("explore", 0.9, 0.0, false)),
            "sanctuary"
        );
        assert_eq!(
            select_adventure_section(&state("sanctuary", 0.0, 0.0, false)),
            "sanctuary"
        );
        assert_eq!(
            select_adventure_section(&state("combat", 0.0, 0.9, false)),
            "boss"
        );
        assert_eq!(
            select_adventure_section(&state("boss", 0.0, 0.0, true)),
            "victory"
        );
    }

    /// Chord quality of a mode's triad on a scale degree, derived from the
    /// actual interval sizes (major = 4+7, minor = 3+7, diminished = 3+6).
    fn triad_quality(mode: &str, degree: i32) -> &'static str {
        let intervals = mode_intervals(mode);
        let root = intervals[degree.rem_euclid(7) as usize];
        let third = intervals[(degree + 2).rem_euclid(7) as usize];
        let fifth = intervals[(degree + 4).rem_euclid(7) as usize];
        match ((third - root).rem_euclid(12), (fifth - root).rem_euclid(12)) {
            (4, 7) => "major",
            (3, 7) => "minor",
            (3, 6) => "diminished",
            _ => "other",
        }
    }

    #[test]
    fn safe_and_dangerous_tonics_keep_contrast() {
        for style in [AdventureStyle::Folk, AdventureStyle::Orchestral] {
            for scene in [
                Scene::Camp,
                Scene::Explore,
                Scene::Town,
                Scene::Sanctuary,
                Scene::Victory,
                Scene::Festival,
                Scene::Reunion,
                Scene::Dawn,
            ] {
                assert_eq!(
                    triad_quality(mode_for(style, scene), 0),
                    "major",
                    "{style:?} {scene:?} safe phase tonic must be major"
                );
            }
            for scene in [
                Scene::Dungeon,
                Scene::Combat,
                Scene::Boss,
                Scene::Skirmish,
                Scene::Assault,
                Scene::Chase,
            ] {
                assert_eq!(
                    triad_quality(mode_for(style, scene), 0),
                    "minor",
                    "{style:?} {scene:?} dangerous phase tonic must be minor"
                );
            }
        }
        // Dark stays bittersweet everywhere except the earned release of victory
        // and the first light of dawn.
        for scene in [
            Scene::Camp,
            Scene::Explore,
            Scene::Town,
            Scene::Dungeon,
            Scene::Combat,
            Scene::Boss,
            Scene::Sanctuary,
            Scene::Skirmish,
            Scene::Assault,
            Scene::Chase,
            Scene::Festival,
            Scene::Reunion,
        ] {
            assert_eq!(
                triad_quality(mode_for(AdventureStyle::Dark, scene), 0),
                "minor",
                "Dark {scene:?} must stay minor"
            );
        }
        assert_eq!(
            triad_quality(mode_for(AdventureStyle::Dark, Scene::Victory), 0),
            "major"
        );
        assert_eq!(
            triad_quality(mode_for(AdventureStyle::Dark, Scene::Dawn), 0),
            "major"
        );
    }

    #[test]
    fn folk_and_orchestral_foregrounds_use_distinct_instruments() {
        let folk = generate_adventure(&sample("fgorch", AdventureStyle::Folk)).unwrap();
        let orch = generate_adventure(&sample("fgorch", AdventureStyle::Orchestral)).unwrap();
        for id in ["camp", "explore", "sanctuary", "victory"] {
            let f = folk.section(id).unwrap();
            let o = orch.section(id).unwrap();

            assert!(
                f.events
                    .iter()
                    .filter(|event| event.is_melody())
                    .all(|event| event.voice() == "recorder"),
                "{id}: folk melody must be recorder-led"
            );
            assert!(
                !f.events.iter().any(|event| event.voice() == "vielle"),
                "{id}: folk must not bow a held vielle pad"
            );

            assert!(
                o.events
                    .iter()
                    .filter(|event| event.is_melody())
                    .all(|event| event.voice() == "vielle"),
                "{id}: orchestral melody must be vielle-led"
            );
            assert!(
                o.events
                    .iter()
                    .any(|event| event.voice() == "vielle" && !event.is_melody()),
                "{id}: orchestral needs a voiced string foundation"
            );
            assert!(
                o.events
                    .iter()
                    .any(|event| event.voice() == "recorder" && !event.is_melody()),
                "{id}: orchestral recorder should answer, not lead"
            );
        }
    }

    #[test]
    fn camp_introduces_folk_and_orchestral_identity_in_the_first_bar() {
        let folk = generate_adventure(&sample("opening", AdventureStyle::Folk)).unwrap();
        let orchestral =
            generate_adventure(&sample("opening", AdventureStyle::Orchestral)).unwrap();
        let folk_opening: Vec<_> = folk
            .section("camp")
            .unwrap()
            .events
            .iter()
            .filter(|event| event.start_tick() < folk.bar_ticks())
            .collect();
        let orchestral_opening: Vec<_> = orchestral
            .section("camp")
            .unwrap()
            .events
            .iter()
            .filter(|event| event.start_tick() < orchestral.bar_ticks())
            .collect();
        assert!(folk_opening.iter().any(|event| event.voice() == "harp"));
        assert!(!folk_opening.iter().any(|event| event.voice() == "vielle"));
        assert!(orchestral_opening
            .iter()
            .any(|event| event.is_melody() && event.voice() == "vielle"));
        assert!(orchestral_opening
            .iter()
            .any(|event| !event.is_melody() && event.voice() == "vielle"));
    }

    #[test]
    fn orchestral_lead_gates_are_broader_than_folk_lilt() {
        let folk = generate_adventure(&sample("gates", AdventureStyle::Folk)).unwrap();
        let orch = generate_adventure(&sample("gates", AdventureStyle::Orchestral)).unwrap();

        let melody = |section: &PortableSection| -> Vec<(u32, u32)> {
            section
                .events
                .iter()
                .filter(|event| event.is_melody())
                .map(|event| (event.start_tick(), event.duration_ticks()))
                .collect()
        };
        let folk_melody = melody(folk.section("camp").unwrap());
        let orch_melody = melody(orch.section("camp").unwrap());

        assert!(
            orch_melody.len() < folk_melody.len(),
            "orchestral lead should have fewer, broader onsets than folk lilt"
        );
        let avg_duration = |events: &[(u32, u32)]| {
            events.iter().map(|(_, duration)| *duration).sum::<u32>() as f64 / events.len() as f64
        };
        assert!(
            avg_duration(&orch_melody) > avg_duration(&folk_melody),
            "orchestral lead should sing legato while folk lilts"
        );
    }

    #[test]
    fn new_phases_carry_their_own_material_and_stay_in_mode_and_register() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let input = sample("new-phases", style);
            let dna = PieceDna::new(subseed(&input.secret, &input.seed, "piece"));
            let tonic = dna.tonic_pitch_class;
            let score = generate_adventure(&input).unwrap();
            let mut ids = HashSet::new();
            for id in [
                "skirmish",
                "assault",
                "chase",
                "festival",
                "reunion",
                "dawn",
            ] {
                let section = score.section(id).unwrap_or_else(|| panic!("missing {id}"));
                assert!(!section.events.is_empty(), "{id} must carry music");
                let pitch_classes: HashSet<i32> =
                    mode_intervals(mode_for(style, section_scene_for(id).unwrap()))
                        .into_iter()
                        .map(|interval| (tonic + interval).rem_euclid(12))
                        .collect();
                for event in &section.events {
                    // Unique ids and safe registers across the new material.
                    assert!(ids.insert(event_id(event)), "duplicate id in {id}");
                    if let Some(pitch) = event.pitch() {
                        assert!(
                            (36..=84).contains(&i32::from(pitch)),
                            "{id} unsafe pitch {pitch}"
                        );
                        assert!(
                            pitch_classes.contains(&(i32::from(pitch) % 12)),
                            "{id} pitch {pitch} left its mode"
                        );
                    }
                }
            }
        }
    }

    fn section_scene_for(id: &str) -> Option<Scene> {
        SECTION_PLANS.iter().find(|plan| plan.id == id).map(|plan| plan.scene)
    }

    #[test]
    fn new_phases_are_distinct_from_their_family_and_from_each_other() {
        let score = generate_adventure(&sample("new-distinct", AdventureStyle::Folk)).unwrap();
        let signature = |id: &str| {
            let section = score.section(id).expect(id);
            musical_signature(section, 0, section.length_ticks)
        };
        // Each new phase must not clone the existing phase in its family.
        for (new_id, parent_id) in [
            ("skirmish", "combat"),
            ("assault", "boss"),
            ("chase", "combat"),
            ("festival", "town"),
            ("reunion", "town"),
            ("dawn", "sanctuary"),
        ] {
            assert_ne!(
                signature(new_id),
                signature(parent_id),
                "{new_id} must not clone {parent_id}"
            );
        }
        // And the six new phases are pairwise distinct from each other.
        let new_ids = [
            "skirmish",
            "assault",
            "chase",
            "festival",
            "reunion",
            "dawn",
        ];
        for left in 0..new_ids.len() {
            for right in left + 1..new_ids.len() {
                assert_ne!(
                    signature(new_ids[left]),
                    signature(new_ids[right]),
                    "{} must differ from {}",
                    new_ids[left],
                    new_ids[right]
                );
            }
        }
    }

    #[test]
    fn combat_set_drives_and_the_happy_set_brightens_without_noising_the_calm_phases() {
        for style in [
            AdventureStyle::Folk,
            AdventureStyle::Dark,
            AdventureStyle::Orchestral,
        ] {
            let score = generate_adventure(&sample("liveliness", style)).unwrap();
            let density = |id: &str| {
                let section = score.section(id).unwrap();
                let bars = section.length_ticks / score.bar_ticks();
                let count = section
                    .events
                    .iter()
                    .filter(|event| matches!(event, MusicEvent::Percussion { .. }))
                    .count();
                count as f64 / f64::from(bars)
            };
            // Combat drives: every combat phase is denser than the calm break.
            for combat in ["skirmish", "assault", "chase"] {
                assert!(
                    density(combat) > density("sanctuary"),
                    "{style:?} {combat} must drive harder than sanctuary"
                );
                assert!(
                    density(combat) > density("camp"),
                    "{style:?} {combat} must drive harder than camp"
                );
            }
            // Happy brightens: every happy phase is more rhythmic than the calm
            // break, and never noisier than the assault.
            for happy in ["festival", "reunion", "dawn"] {
                assert!(
                    density(happy) > density("sanctuary"),
                    "{style:?} {happy} must be livelier than sanctuary"
                );
                assert!(
                    density(happy) <= density("assault"),
                    "{style:?} {happy} must not out-drum the assault"
                );
            }
        }
    }
}
