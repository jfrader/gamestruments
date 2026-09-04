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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveCondition {
    #[serde(default)]
    pub numeric: serde_json::Value,
    #[serde(default)]
    pub categorical: serde_json::Value,
}

#[derive(Clone, Debug, Default)]
pub struct GameState {
    pub intensity: f64,
    pub position_pressure: f64,
    pub final_lap: bool,
    pub race_phase: String,
    pub finish_result: String,
}

impl PortableScore {
    pub fn bar_ticks(&self) -> u32 {
        self.beats_per_bar * self.ticks_per_beat
    }

    pub fn ticks_per_second(&self) -> f64 {
        self.bpm * f64::from(self.ticks_per_beat) / 60.0
    }

    pub fn section(&self, id: &str) -> Option<&PortableSection> {
        self.sections.iter().find(|section| section.id == id)
    }
}
