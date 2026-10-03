use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const SCORE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableScore {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub bpm: f64,
    pub beats_per_bar: u32,
    pub ticks_per_beat: u32,
    pub crossfade_bars: f64,
    pub default_section: String,
    pub sections: Vec<PortableSection>,
    pub rules: Vec<AdaptiveRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<SongForm>,
}

impl PortableScore {
    /// Shift every pitched event by `semitones`. Intervals, voicings and the
    /// arrangement are preserved, so a score generated for one seed can be
    /// moved into the key of the score it replaces and blend at the seam.
    pub fn transpose(&mut self, semitones: i32) {
        if semitones == 0 {
            return;
        }
        for section in &mut self.sections {
            for event in &mut section.events {
                if let MusicEvent::Note { pitch, .. } = event {
                    // Shift by whole octaves at the MIDI bounds so a chord's
                    // intervals survive at the extremes instead of clamping
                    // single notes onto the limit.
                    let mut shifted = i32::from(*pitch) + semitones;
                    while shifted < 0 {
                        shifted += 12;
                    }
                    while shifted > 127 {
                        shifted -= 12;
                    }
                    *pitch = shifted as u8;
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SongFormStep {
    pub section: String,
    #[serde(default = "one_repeat", skip_serializing_if = "is_one_repeat")]
    pub repeats: u32,
}

fn one_repeat() -> u32 {
    1
}

fn is_one_repeat(value: &u32) -> bool {
    *value == 1
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SongForm {
    pub steps: Vec<SongFormStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_from: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<FormOrigin>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FormOrigin {
    TransitionStart,
    TransitionEnd,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableSection {
    pub id: String,
    pub label: String,
    pub feeling: String,
    pub color: String,
    pub length_ticks: u32,
    pub events: Vec<MusicEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MusicEvent {
    #[serde(rename_all = "camelCase")]
    Note {
        id: String,
        section: String,
        lane: String,
        start_tick: u32,
        duration_ticks: u32,
        velocity: f64,
        pitch: u8,
        voice: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Percussion {
        id: String,
        section: String,
        lane: String,
        start_tick: u32,
        duration_ticks: u32,
        velocity: f64,
        voice: String,
    },
}

impl MusicEvent {
    pub fn start_tick(&self) -> u32 {
        match self {
            Self::Note { start_tick, .. } | Self::Percussion { start_tick, .. } => *start_tick,
        }
    }

    pub fn duration_ticks(&self) -> u32 {
        match self {
            Self::Note { duration_ticks, .. } | Self::Percussion { duration_ticks, .. } => {
                *duration_ticks
            }
        }
    }

    pub fn velocity(&self) -> f64 {
        match self {
            Self::Note { velocity, .. } | Self::Percussion { velocity, .. } => *velocity,
        }
    }

    pub fn voice(&self) -> &str {
        match self {
            Self::Note { voice, .. } | Self::Percussion { voice, .. } => voice,
        }
    }

    pub fn is_melody(&self) -> bool {
        matches!(
            self,
            Self::Note {
                role: Some(role),
                ..
            } if role == "melody"
        )
    }

    pub fn pitch(&self) -> Option<u8> {
        match self {
            Self::Note { pitch, .. } => Some(*pitch),
            Self::Percussion { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveRule {
    pub target: String,
    pub priority: i32,
    pub when: AdaptiveCondition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveCondition {
    #[serde(default)]
    pub numeric: serde_json::Value,
    #[serde(default)]
    pub categorical: serde_json::Value,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub intensity: f64,
    pub position_pressure: f64,
    pub final_lap: bool,
    pub race_phase: String,
    pub finish_result: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceState {
    pub phase: String,
    pub heat: f64,
    pub focus: f64,
    pub progress: f64,
}

/// Area state a game is in for the `adventure` recipe. Discovery and threat
/// drive section choice; `quest_complete` always wins.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdventureState {
    pub area_phase: String,
    pub discovery: f64,
    pub threat: f64,
    pub quest_complete: bool,
}

impl PortableScore {
    pub fn bar_ticks(&self) -> u32 {
        self.beats_per_bar.saturating_mul(self.ticks_per_beat)
    }

    pub fn ticks_per_second(&self) -> f64 {
        self.bpm * f64::from(self.ticks_per_beat) / 60.0
    }

    pub fn section(&self, id: &str) -> Option<&PortableSection> {
        self.sections.iter().find(|section| section.id == id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCORE_SCHEMA_VERSION {
            return Err(format!(
                "unsupported schema version {}",
                self.schema_version
            ));
        }
        if self.id.is_empty() {
            return Err("score id is empty".into());
        }
        if self.title.is_empty() {
            return Err("score title is empty".into());
        }
        if !self.bpm.is_finite() || self.bpm <= 0.0 {
            return Err("bpm must be positive".into());
        }
        if self.beats_per_bar == 0 {
            return Err("beats_per_bar must be positive".into());
        }
        if self.ticks_per_beat == 0 {
            return Err("ticks_per_beat must be positive".into());
        }
        if !self.crossfade_bars.is_finite() || self.crossfade_bars <= 0.0 {
            return Err("crossfade_bars must be positive".into());
        }
        if self.sections.is_empty() {
            return Err("at least one section is required".into());
        }

        let bar_ticks = self
            .beats_per_bar
            .checked_mul(self.ticks_per_beat)
            .ok_or_else(|| "bar length exceeds timing bounds".to_string())?;
        let crossfade_ticks = self.crossfade_bars * f64::from(bar_ticks);
        if !crossfade_ticks.is_finite()
            || crossfade_ticks > f64::from(u32::MAX)
            || crossfade_ticks.fract().abs() > f64::EPSILON
        {
            return Err("crossfade duration must resolve to integer ticks".into());
        }

        let mut section_ids = HashSet::new();
        let mut event_ids = HashSet::new();
        for section in &self.sections {
            if section.id.is_empty() {
                return Err("section id is empty".into());
            }
            if !section_ids.insert(section.id.as_str()) {
                return Err(format!("duplicate section id {}", section.id));
            }
            if section.label.is_empty() {
                return Err(format!("section {} has no label", section.id));
            }
            if section.length_ticks == 0 {
                return Err(format!("section {} has invalid length", section.id));
            }

            for event in &section.events {
                validate_event(event, section, &mut event_ids)?;
            }
        }

        if !section_ids.contains(self.default_section.as_str()) {
            return Err(format!("unknown default section {}", self.default_section));
        }
        for rule in &self.rules {
            if !section_ids.contains(rule.target.as_str()) {
                return Err(format!("rule targets unknown section {}", rule.target));
            }
            validate_numeric_conditions(&rule.when.numeric)?;
            if !rule.when.categorical.is_object() {
                return Err(format!(
                    "categorical conditions for {} must be an object",
                    rule.target
                ));
            }
        }
        if let Some(form) = &self.form {
            validate_song_form(form, &section_ids)?;
        }
        Ok(())
    }
}

fn validate_song_form(form: &SongForm, section_ids: &HashSet<&str>) -> Result<(), String> {
    if form.steps.is_empty() {
        return Err("song form must contain at least one step".into());
    }
    for (index, step) in form.steps.iter().enumerate() {
        if step.section.is_empty() {
            return Err(format!("song form step {index} has an empty section"));
        }
        if !section_ids.contains(step.section.as_str()) {
            return Err(format!(
                "song form step {index} targets unknown section {}",
                step.section
            ));
        }
        if step.repeats == 0 {
            return Err(format!("song form step {index} repeats must be positive"));
        }
    }
    if let Some(loop_from) = form.loop_from {
        if (loop_from as usize) >= form.steps.len() {
            return Err(format!("song form loopFrom {loop_from} is out of range"));
        }
    }
    Ok(())
}

fn validate_event<'a>(
    event: &'a MusicEvent,
    section: &PortableSection,
    event_ids: &mut HashSet<&'a str>,
) -> Result<(), String> {
    let (id, event_section, lane, velocity, voice, role) = match event {
        MusicEvent::Note {
            id,
            section,
            lane,
            velocity,
            voice,
            role,
            ..
        } => (id, section, lane, velocity, voice, role.as_deref()),
        MusicEvent::Percussion {
            id,
            section,
            lane,
            velocity,
            voice,
            ..
        } => (id, section, lane, velocity, voice, None),
    };
    if id.is_empty() {
        return Err(format!(
            "section {} contains an event without an id",
            section.id
        ));
    }
    if !event_ids.insert(id.as_str()) {
        return Err(format!("duplicate event id {id}"));
    }
    if event_section != &section.id {
        return Err(format!("event {id} belongs to {event_section}"));
    }
    if lane.is_empty() {
        return Err(format!("event {id} has no lane"));
    }
    if event.duration_ticks() == 0 {
        return Err(format!("event {id} has invalid duration_ticks"));
    }
    let end_tick = event
        .start_tick()
        .checked_add(event.duration_ticks())
        .ok_or_else(|| format!("event {id} exceeds timing bounds"))?;
    if end_tick > section.length_ticks {
        return Err(format!("event {id} exceeds section {}", section.id));
    }
    if !velocity.is_finite() || !(0.0..=1.0).contains(velocity) {
        return Err(format!("event {id} has invalid velocity"));
    }

    match event {
        MusicEvent::Note { pitch, .. } => {
            const NOTE_VOICES: [&str; 23] = [
                "warm",
                "glass",
                "pulse",
                "bass",
                "pluck",
                "chip",
                "epiano",
                "organ",
                "supersaw",
                "triangle",
                "felt",
                "dusk",
                "harp",
                "recorder",
                "vielle",
                "bell",
                "saw-bass",
                "trance-pad",
                "trance-lead",
                "nylon-guitar",
                "charango",
                "quena",
                "marimba",
            ];
            if !NOTE_VOICES.contains(&voice.as_str()) {
                return Err(format!("note {id} has unsupported voice {voice}"));
            }
            if *pitch > 127 {
                return Err(format!("note {id} has invalid pitch"));
            }
            if role.is_some_and(|value| value != "melody") {
                return Err(format!("note {id} has unsupported role"));
            }
        }
        MusicEvent::Percussion { .. } => {
            const PERCUSSION_VOICES: [&str; 12] = [
                "techno-kick",
                "clap",
                "kick",
                "snare",
                "hat",
                "tom",
                "reverse-cymbal",
                "air-impact",
                "frame-drum",
                "tambourine",
                "bombo",
                "bombo-rim",
            ];
            if !PERCUSSION_VOICES.contains(&voice.as_str()) {
                return Err(format!("percussion {id} has unsupported voice {voice}"));
            }
        }
    }
    Ok(())
}

fn validate_numeric_conditions(value: &serde_json::Value) -> Result<(), String> {
    let conditions = value
        .as_object()
        .ok_or_else(|| "numeric conditions must be an object".to_string())?;
    for (name, value) in conditions {
        let range = value
            .as_object()
            .ok_or_else(|| format!("{name} range must be an object"))?;
        let minimum = range.get("min").and_then(serde_json::Value::as_f64);
        let maximum = range.get("max").and_then(serde_json::Value::as_f64);
        if range.contains_key("min") && minimum.is_none() {
            return Err(format!("{name} has invalid minimum"));
        }
        if range.contains_key("max") && maximum.is_none() {
            return Err(format!("{name} has invalid maximum"));
        }
        if minimum.zip(maximum).is_some_and(|(min, max)| min > max) {
            return Err(format!("{name} has an inverted range"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MusicEvent, PortableScore, PortableSection, SCORE_SCHEMA_VERSION};

    fn valid_score() -> PortableScore {
        PortableScore {
            schema_version: SCORE_SCHEMA_VERSION,
            id: "score".into(),
            title: "Score".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 960,
            crossfade_bars: 1.0,
            default_section: "main".into(),
            sections: vec![PortableSection {
                id: "main".into(),
                label: "Main".into(),
                feeling: "steady".into(),
                color: "#ffffff".into(),
                length_ticks: 3840,
                events: vec![MusicEvent::Note {
                    id: "note".into(),
                    section: "main".into(),
                    lane: "melody".into(),
                    start_tick: 0,
                    duration_ticks: 960,
                    velocity: 0.8,
                    pitch: 60,
                    voice: "chip".into(),
                    role: Some("melody".into()),
                }],
            }],
            rules: Vec::new(),
            form: None,
        }
    }

    #[test]
    fn validates_a_well_formed_score() {
        assert_eq!(valid_score().validate(), Ok(()));
    }

    #[test]
    fn rejects_an_event_beyond_its_section() {
        let mut score = valid_score();
        if let MusicEvent::Note { start_tick, .. } = &mut score.sections[0].events[0] {
            *start_tick = 3839;
        }
        assert_eq!(
            score.validate(),
            Err("event note exceeds section main".into())
        );
    }

    #[test]
    fn rejects_a_pitch_outside_midi_range() {
        let mut score = valid_score();
        if let MusicEvent::Note { pitch, .. } = &mut score.sections[0].events[0] {
            *pitch = 128;
        }
        assert_eq!(score.validate(), Err("note note has invalid pitch".into()));
    }

    fn percussion_score(voice: &str) -> PortableScore {
        let mut score = valid_score();
        score.sections[0].events = vec![MusicEvent::Percussion {
            id: "percussion".into(),
            section: "main".into(),
            lane: "percussion".into(),
            start_tick: 0,
            duration_ticks: 960,
            velocity: 0.6,
            voice: voice.into(),
        }];
        score
    }

    #[test]
    fn accepts_the_adventure_acoustic_percussion_voices() {
        for voice in ["frame-drum", "tambourine"] {
            assert_eq!(percussion_score(voice).validate(), Ok(()), "{voice}");
        }
    }

    #[test]
    fn accepts_the_club_percussion_voices() {
        for voice in ["techno-kick", "clap"] {
            assert_eq!(percussion_score(voice).validate(), Ok(()), "{voice}");
        }
    }

    #[test]
    fn accepts_folklore_percussion_voices() {
        for voice in ["bombo", "bombo-rim"] {
            assert_eq!(percussion_score(voice).validate(), Ok(()), "{voice}");
        }
    }

    #[test]
    fn rejects_unknown_percussion_voices() {
        for voice in ["frame_drum", "tamb", "cowbell", "bongo"] {
            assert_eq!(
                percussion_score(voice).validate(),
                Err(format!(
                    "percussion percussion has unsupported voice {voice}"
                ))
            );
        }
    }
}
