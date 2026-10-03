use crate::dmath;
use crate::score::MusicEvent;

use serde::Deserialize;
use std::collections::VecDeque;
use std::f32::consts::TAU;

const NOTE_TAIL_SECONDS: f32 = 0.16;
const MIN_GAIN: f32 = 0.0001;
/// Sidechain pumping: how deep a pumped voice ducks when a club kick lands,
/// and how fast it recovers.
const PUMP_DEPTH: f32 = 0.7;
const PUMP_RECOVERY_SECONDS: f32 = 0.11;
/// How long a voice takes to fade in or out when the solo changes.
const SOLO_FADE_SECONDS: f32 = 0.018;

/// Baked equal-power pan table, 33 entries for pan steps of 1/16 from -1 to +1.
/// gL = cos(θ), gR = sin(θ), θ = (pan + 1) * π/4.
/// Centre (pan=0) gives −3.01 dB per side (constant power).
/// Computed once; lookup only — no sin/cos/dmath calls on the audio path.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
const PAN_TABLE: [[f32; 2]; 33] = [
    [1.00000000e+00f32, 0.00000000e+00f32],
    [9.98795456e-01f32, 4.90676743e-02f32],
    [9.95184727e-01f32, 9.80171403e-02f32],
    [9.89176510e-01f32, 1.46730474e-01f32],
    [9.80785280e-01f32, 1.95090322e-01f32],
    [9.70031253e-01f32, 2.42980180e-01f32],
    [9.56940336e-01f32, 2.90284677e-01f32],
    [9.41544065e-01f32, 3.36889853e-01f32],
    [9.23879533e-01f32, 3.82683432e-01f32],
    [9.03989293e-01f32, 4.27555093e-01f32],
    [8.81921264e-01f32, 4.71396737e-01f32],
    [8.57728610e-01f32, 5.14102744e-01f32],
    [8.31469612e-01f32, 5.55570233e-01f32],
    [8.03207531e-01f32, 5.95699304e-01f32],
    [7.73010453e-01f32, 6.34393284e-01f32],
    [7.40951125e-01f32, 6.71558955e-01f32],
    [7.07106781e-01f32, 7.07106781e-01f32],
    [6.71558955e-01f32, 7.40951125e-01f32],
    [6.34393284e-01f32, 7.73010453e-01f32],
    [5.95699304e-01f32, 8.03207531e-01f32],
    [5.55570233e-01f32, 8.31469612e-01f32],
    [5.14102744e-01f32, 8.57728610e-01f32],
    [4.71396737e-01f32, 8.81921264e-01f32],
    [4.27555093e-01f32, 9.03989293e-01f32],
    [3.82683432e-01f32, 9.23879533e-01f32],
    [3.36889853e-01f32, 9.41544065e-01f32],
    [2.90284677e-01f32, 9.56940336e-01f32],
    [2.42980180e-01f32, 9.70031253e-01f32],
    [1.95090322e-01f32, 9.80785280e-01f32],
    [1.46730474e-01f32, 9.89176510e-01f32],
    [9.80171403e-02f32, 9.95184727e-01f32],
    [4.90676743e-02f32, 9.98795456e-01f32],
    [6.12323400e-17f32, 1.00000000e+00f32],
];

fn pan_gains(pan: f32) -> (f32, f32) {
    let idx = ((pan + 1.0) * 16.0).round().clamp(0.0, 32.0) as usize;
    let [gl, gr] = PAN_TABLE[idx];
    (gl, gr)
}

/// Fixed per-VoiceType pan (quantised to 1/16 steps). Centre for low anchors;
/// small spreads for body, larger for highs/air. Echoes get opposite at call site.
fn voice_type_pan(vt: VoiceType) -> f32 {
    match vt {
        // centre (0.0)
        VoiceType::Kick
        | VoiceType::Bass
        | VoiceType::Organ
        | VoiceType::Warm
        | VoiceType::Pulse
        | VoiceType::Triangle => 0.0,
        // ±0.15
        VoiceType::Snare => 0.125,
        VoiceType::FrameDrum => -0.125,
        VoiceType::Bombo => 0.0,
        VoiceType::BomboRim => 0.0625,
        // ±0.25
        VoiceType::Epiano => 0.25,
        VoiceType::Tom => -0.25,
        VoiceType::Recorder => 0.25,
        VoiceType::Vielle => -0.25,
        VoiceType::NylonGuitar => 0.1875,
        VoiceType::Harp => 0.1875,
        VoiceType::Charango => -0.3125,
        VoiceType::Quena => 0.3125,
        VoiceType::Marimba => 0.0625,
        VoiceType::TrancePad => 0.0,
        VoiceType::TranceLead => -0.125,
        VoiceType::TechnoKick => 0.0,
        VoiceType::Clap => 0.0625,
        VoiceType::SawBass => 0.0,
        // ±0.35
        VoiceType::Pluck => 0.3125,
        VoiceType::Chip => -0.3125,
        VoiceType::Bell => 0.375,
        // ±0.55 for atmosphere + echoes (echo sign flipped at creation)
        VoiceType::Felt => 0.5625,
        VoiceType::Dusk => -0.5625,
        // ±0.65
        VoiceType::Glass => 0.625,
        VoiceType::Tambourine => -0.625,
        VoiceType::Supersaw => 0.6875,
        // ±0.8
        VoiceType::Hat => 0.8125,
        VoiceType::ReverseCymbal => -0.8125,
        VoiceType::AirImpact => 0.8125,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VoiceType {
    Warm,
    Glass,
    Pulse,
    Pluck,
    Felt,
    Dusk,
    Harp,
    Recorder,
    Vielle,
    NylonGuitar,
    Charango,
    Quena,
    Marimba,
    Bell,
    TrancePad,
    TranceLead,
    TechnoKick,
    Clap,
    SawBass,
    FrameDrum,
    Tambourine,
    Bombo,
    BomboRim,
    Bass,
    Epiano,
    Organ,
    Supersaw,
    Triangle,
    Chip,
    Kick,
    Snare,
    Hat,
    Tom,
    ReverseCymbal,
    AirImpact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FilterMode {
    Lowpass,
    Highpass,
    Bandpass,
}

#[derive(Clone)]
struct Biquad {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a0: f32,
    a1: f32,
    a2: f32,
    coefficient_key: Option<(u32, u32, u32, FilterMode)>,
}

impl Biquad {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            coefficient_key: None,
        }
    }

    fn process(&mut self, x: f32, fc: f32, q: f32, sr: f32, mode: FilterMode) -> f32 {
        let fc = fc.max(20.0).min(sr * 0.49);
        let q = q.max(0.1);
        let coefficient_key = (fc.to_bits(), q.to_bits(), sr.to_bits(), mode);
        if self.coefficient_key != Some(coefficient_key) {
            let omega = std::f32::consts::TAU * (fc / sr);
            let sin_om = dmath::sin(omega);
            let cos_om = dmath::cos(omega);
            let alpha = sin_om / (2.0 * q);
            let (b0, b1, b2) = match mode {
                FilterMode::Lowpass => {
                    let b0 = (1.0 - cos_om) * 0.5;
                    let b1 = 1.0 - cos_om;
                    let b2 = b0;
                    (b0, b1, b2)
                }
                FilterMode::Highpass => {
                    let b0 = (1.0 + cos_om) * 0.5;
                    let b1 = -(1.0 + cos_om);
                    let b2 = b0;
                    (b0, b1, b2)
                }
                FilterMode::Bandpass => {
                    let b0 = alpha;
                    let b1 = 0.0;
                    let b2 = -alpha;
                    (b0, b1, b2)
                }
            };
            let a0 = 1.0 + alpha;
            self.b0 = b0;
            self.b1 = b1;
            self.b2 = b2;
            self.a0 = a0;
            self.a1 = -2.0 * cos_om;
            self.a2 = 1.0 - alpha;
            self.coefficient_key = Some(coefficient_key);
        }
        let y = (self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2)
            / self.a0;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

#[derive(Clone)]
struct Voice {
    voice_type: VoiceType,
    base_freq: f32,
    velocity: f32,
    velocity_gain: f32,
    frequency_multipliers: [f32; 3],
    is_melody: bool,
    start_phase: f32,
    duration: f32,
    life: f32,
    pitch: u8,
    noise_state: u32,
    // osc phases (radians)
    phase1: f32,
    phase2: f32,
    phase3: f32,
    vib_phase: f32,
    trem_phase: f32,
    filt: Biquad,
    // pan is quantised at creation; used only by stereo accumulation path.
    // mono fill path ignores it completely.
    pan: f32,
    // dedicated filter states for stereo split on twin-osc voices (Epiano/Supersaw);
    // mono path and non-twin voices only ever touch `filt` (state kept in sync).
    filt_l: Biquad,
    filt_r: Biquad,
    /// Ducks under every club kick (sidechain pumping).
    pumped: bool,
    /// The voice's level under the synth's [`Solo`]: it ramps to 0 or 1.
    solo_level: f32,
}

fn voice_velocity_gain(voice_type: VoiceType, velocity: f32) -> f32 {
    let exponent = match voice_type {
        VoiceType::Triangle => 0.75,
        VoiceType::Bass | VoiceType::Epiano => 0.78,
        VoiceType::Harp | VoiceType::Organ | VoiceType::Supersaw => 0.8,
        VoiceType::Warm
        | VoiceType::Glass
        | VoiceType::Pulse
        | VoiceType::Pluck
        | VoiceType::Felt
        | VoiceType::Dusk
        | VoiceType::Bell
        | VoiceType::TrancePad
        | VoiceType::TranceLead
        | VoiceType::Clap
        | VoiceType::FrameDrum
        | VoiceType::Tambourine
        | VoiceType::Marimba
        | VoiceType::Charango
        | VoiceType::Chip => 0.82,
        VoiceType::NylonGuitar | VoiceType::Bombo | VoiceType::BomboRim => {
            return dmath::powf(velocity, 0.82);
        }
        VoiceType::Recorder | VoiceType::Vielle | VoiceType::Quena => 0.84,
        VoiceType::SawBass => 0.78,
        VoiceType::Kick
        | VoiceType::Snare
        | VoiceType::Hat
        | VoiceType::Tom
        | VoiceType::TechnoKick
        | VoiceType::ReverseCymbal
        | VoiceType::AirImpact => return 1.0,
    };
    velocity_curve(velocity, exponent)
}

fn note_voice_type(name: &str) -> Option<VoiceType> {
    Some(match name {
        "warm" => VoiceType::Warm,
        "glass" => VoiceType::Glass,
        "pulse" => VoiceType::Pulse,
        "pluck" => VoiceType::Pluck,
        "chip" => VoiceType::Chip,
        "organ" => VoiceType::Organ,
        "supersaw" => VoiceType::Supersaw,
        "triangle" => VoiceType::Triangle,
        "bass" => VoiceType::Bass,
        "epiano" => VoiceType::Epiano,
        "felt" => VoiceType::Felt,
        "dusk" => VoiceType::Dusk,
        "harp" => VoiceType::Harp,
        "recorder" => VoiceType::Recorder,
        "vielle" => VoiceType::Vielle,
        "nylon-guitar" => VoiceType::NylonGuitar,
        "charango" => VoiceType::Charango,
        "quena" => VoiceType::Quena,
        "marimba" => VoiceType::Marimba,
        "bell" => VoiceType::Bell,
        "saw-bass" => VoiceType::SawBass,
        "trance-pad" => VoiceType::TrancePad,
        "trance-lead" => VoiceType::TranceLead,
        _ => return None,
    })
}

fn percussion_voice_type(name: &str) -> Option<VoiceType> {
    Some(match name {
        "kick" => VoiceType::Kick,
        "snare" => VoiceType::Snare,
        "hat" => VoiceType::Hat,
        "tom" => VoiceType::Tom,
        "reverse-cymbal" => VoiceType::ReverseCymbal,
        "air-impact" => VoiceType::AirImpact,
        "frame-drum" => VoiceType::FrameDrum,
        "tambourine" => VoiceType::Tambourine,
        "bombo" => VoiceType::Bombo,
        "bombo-rim" => VoiceType::BomboRim,
        "techno-kick" => VoiceType::TechnoKick,
        "clap" => VoiceType::Clap,
        _ => return None,
    })
}

/// Which voices a synth lets through, to audition part of a mix.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Solo {
    #[default]
    Full,
    /// Only the melody line.
    Melody,
    /// Everything but the melody line.
    Rhythm,
    /// One instrument alone, or everything but it when `mute`.
    Voice { voice: String, mute: bool },
}

/// A [`Solo`] resolved to the synth's voice types.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SoloFilter {
    Full,
    Melody,
    Rhythm,
    Voice {
        voice: Option<VoiceType>,
        mute: bool,
    },
}

impl SoloFilter {
    fn new(solo: &Solo) -> Self {
        match solo {
            Solo::Full => Self::Full,
            Solo::Melody => Self::Melody,
            Solo::Rhythm => Self::Rhythm,
            Solo::Voice { voice, mute } => Self::Voice {
                voice: note_voice_type(voice).or_else(|| percussion_voice_type(voice)),
                mute: *mute,
            },
        }
    }

    fn passes(self, voice: &Voice) -> bool {
        match self {
            Self::Full => true,
            Self::Melody => voice.is_melody,
            Self::Rhythm => !voice.is_melody,
            Self::Voice { voice: solo, mute } => (Some(voice.voice_type) == solo) != mute,
        }
    }

    /// The level a voice starts at, so a note that begins soloed out stays silent.
    fn level(self, voice: &Voice) -> f32 {
        if self.passes(voice) {
            1.0
        } else {
            0.0
        }
    }
}

fn voice_frequency_multipliers(voice_type: VoiceType) -> [f32; 3] {
    let multiplier = |cents: f32| dmath::powf(2.0, cents / 1200.0);
    match voice_type {
        VoiceType::Warm => [multiplier(-10.0), multiplier(10.0), 1.0],
        VoiceType::Glass => [multiplier(-6.0), multiplier(6.0), 1.0],
        VoiceType::Pulse => [multiplier(-8.0), multiplier(8.0), 1.0],
        VoiceType::Pluck => [multiplier(-5.0), multiplier(5.0), 1.0],
        VoiceType::Felt => [multiplier(-2.0), multiplier(2.0), 1.0],
        VoiceType::Dusk => [multiplier(-3.0), multiplier(3.0), 1.0],
        VoiceType::Epiano => [multiplier(-7.0), multiplier(7.0), multiplier(3.0)],
        VoiceType::Supersaw => [multiplier(-11.0), 1.0, multiplier(13.0)],
        VoiceType::TrancePad => [1.0, multiplier(12.0), multiplier(-12.0)],
        VoiceType::TranceLead => [multiplier(8.0), multiplier(-8.0), 1.0],
        _ => [1.0; 3],
    }
}

pub struct Synth {
    sample_rate: f32,
    phase: f32,
    voices: Vec<Voice>,
    /// Start times of the club kicks that pump the pumped voices, ascending.
    kicks: VecDeque<f32>,
    solo: SoloFilter,
    /// How far a voice's solo level moves per sample.
    solo_step: f32,
}

impl Synth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            voices: Vec::new(),
            kicks: VecDeque::new(),
            solo: SoloFilter::Full,
            solo_step: 1.0 / (SOLO_FADE_SECONDS * sample_rate),
        }
    }

    /// Let only the voices `solo` names through; the others fade out.
    pub fn set_solo(&mut self, solo: &Solo) {
        self.solo = SoloFilter::new(solo);
    }

    /// Move a voice's level toward what the solo lets through.
    fn solo_gain(solo: SoloFilter, step: f32, voice: &mut Voice) -> f32 {
        let target = solo.level(voice);
        if voice.solo_level != target {
            voice.solo_level = if voice.solo_level < target {
                (voice.solo_level + step).min(target)
            } else {
                (voice.solo_level - step).max(target)
            };
        }
        voice.solo_level
    }

    /// The pumped voices' gain at time `t`: a dip at the latest club kick that
    /// recovers over [`PUMP_RECOVERY_SECONDS`]. Unity when no kick has landed.
    fn pump_gain(&mut self, t: f32) -> f32 {
        while self.kicks.len() > 1 && self.kicks[1] <= t {
            self.kicks.pop_front();
        }
        match self.kicks.front() {
            Some(&kick) if kick <= t => {
                1.0 - PUMP_DEPTH * natural_decay(t - kick, PUMP_RECOVERY_SECONDS)
            }
            _ => 1.0,
        }
    }

    pub fn trigger(&mut self, event: &MusicEvent, ticks_per_second: f64) {
        self.trigger_at(event, ticks_per_second, 0.0);
    }

    /// Schedule `event` to start `offset_seconds` after the current audio
    /// position. A caller filling one buffer from a window of score events
    /// passes each event's offset, so note timing is sample-accurate and does
    /// not depend on how often the caller runs.
    pub fn trigger_at(&mut self, event: &MusicEvent, ticks_per_second: f64, offset_seconds: f64) {
        let duration = event.duration_ticks() as f64 / ticks_per_second;
        let velocity = event.velocity() as f32;
        let voice = event.voice();
        let is_melody = event.is_melody();
        if let Some(pitch) = event.pitch() {
            let base_freq = midi_to_freq(pitch);
            let vtype = note_voice_type(voice).unwrap_or(VoiceType::Warm);
            let life = match vtype {
                VoiceType::Harp => harp_life(base_freq),
                VoiceType::Bell => bell_life(base_freq),
                VoiceType::SawBass => duration as f32 + 0.12,
                VoiceType::TrancePad => duration as f32 + 0.62,
                VoiceType::TranceLead => duration as f32 + 0.17,
                VoiceType::Felt => duration as f32 + 0.65,
                VoiceType::Dusk => duration as f32 + 1.25,
                VoiceType::Recorder => duration as f32 + 0.3,
                VoiceType::Vielle => duration as f32 + 0.55,
                VoiceType::NylonGuitar => duration as f32 + NOTE_TAIL_SECONDS,
                VoiceType::Charango => duration as f32 + 0.12,
                VoiceType::Quena => duration as f32 + 0.3,
                VoiceType::Marimba => duration as f32 + 0.55,
                _ => duration as f32 + NOTE_TAIL_SECONDS + 0.05,
            };
            let noise_state = if matches!(
                vtype,
                VoiceType::Harp
                    | VoiceType::Recorder
                    | VoiceType::Vielle
                    | VoiceType::NylonGuitar
                    | VoiceType::Charango
                    | VoiceType::Quena
                    | VoiceType::Marimba
            ) {
                match event {
                    MusicEvent::Note { id, .. } => deterministic_noise_state(id),
                    _ => unreachable!(),
                }
            } else {
                0x1234_5678
            };
            let velocity = velocity.clamp(0.0, 1.0);
            let mut voice = Voice {
                voice_type: vtype,
                base_freq,
                velocity,
                velocity_gain: voice_velocity_gain(vtype, velocity),
                frequency_multipliers: voice_frequency_multipliers(vtype),
                is_melody,
                start_phase: self.phase + offset_seconds as f32,
                duration: duration as f32,
                life,
                pitch,
                noise_state,
                phase1: 0.0,
                phase2: 0.0,
                phase3: 0.0,
                vib_phase: 0.0,
                trem_phase: 0.0,
                filt: Biquad::new(),
                pan: 0.0,
                filt_l: Biquad::new(),
                filt_r: Biquad::new(),
                pumped: matches!(
                    vtype,
                    VoiceType::SawBass | VoiceType::TrancePad | VoiceType::TranceLead
                ),
                solo_level: 1.0,
            };
            voice.solo_level = self.solo.level(&voice);
            voice.pan = voice_type_pan(voice.voice_type);
            if matches!(vtype, VoiceType::Felt | VoiceType::Dusk) {
                let mut echo = voice.clone();
                echo.start_phase += 0.42;
                echo.velocity *= 0.16;
                echo.velocity_gain = voice_velocity_gain(vtype, echo.velocity);
                echo.pan = -echo.pan;
                // echo gets fresh filter states (clone zeros them); opposite pan already set.
                self.voices.push(echo);
            }
            self.voices.push(voice);
            return;
        }
        // percussion
        let vtype = percussion_voice_type(voice).unwrap_or(VoiceType::Kick);
        let mut base_freq = 80.0f32;
        let life = match vtype {
            VoiceType::Kick => 0.225,
            VoiceType::Snare => 0.125,
            VoiceType::Hat => 0.085,
            VoiceType::Tom => 0.20,
            VoiceType::FrameDrum => 0.62,
            VoiceType::Tambourine => 0.48,
            VoiceType::Bombo => 0.66,
            VoiceType::BomboRim => 0.18,
            VoiceType::TechnoKick => 0.6,
            VoiceType::Clap => 0.35,
            VoiceType::ReverseCymbal | VoiceType::AirImpact => duration.max(0.04) as f32 + 0.13,
            _ => 0.12,
        };
        let mut perc_id: Option<String> = None;
        if matches!(
            vtype,
            VoiceType::Tom
                | VoiceType::FrameDrum
                | VoiceType::Tambourine
                | VoiceType::Bombo
                | VoiceType::BomboRim
                | VoiceType::TechnoKick
                | VoiceType::Clap
        ) {
            if let MusicEvent::Percussion { id, .. } = event {
                let u = deterministic_unit(id);
                base_freq = match vtype {
                    VoiceType::Tom => 155.0 + u * 58.0,
                    VoiceType::FrameDrum => 82.0 + u * 24.0,
                    VoiceType::Tambourine => u,
                    VoiceType::Bombo => 65.0 + u * 16.0,
                    VoiceType::BomboRim => u * 200.0 + 900.0,
                    VoiceType::TechnoKick | VoiceType::Clap => u,
                    _ => unreachable!(),
                };
                perc_id = Some(id.clone());
            }
        } else if matches!(
            vtype,
            VoiceType::Kick
                | VoiceType::Snare
                | VoiceType::Hat
                | VoiceType::Tom
                | VoiceType::ReverseCymbal
                | VoiceType::AirImpact
        ) {
            if let MusicEvent::Percussion { id, .. } = event {
                perc_id = Some(id.clone());
            }
        }
        let mut noise_state = 0x1234_5678u32;
        if let Some(ref id) = perc_id {
            let noise_dur = match vtype {
                VoiceType::Kick => 0.018,
                VoiceType::Snare => 0.16,
                VoiceType::Hat => 0.08,
                VoiceType::Tom => 0.026,
                VoiceType::FrameDrum => 0.034,
                VoiceType::Tambourine => 0.43,
                VoiceType::Bombo => 0.048,
                VoiceType::BomboRim => 0.015,
                _ => 0.1,
            };
            let buf_dur_s = 0.75f32;
            let available = (buf_dur_s - noise_dur - 0.005).max(0.0);
            let u = deterministic_unit(id);
            let offset_s = u * available;
            let offset_n = (offset_s * self.sample_rate).floor() as u32;
            let mut s = 0x1234_5678u32;
            for _ in 0..offset_n {
                s ^= s << 13;
                s ^= s >> 17;
                s ^= s << 5;
            }
            noise_state = s;
        }
        let velocity = velocity.clamp(0.0, 1.0);
        let mut perc = Voice {
            voice_type: vtype,
            base_freq,
            velocity,
            velocity_gain: voice_velocity_gain(vtype, velocity),
            frequency_multipliers: voice_frequency_multipliers(vtype),
            is_melody: false,
            start_phase: self.phase + offset_seconds as f32,
            duration: if matches!(vtype, VoiceType::ReverseCymbal | VoiceType::AirImpact) {
                duration.max(0.04) as f32
            } else {
                life
            },
            life: life + 0.01,
            pitch: 0,
            noise_state,
            phase1: 0.0,
            phase2: 0.0,
            phase3: 0.0,
            vib_phase: 0.0,
            trem_phase: 0.0,
            filt: Biquad::new(),
            pan: 0.0,
            filt_l: Biquad::new(),
            filt_r: Biquad::new(),
            pumped: false,
            solo_level: 1.0,
        };
        perc.solo_level = self.solo.level(&perc);
        perc.pan = voice_type_pan(perc.voice_type);
        if vtype == VoiceType::TechnoKick {
            let at = perc.start_phase;
            let index = self.kicks.partition_point(|kick| *kick <= at);
            self.kicks.insert(index, at);
        }
        self.voices.push(perc);
    }

    pub fn fill(&mut self, buffer: &mut [f32]) {
        let sr = self.sample_rate;
        let dt = 1.0 / sr;
        let mut i = 0;
        while i < buffer.len() {
            let t = self.phase;
            let pump = self.pump_gain(t);
            let mut mix = 0.0f32;
            let mut j = 0;
            while j < self.voices.len() {
                if t < self.voices[j].start_phase {
                    j += 1;
                    continue;
                }
                let age = (t - self.voices[j].start_phase).max(0.0);
                if age >= self.voices[j].life {
                    self.voices.swap_remove(j);
                    continue;
                }
                let contrib = Synth::generate_voice_sample(&mut self.voices[j], age, sr, dt)
                    * Synth::solo_gain(self.solo, self.solo_step, &mut self.voices[j]);
                mix += if self.voices[j].pumped {
                    contrib * pump
                } else {
                    contrib
                };
                j += 1;
            }
            buffer[i] = mix.clamp(-4.0, 4.0);
            self.phase += dt;
            i += 1;
        }
    }

    /// Stereo accumulation path. Applies per-voice pan via baked table.
    /// The mono `fill` body above is left **byte-for-byte identical** in source and arithmetic.
    /// Twin-osc voices (Epiano, Supersaw) split their detuned oscillators L/R for free width;
    /// Felt/Dusk echoes are already opposite-pan voices so get opposite placement automatically.
    pub fn fill_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(left.len(), right.len(), "stereo buffers must match length");
        let sr = self.sample_rate;
        let dt = 1.0 / sr;
        let mut i = 0;
        while i < left.len() {
            let t = self.phase;
            let pump = self.pump_gain(t);
            let mut mix_l = 0.0f32;
            let mut mix_r = 0.0f32;
            let mut j = 0;
            while j < self.voices.len() {
                if t < self.voices[j].start_phase {
                    j += 1;
                    continue;
                }
                let age = (t - self.voices[j].start_phase).max(0.0);
                if age >= self.voices[j].life {
                    self.voices.swap_remove(j);
                    continue;
                }
                let vpan = self.voices[j].pan;
                let (cl, cr) = if matches!(
                    self.voices[j].voice_type,
                    VoiceType::Epiano | VoiceType::Supersaw
                ) {
                    Synth::generate_voice_sample_stereo(&mut self.voices[j], age, sr, dt)
                } else {
                    let c = Synth::generate_voice_sample(&mut self.voices[j], age, sr, dt);
                    (c, c)
                };
                let (gl, gr) = pan_gains(vpan);
                let gain = if self.voices[j].pumped { pump } else { 1.0 }
                    * Synth::solo_gain(self.solo, self.solo_step, &mut self.voices[j]);
                mix_l += cl * gl * gain;
                mix_r += cr * gr * gain;
                j += 1;
            }
            left[i] = mix_l.clamp(-4.0, 4.0);
            right[i] = mix_r.clamp(-4.0, 4.0);
            self.phase += dt;
            i += 1;
        }
    }

    fn generate_voice_sample(v: &mut Voice, age: f32, sr: f32, dt: f32) -> f32 {
        let vel = v.velocity;
        let is_mel = v.is_melody;
        let base = v.base_freq;
        match v.voice_type {
            VoiceType::Warm
            | VoiceType::Glass
            | VoiceType::Pulse
            | VoiceType::Pluck
            | VoiceType::Felt
            | VoiceType::Dusk => {
                let (primary, secondary, sec_r, sec_g, g, att, dec, sus, rel, cs, ce, res, pd) =
                    match v.voice_type {
                        VoiceType::Warm => (
                            Wave::Saw,
                            Wave::Triangle,
                            1.002,
                            0.62,
                            0.068,
                            0.048,
                            0.22,
                            0.74,
                            0.2,
                            1600.0,
                            620.0,
                            0.35,
                            0.0,
                        ),
                        VoiceType::Glass => (
                            Wave::Sine,
                            Wave::Sine,
                            2.003,
                            0.22,
                            0.07,
                            0.01,
                            0.18,
                            0.5,
                            0.16,
                            3800.0,
                            1400.0,
                            0.45,
                            0.0,
                        ),
                        VoiceType::Pulse => (
                            Wave::Saw,
                            Wave::Triangle,
                            0.5,
                            0.38,
                            0.05,
                            0.014,
                            0.14,
                            0.66,
                            0.14,
                            1900.0,
                            780.0,
                            0.4,
                            0.0,
                        ),
                        VoiceType::Pluck => (
                            Wave::Saw,
                            Wave::Triangle,
                            2.0,
                            0.16,
                            0.05,
                            0.004,
                            0.09,
                            0.22,
                            0.08,
                            3400.0,
                            720.0,
                            0.9,
                            0.004,
                        ),
                        VoiceType::Felt => (
                            Wave::Sine,
                            Wave::Triangle,
                            2.0,
                            0.06,
                            0.06,
                            0.025,
                            0.35,
                            0.12,
                            0.6,
                            1400.0,
                            420.0,
                            0.25,
                            0.0,
                        ),
                        VoiceType::Dusk => (
                            Wave::Triangle,
                            Wave::Sine,
                            1.001,
                            0.35,
                            0.055,
                            0.4,
                            0.9,
                            0.55,
                            1.2,
                            900.0,
                            500.0,
                            0.25,
                            0.0,
                        ),
                        _ => unreachable!(),
                    };
                let f1 = compute_freq(base, age, pd) * v.frequency_multipliers[0];
                let f2 = compute_freq(base * sec_r, age, pd * 0.5) * v.frequency_multipliers[1];
                let s1 = generate_osc(v.phase1, primary);
                v.phase1 += TAU * f1 * dt;
                let s2 = generate_osc(v.phase2, secondary);
                v.phase2 += TAU * f2 * dt;
                let mut sig = s1 + s2 * sec_g;
                // filter sweep
                let bright = 0.72 + vel * 0.48;
                let mut fc = (cs * bright).min(sr * 0.44);
                let end_fc = (ce * (0.82 + vel * 0.28)).max(120.0);
                let ramp_d = v.duration.min(dec);
                if ramp_d > 0.0 && age < ramp_d {
                    let frac = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, frac);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = res + if is_mel { 0.25 } else { 0.0 };
                sig = v.filt.process(sig, fc, q, sr, FilterMode::Lowpass);
                let peak = g * v.velocity_gain * if is_mel { 1.18 } else { 1.0 };
                let cap = if matches!(v.voice_type, VoiceType::Felt | VoiceType::Dusk) {
                    rel
                } else {
                    0.24
                };
                let env = compute_envelope_with_cap(age, v.duration, peak, sus, att, dec, rel, cap);
                sig * env
            }
            VoiceType::Harp => {
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let octave = generate_osc(v.phase2, Wave::Sine);
                let upper = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * compute_freq(base, age, 0.004) * dt;
                v.phase2 += TAU * base * 2.006 * dt;
                v.phase3 += TAU * base * 3.012 * dt;

                let lower_note = (220.0 / base).clamp(0.25, 1.0);
                let body_decay = natural_decay(age, 0.72 + lower_note * 0.52);
                let octave_decay = natural_decay(age, 0.34 + lower_note * 0.22);
                let upper_decay = natural_decay(age, 0.13 + lower_note * 0.08);
                let attack = (age / 0.003).clamp(0.0, 1.0);
                let string = fundamental * body_decay
                    + octave * 0.28 * octave_decay
                    + upper * 0.12 * upper_decay;
                let excitation = if age < 0.032 {
                    let transient = 1.0 - age / 0.032;
                    let sample = noise(&mut v.noise_state);
                    v.filt.process(
                        sample,
                        (base * 5.5).clamp(900.0, 3200.0),
                        0.7,
                        sr,
                        FilterMode::Bandpass,
                    ) * transient
                        * 0.12
                } else {
                    0.0
                };
                (string + excitation)
                    * attack
                    * 0.105
                    * v.velocity_gain
                    * if is_mel { 1.08 } else { 1.0 }
            }
            VoiceType::NylonGuitar => {
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let octave = generate_osc(v.phase2, Wave::Triangle);
                let upper = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * compute_freq(base, age, 0.003) * dt;
                v.phase2 += TAU * base * 2.01 * dt;
                v.phase3 += TAU * base * 3.015 * dt;

                let lower_note = (220.0 / base).clamp(0.25, 1.0);
                let body_decay = natural_decay(age, 0.82 + lower_note * 0.55);
                let harm_decay = natural_decay(age, 0.42 + lower_note * 0.28);
                let upper_decay = natural_decay(age, 0.17 + lower_note * 0.11);
                let attack = (age / 0.004).clamp(0.0, 1.0);
                let string = fundamental * body_decay
                    + octave * 0.32 * harm_decay
                    + upper * 0.085 * upper_decay;
                let excitation = if age < 0.028 {
                    let transient = 1.0 - age / 0.028;
                    let sample = noise(&mut v.noise_state);
                    v.filt.process(
                        sample,
                        (base * 4.8).clamp(780.0, 2600.0),
                        0.62,
                        sr,
                        FilterMode::Bandpass,
                    ) * transient
                        * 0.085
                } else {
                    0.0
                };
                let release = (1.0 - (age - v.duration).max(0.0) / NOTE_TAIL_SECONDS).max(0.0);
                (string + excitation)
                    * attack
                    * release
                    * 0.12
                    * v.velocity_gain
                    * if is_mel { 1.06 } else { 1.0 }
            }
            VoiceType::Charango => {
                let course_a = generate_osc(v.phase1, Wave::Triangle);
                let course_b = generate_osc(v.phase2, Wave::Triangle);
                let octave = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * compute_freq(base, age, 0.004) * dt;
                v.phase2 += TAU * compute_freq(base * 1.006, age, 0.004) * dt;
                v.phase3 += TAU * base * 2.0 * dt;

                let lower_note = (280.0 / base).clamp(0.35, 1.0);
                let body = natural_decay(age, 0.22 + lower_note * 0.16);
                let course_decay = natural_decay(age, 0.2 + lower_note * 0.14);
                let octave_decay = natural_decay(age, 0.09 + lower_note * 0.07);
                let attack = (age / 0.0025).clamp(0.0, 1.0);
                let release = (1.0 - (age - v.duration).max(0.0) / (v.life - v.duration)).max(0.0);
                let strings =
                    course_a * body + course_b * 0.7 * course_decay + octave * 0.18 * octave_decay;
                let snap = if age < 0.02 {
                    let transient = 1.0 - age / 0.02;
                    let sample = noise(&mut v.noise_state);
                    v.filt.process(
                        sample,
                        (base * 5.2).clamp(1600.0, 4200.0),
                        0.75,
                        sr,
                        FilterMode::Bandpass,
                    ) * transient
                        * 0.05
                } else {
                    0.0
                };
                (strings + snap)
                    * attack
                    * release
                    * 0.13
                    * v.velocity_gain
                    * if is_mel { 1.06 } else { 1.0 }
            }
            VoiceType::Quena => {
                let vibrato_ramp = ((age - 0.25) / 0.4).clamp(0.0, 1.0);
                let vibrato_cents = dmath::sin(v.vib_phase) * 7.0 * vibrato_ramp;
                v.vib_phase += TAU * (5.4 + (v.pitch % 5) as f32 * 0.06) * dt;
                let frequency = base * dmath::powf(2.0, vibrato_cents / 1200.0);
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let second = generate_osc(v.phase2, Wave::Sine);
                let third = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * frequency * dt;
                v.phase2 += TAU * frequency * 2.0 * dt;
                v.phase3 += TAU * frequency * 3.0 * dt;

                let tone = fundamental * 0.68 + second * 0.24 + third * 0.09;
                let breath_sample = noise(&mut v.noise_state);
                let breath = v.filt.process(
                    breath_sample,
                    (base * 6.0).clamp(1200.0, 3000.0),
                    0.5,
                    sr,
                    FilterMode::Bandpass,
                );
                let breath_attack = (age / 0.1).clamp(0.0, 1.0);
                let sig = tone + breath * (0.04 + 0.04 * (1.0 - breath_attack));
                let peak = 0.09 * v.velocity_gain * if is_mel { 1.08 } else { 1.0 };
                let env =
                    compute_envelope_with_cap(age, v.duration, peak, 0.72, 0.06, 0.14, 0.22, 0.22);
                sig * env
            }
            VoiceType::Marimba => {
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let octave = generate_osc(v.phase2, Wave::Sine);
                let fourth = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * compute_freq(base, age, 0.0012) * dt;
                v.phase2 += TAU * base * 2.0 * dt;
                v.phase3 += TAU * base * 4.01 * dt;

                let lower_note = (220.0 / base).clamp(0.3, 1.0);
                let body = natural_decay(age, 0.55 + lower_note * 0.6);
                let octave_decay = natural_decay(age, 0.3 + lower_note * 0.28);
                let fourth_decay = natural_decay(age, 0.12 + lower_note * 0.12);
                let attack = (age / 0.002).clamp(0.0, 1.0);
                let release = (1.0 - (age - v.duration).max(0.0) / (v.life - v.duration)).max(0.0);
                let wood = fundamental * body
                    + octave * 0.24 * octave_decay
                    + fourth * 0.09 * fourth_decay;
                let mallet = if age < 0.005 {
                    let transient = 1.0 - age / 0.005;
                    let sample = noise(&mut v.noise_state);
                    v.filt.process(
                        sample,
                        (base * 6.0).clamp(1400.0, 3600.0),
                        0.6,
                        sr,
                        FilterMode::Bandpass,
                    ) * transient
                        * 0.06
                } else {
                    0.0
                };
                (wood + mallet)
                    * attack
                    * release
                    * 0.11
                    * v.velocity_gain
                    * if is_mel { 1.05 } else { 1.0 }
            }
            VoiceType::Recorder => {
                let vibrato_ramp = ((age - 0.2) / 0.38).clamp(0.0, 1.0);
                let vibrato_cents = dmath::sin(v.vib_phase) * 6.0 * vibrato_ramp;
                v.vib_phase += TAU * (5.05 + (v.pitch % 4) as f32 * 0.07) * dt;
                let frequency = base * dmath::powf(2.0, vibrato_cents / 1200.0);
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let second = generate_osc(v.phase2, Wave::Sine);
                let third = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * frequency * dt;
                v.phase2 += TAU * frequency * 2.0 * dt;
                v.phase3 += TAU * frequency * 3.0 * dt;

                let tone = fundamental * 0.82 + second * 0.2 + third * 0.075;
                let breath_sample = noise(&mut v.noise_state);
                let breath = v.filt.process(
                    breath_sample,
                    (base * 7.0).clamp(1800.0, 4200.0),
                    0.55,
                    sr,
                    FilterMode::Bandpass,
                );
                let breath_attack = (age / 0.07).clamp(0.0, 1.0);
                let sig = tone + breath * (0.018 + 0.025 * (1.0 - breath_attack));
                let peak = 0.085 * v.velocity_gain * if is_mel { 1.08 } else { 1.0 };
                let env =
                    compute_envelope_with_cap(age, v.duration, peak, 0.78, 0.045, 0.12, 0.24, 0.24);
                sig * env
            }
            VoiceType::Vielle => {
                let vibrato_ramp = ((age - 0.32) / 0.55).clamp(0.0, 1.0);
                let vibrato_cents = dmath::sin(v.vib_phase) * 3.6 * vibrato_ramp;
                v.vib_phase += TAU * (4.65 + (v.pitch % 5) as f32 * 0.045) * dt;
                let frequency = base * dmath::powf(2.0, vibrato_cents / 1200.0);
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let second = generate_osc(v.phase2, Wave::Sine);
                let third = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * frequency * dt;
                v.phase2 += TAU * frequency * 2.0 * dt;
                v.phase3 += TAU * frequency * 3.0 * dt;

                let evolution = dmath::sin(v.trem_phase);
                v.trem_phase += TAU * (0.43 + (v.pitch % 3) as f32 * 0.035) * dt;
                let second_gain = 0.28 + evolution * 0.045;
                let third_gain = 0.115 - evolution * 0.025;
                let tone = fundamental * 0.72 + second * second_gain + third * third_gain;
                let bow_sample = noise(&mut v.noise_state);
                let bow = v.filt.process(
                    bow_sample,
                    (base * 5.0).clamp(1150.0, 2800.0),
                    0.48,
                    sr,
                    FilterMode::Bandpass,
                );
                let sig = tone + bow * 0.025;
                let peak = 0.082 * v.velocity_gain * if is_mel { 1.08 } else { 1.0 };
                let env =
                    compute_envelope_with_cap(age, v.duration, peak, 0.72, 0.085, 0.28, 0.45, 0.45);
                sig * env
            }
            VoiceType::Bell => {
                let fundamental = generate_osc(v.phase1, Wave::Sine);
                let tierce = generate_osc(v.phase2, Wave::Sine);
                let upper = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * base * dt;
                v.phase2 += TAU * base * 2.72 * dt;
                v.phase3 += TAU * base * 4.07 * dt;

                let lower_note = (330.0 / base).clamp(0.35, 1.0);
                let body = fundamental * natural_decay(age, 0.78 + lower_note * 0.62);
                let color = tierce * 0.24 * natural_decay(age, 0.5 + lower_note * 0.28);
                let shimmer = upper * 0.085 * natural_decay(age, 0.19 + lower_note * 0.13);
                let strike = (age / 0.0025).clamp(0.0, 1.0);
                (body + color + shimmer)
                    * strike
                    * 0.09
                    * v.velocity_gain
                    * if is_mel { 1.06 } else { 1.0 }
            }
            VoiceType::SawBass => {
                // A resonant sawtooth bass whose filter snaps shut after the attack.
                let saw = generate_osc(v.phase1, Wave::Saw);
                let sub = generate_osc(v.phase2, Wave::Square);
                v.phase1 += TAU * base * dt;
                v.phase2 += TAU * base * 0.5 * dt;
                let cutoff = 180.0 + 1400.0 * vel * natural_decay(age, 0.09);
                let sig = v
                    .filt
                    .process(saw + sub * 0.35, cutoff, 1.2, sr, FilterMode::Lowpass);
                let env = compute_envelope_with_cap(
                    age,
                    v.duration,
                    0.09 * v.velocity_gain,
                    0.7,
                    0.004,
                    0.15,
                    0.08,
                    0.08,
                );
                sig * env
            }
            VoiceType::TrancePad => {
                // Three detuned saws under a soft filter, swelling in slowly.
                let spread = v.frequency_multipliers;
                let saws = generate_osc(v.phase1, Wave::Saw)
                    + generate_osc(v.phase2, Wave::Saw)
                    + generate_osc(v.phase3, Wave::Saw);
                v.phase1 += TAU * base * spread[0] * dt;
                v.phase2 += TAU * base * spread[1] * dt;
                v.phase3 += TAU * base * spread[2] * dt;
                let sig = v.filt.process(
                    saws / 3.0,
                    (2400.0 + vel * 2000.0).min(sr * 0.44),
                    0.3,
                    sr,
                    FilterMode::Lowpass,
                );
                let env = compute_envelope_with_cap(
                    age,
                    v.duration,
                    0.05 * v.velocity_gain,
                    0.85,
                    0.35,
                    0.5,
                    0.6,
                    0.6,
                );
                sig * env
            }
            VoiceType::TranceLead => {
                // A plucked supersaw lead: the filter snaps open and closes again.
                let saws = generate_osc(v.phase1, Wave::Saw) + generate_osc(v.phase2, Wave::Saw);
                v.phase1 += TAU * base * v.frequency_multipliers[0] * dt;
                v.phase2 += TAU * base * v.frequency_multipliers[1] * dt;
                let cutoff = (600.0 + 5000.0 * vel * natural_decay(age, 0.18)).min(sr * 0.44);
                let sig = v
                    .filt
                    .process(saws * 0.5, cutoff, 0.8, sr, FilterMode::Lowpass);
                let env = compute_envelope_with_cap(
                    age,
                    v.duration,
                    0.07 * v.velocity_gain,
                    0.35,
                    0.003,
                    0.25,
                    0.15,
                    0.15,
                );
                sig * env
            }
            VoiceType::TechnoKick => {
                // A clean club kick: a sine that sweeps down onto the sub, a
                // punch that falls away fast over a shorter sub tail, and a
                // small click on top. No clipping: the weight is the sub.
                let pitch = 48.0 + 110.0 * natural_decay(age, 0.045);
                let body = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * pitch * dt;
                let envelope = 0.6 * natural_decay(age, 0.06) + 0.4 * natural_decay(age, 0.22);
                let click = v.filt.process(
                    noise(&mut v.noise_state),
                    2500.0,
                    0.5,
                    sr,
                    FilterMode::Highpass,
                ) * natural_decay(age, 0.002)
                    * 0.15;
                (body * envelope + click) * (age / 0.001).clamp(0.0, 1.0) * 0.32 * vel
            }
            VoiceType::Clap => {
                // Three hands a hair apart, then the room's short tail.
                let bursts = [0.0f32, 0.011, 0.022]
                    .iter()
                    .map(|start| {
                        if age >= *start && age < start + 0.006 {
                            1.0
                        } else {
                            0.0
                        }
                    })
                    .sum::<f32>()
                    .min(1.0);
                let tail = if age >= 0.022 {
                    natural_decay(age - 0.022, 0.12)
                } else {
                    0.0
                };
                let sig = v.filt.process(
                    noise(&mut v.noise_state),
                    1200.0,
                    0.7,
                    sr,
                    FilterMode::Bandpass,
                );
                sig * (bursts + tail * 0.8) * 0.22 * v.velocity_gain
            }
            VoiceType::Bass => {
                let f_body = compute_freq(base, age, 0.004);
                let s_body = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * f_body * dt;
                let f_sub = base * 0.5;
                let s_sub = generate_osc(v.phase2, Wave::Sine);
                v.phase2 += TAU * f_sub * dt;
                let bg = 0.42 + vel * 0.1;
                let sg = 0.95;
                let mut sig = s_body * bg + s_sub * sg;
                let mut fc = 520.0 + vel * 680.0;
                let end_fc = (base * 2.4).clamp(180.0, 420.0);
                let ramp_d = v.duration.min(0.16f32);
                if age < ramp_d {
                    let frac = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, frac);
                } else {
                    fc = end_fc;
                }
                let q = 0.7 + vel * 0.35;
                sig = v.filt.process(sig, fc, q, sr, FilterMode::Lowpass);
                let peak = 0.12 * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.62, 0.008, 0.12, 0.11);
                sig * env
            }
            VoiceType::Epiano => {
                let f_l = base * v.frequency_multipliers[0];
                let f_r = base * v.frequency_multipliers[1];
                let s_l = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * f_l * dt;
                let s_r = generate_osc(v.phase2, Wave::Sine);
                v.phase2 += TAU * f_r * dt;
                let tine_f = base * (2.001 + vel * 0.003) * v.frequency_multipliers[2];
                let s_t = generate_osc(v.phase3, Wave::Sine);
                v.phase3 += TAU * tine_f * dt;
                let bg = 0.62;
                let mut sig = (s_l + s_r) * bg + s_t * 0.0; // tine g separate
                                                            // tine pre gain ramp
                let tine_peak = 0.11 + dmath::powf(vel, 1.7) * 0.38;
                let tine_dec = 0.09 + (1.0 - vel) * 0.08;
                let tine_d_t = v.duration.min(tine_dec);
                let mut tg = 0.012;
                if age < 0.004 {
                    let fr = age / 0.004;
                    let tgt = tine_peak * 0.7;
                    tg = MIN_GAIN * dmath::powf(tgt / MIN_GAIN, fr);
                } else if age < tine_d_t {
                    let fr = (age - 0.004) / (tine_d_t - 0.004).max(1e-6);
                    let tgt = tine_peak * 0.7;
                    tg = tgt * dmath::powf(0.012 / tgt, fr);
                }
                sig += s_t * tg;
                // filter
                let mut fc = 1100.0 + dmath::powf(vel, 1.4) * 2200.0;
                let end_fc = 780.0 + vel * 420.0;
                let ramp_d = v.duration.min(0.28);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.45 + vel * 0.35;
                sig = v.filt.process(sig, fc, q, sr, FilterMode::Lowpass);
                let peak = 0.12 * v.velocity_gain;
                let sus = 0.48 + (1.0 - vel) * 0.12;
                let att = 0.012;
                let dec = 0.18 + (1.0 - vel) * 0.08;
                let rel = 0.16;
                let env = compute_envelope(age, v.duration, peak, sus, att, dec, rel);
                // trem
                let tr = 0.975 + dmath::sin(v.trem_phase) * (0.018 + vel * 0.008);
                v.trem_phase += TAU * (4.65 + ((v.pitch as i32 % 5) as f32) * 0.07) * dt;
                sig * env * tr
            }
            VoiceType::Organ => {
                let gs = [0.42f32, 0.28, 0.16];
                let rs = [1.0f32, 2.0, 3.0];
                let mut mix = 0.0;
                let ps = [&mut v.phase1, &mut v.phase2, &mut v.phase3];
                for (i, (&r, &g)) in rs.iter().zip(gs.iter()).enumerate() {
                    let f = base * r;
                    mix += dmath::sin(*ps[i]) * g;
                    *ps[i] += TAU * f * dt;
                }
                let tr = 0.92 + dmath::sin(v.trem_phase) * 0.08;
                v.trem_phase += TAU * 5.4 * dt;
                let peak = (if is_mel { 0.09 } else { 0.034 }) * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.7, 0.03, 0.18, 0.2);
                mix * env * tr
            }
            VoiceType::Supersaw => {
                let mut mix = 0.0;
                let ps = [&mut v.phase1, &mut v.phase2, &mut v.phase3];
                for (i, &frequency_multiplier) in v.frequency_multipliers.iter().enumerate() {
                    let f = base * frequency_multiplier;
                    mix += saw_phase(*ps[i]);
                    *ps[i] += TAU * f * dt;
                }
                let mut fc = 2400.0 + vel * 900.0;
                let end_fc = 1100.0;
                let ramp_d = v.duration.min(0.22);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.4;
                let sig = v.filt.process(mix, fc, q, sr, FilterMode::Lowpass);
                let peak = (if is_mel { 0.034 } else { 0.016 }) * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.62, 0.02, 0.14, 0.16);
                sig * env
            }
            VoiceType::Triangle => {
                let s = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * base * dt;
                let peak = 0.1 * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.7, 0.004, 0.05, 0.04);
                s * env
            }
            VoiceType::Chip => {
                let is_lead = is_mel;
                let oct_g = if is_lead { 0.12 } else { 0.04 };
                let steps = if is_lead { 28 } else { 48 };
                let vib_f = if is_lead { 5.7 } else { 0.8 };
                let vib_d = if is_lead { 16.0 } else { 4.0 };
                let vib = dmath::sin(v.vib_phase) * vib_d;
                v.vib_phase += TAU * vib_f * dt;
                let f1 = base * dmath::powf(2.0, vib / 1200.0);
                let s1 = generate_osc(v.phase1, Wave::Square);
                v.phase1 += TAU * f1 * dt;
                let f2 = base * 2.0;
                let s2 = generate_osc(v.phase2, Wave::Square);
                v.phase2 += TAU * f2 * dt;
                let mut sig = s1 + s2 * oct_g;
                sig = bitcrush(sig, steps);
                let peak = (if is_lead { 0.032 } else { 0.011 }) * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.55, 0.003, 0.04, 0.03);
                sig * env
            }
            VoiceType::Kick => {
                let start_f = 162.0 + vel * 18.0;
                let end_f = 49.0;
                let tf = if age < 0.12 {
                    start_f * dmath::powf(end_f / start_f, age / 0.12)
                } else {
                    end_f
                };
                let s = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * tf * dt;
                let g0 = (0.27 * vel).max(MIN_GAIN);
                let te = if age < 0.22 {
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.22)
                } else {
                    MIN_GAIN
                };
                let tone = s * te;
                let ng = if age < 0.018 {
                    let g0 = (0.045 * vel).max(MIN_GAIN);
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.018)
                } else {
                    0.0
                };
                let n = noise(&mut v.noise_state);
                let nf = v.filt.process(n, 4800.0, 0.65, sr, FilterMode::Highpass);
                tone + nf * ng
            }
            VoiceType::Snare => {
                let start_f = 205.0f32;
                let end_f = 142.0f32;
                let tf = if age < 0.085 {
                    start_f * dmath::powf(end_f / start_f, age / 0.085_f32)
                } else {
                    end_f
                };
                let s = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * tf * dt;
                let g0 = (0.105 * vel).max(MIN_GAIN);
                let te = if age < 0.12 {
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.12)
                } else {
                    MIN_GAIN
                };
                let tone = s * te;
                let ng = if age < 0.16 {
                    let g0 = (0.205 * vel).max(MIN_GAIN);
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.16)
                } else {
                    0.0
                };
                let n = noise(&mut v.noise_state);
                let nf = v.filt.process(n, 2350.0, 0.72, sr, FilterMode::Bandpass);
                tone + nf * ng
            }
            VoiceType::Hat => {
                let ng = if age < 0.08 {
                    let g0 = (0.12 * vel).max(MIN_GAIN);
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.08)
                } else {
                    0.0
                };
                let n = noise(&mut v.noise_state);
                let nf = v.filt.process(n, 6200.0, 0.35, sr, FilterMode::Highpass);
                nf * ng
            }
            VoiceType::FrameDrum => {
                let body = generate_osc(v.phase1, Wave::Sine);
                let first_mode = generate_osc(v.phase2, Wave::Sine);
                let second_mode = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * base * dt;
                v.phase2 += TAU * base * 1.59 * dt;
                v.phase3 += TAU * base * 2.14 * dt;

                let modes = body * natural_decay(age, 0.19)
                    + first_mode * 0.38 * natural_decay(age, 0.12)
                    + second_mode * 0.2 * natural_decay(age, 0.075);
                let strike = if age < 0.034 {
                    let transient = natural_decay(age, 0.009);
                    let sample = noise(&mut v.noise_state);
                    v.filt
                        .process(sample, 1350.0 + base * 3.0, 0.72, sr, FilterMode::Bandpass)
                        * transient
                        * 0.13
                } else {
                    0.0
                };
                (modes + strike) * 0.2 * v.velocity_gain
            }
            VoiceType::Bombo => {
                let body = generate_osc(v.phase1, Wave::Sine);
                let mode1 = generate_osc(v.phase2, Wave::Sine);
                let mode2 = generate_osc(v.phase3, Wave::Sine);
                v.phase1 += TAU * base * dt;
                v.phase2 += TAU * base * 1.49 * dt;
                v.phase3 += TAU * base * 2.09 * dt;

                let modes = body * natural_decay(age, 0.26)
                    + mode1 * 0.41 * natural_decay(age, 0.16)
                    + mode2 * 0.21 * natural_decay(age, 0.085);
                let strike = if age < 0.042 {
                    let transient = natural_decay(age, 0.011);
                    let sample = noise(&mut v.noise_state);
                    v.filt
                        .process(sample, 920.0 + base * 2.8, 0.78, sr, FilterMode::Bandpass)
                        * transient
                        * 0.16
                } else {
                    0.0
                };
                let tail = ((v.life - age) / 0.04).clamp(0.0, 1.0);
                (modes + strike) * 0.26 * v.velocity_gain * tail
            }
            VoiceType::BomboRim => {
                let click = if age < 0.009 {
                    let t = age / 0.009;
                    (1.0 - t) * noise(&mut v.noise_state) * 0.65
                } else {
                    0.0
                };
                let wood = generate_osc(v.phase1, Wave::Sine);
                let upper = generate_osc(v.phase2, Wave::Sine);
                v.phase1 += TAU * base * dt;
                v.phase2 += TAU * base * 1.47 * dt;
                let woody =
                    wood * natural_decay(age, 0.022) + upper * 0.35 * natural_decay(age, 0.012);
                (click + woody * 0.55) * 0.34 * v.velocity_gain
            }
            VoiceType::Tambourine => {
                let variation = base;
                let first_gap = 0.058 + variation * 0.018;
                let second_gap = 0.142 + variation * 0.027;
                let third_gap = 0.25 + variation * 0.035;
                let envelope = rattle_burst(age, 0.0, 0.052)
                    + rattle_burst(age, first_gap, 0.047) * 0.82
                    + rattle_burst(age, second_gap, 0.056) * 0.64
                    + rattle_burst(age, third_gap, 0.07) * 0.42;
                let sample = noise(&mut v.noise_state);
                let airy = v.filt.process(
                    sample,
                    4300.0 + variation * 1500.0,
                    0.32,
                    sr,
                    FilterMode::Highpass,
                );
                airy * envelope.min(1.25) * 0.12 * v.velocity_gain
            }
            VoiceType::ReverseCymbal | VoiceType::AirImpact => {
                let reverse = matches!(v.voice_type, VoiceType::ReverseCymbal);
                let progress = (age / v.duration).clamp(0.0, 1.0);
                let peak = (vel * if reverse { 0.085 } else { 0.075 }).max(MIN_GAIN);
                let gain = if reverse && age < v.duration {
                    MIN_GAIN * dmath::powf(peak / MIN_GAIN, progress)
                } else if reverse && age < v.duration + 0.12 {
                    peak * dmath::powf(MIN_GAIN / peak, (age - v.duration) / 0.12)
                } else if !reverse && age < 0.006 {
                    MIN_GAIN + (peak - MIN_GAIN) * age / 0.006
                } else if !reverse && age < v.duration {
                    peak * dmath::powf(MIN_GAIN / peak, (age - 0.006) / (v.duration - 0.006))
                } else {
                    0.0
                };
                let frequency = if reverse {
                    1600.0 + 3200.0 * progress
                } else {
                    1200.0
                };
                let mode = if reverse {
                    FilterMode::Highpass
                } else {
                    FilterMode::Bandpass
                };
                let sample = noise(&mut v.noise_state);
                v.filt.process(sample, frequency, 0.45, sr, mode) * gain
            }
            VoiceType::Tom => {
                let bs = v.base_freq;
                let bf = if age < 0.15 {
                    bs * dmath::powf(bs * 0.58 / bs, age / 0.15)
                } else {
                    bs * 0.58
                };
                let sb = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * bf * dt;
                let os = bs * 1.63;
                let oe = bs * 0.92;
                let of = if age < 0.11 {
                    os * dmath::powf(oe / os, age / 0.11)
                } else {
                    oe
                };
                let so = generate_osc(v.phase2, Wave::Triangle);
                v.phase2 += TAU * of * dt;
                let g0 = (0.17 * vel).max(MIN_GAIN);
                let te = if age < 0.19 {
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.19)
                } else {
                    MIN_GAIN
                };
                let tone = (sb + so * 0.23) * te;
                let ng = if age < 0.026 {
                    let g0 = (0.032 * vel).max(MIN_GAIN);
                    g0 * dmath::powf(MIN_GAIN / g0, age / 0.026)
                } else {
                    0.0
                };
                let n = noise(&mut v.noise_state);
                let nf = v.filt.process(n, 1750.0, 0.8, sr, FilterMode::Bandpass);
                tone + nf * ng
            }
        }
    }

    /// Stereo sample generator. For non-twin voices returns (c, c) after calling the
    /// mono generator (so mono filter state updated exactly). For Epiano/Supersaw,
    /// duplicates the (small) twin-osc arm so we can split the detuned before/around
    /// the (shared) filter update; we also drive the mono `filt` with the sum pre
    /// so that a subsequent mono fill sees identical filter state.
    /// The original generate_voice_sample arms are never edited.
    fn generate_voice_sample_stereo(v: &mut Voice, age: f32, sr: f32, dt: f32) -> (f32, f32) {
        let vel = v.velocity;
        let is_mel = v.is_melody;
        let base = v.base_freq;
        match v.voice_type {
            VoiceType::Epiano => {
                // exact copy of osc/phase/tine pre-filter logic from the mono arm (untouched)
                let f_l = base * v.frequency_multipliers[0];
                let f_r = base * v.frequency_multipliers[1];
                let s_l = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * f_l * dt;
                let s_r = generate_osc(v.phase2, Wave::Sine);
                v.phase2 += TAU * f_r * dt;
                let tine_f = base * (2.001 + vel * 0.003) * v.frequency_multipliers[2];
                let s_t = generate_osc(v.phase3, Wave::Sine);
                v.phase3 += TAU * tine_f * dt;
                let bg = 0.62;
                let mut pre_l = s_l * bg;
                let mut pre_r = s_r * bg;
                // tine pre gain ramp (exact copy)
                let tine_peak = 0.11 + dmath::powf(vel, 1.7) * 0.38;
                let tine_dec = 0.09 + (1.0 - vel) * 0.08;
                let tine_d_t = v.duration.min(tine_dec);
                let mut tg = 0.012;
                if age < 0.004 {
                    let fr = age / 0.004;
                    let tgt = tine_peak * 0.7;
                    tg = MIN_GAIN * dmath::powf(tgt / MIN_GAIN, fr);
                } else if age < tine_d_t {
                    let fr = (age - 0.004) / (tine_d_t - 0.004).max(1e-6);
                    let tgt = tine_peak * 0.7;
                    tg = tgt * dmath::powf(0.012 / tgt, fr);
                }
                pre_l += s_t * tg;
                pre_r += s_t * tg;
                // filter coefs (exact)
                let mut fc = 1100.0 + dmath::powf(vel, 1.4) * 2200.0;
                let end_fc = 780.0 + vel * 420.0;
                let ramp_d = v.duration.min(0.28);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.45 + vel * 0.35;
                // keep mono filt state identical to what mono path would do
                let pre_sum = pre_l + pre_r;
                let _ = v.filt.process(pre_sum, fc, q, sr, FilterMode::Lowpass);
                // split filters get their own pre (for independent IIR history on sides)
                let y_l = v.filt_l.process(pre_l, fc, q, sr, FilterMode::Lowpass);
                let y_r = v.filt_r.process(pre_r, fc, q, sr, FilterMode::Lowpass);
                let peak = 0.12 * v.velocity_gain;
                let sus = 0.48 + (1.0 - vel) * 0.12;
                let att = 0.012;
                let dec = 0.18 + (1.0 - vel) * 0.08;
                let rel = 0.16;
                let env = compute_envelope(age, v.duration, peak, sus, att, dec, rel);
                let tr = 0.975 + dmath::sin(v.trem_phase) * (0.018 + vel * 0.008);
                v.trem_phase += TAU * (4.65 + ((v.pitch as i32 % 5) as f32) * 0.07) * dt;
                (y_l * env * tr, y_r * env * tr)
            }
            VoiceType::Supersaw => {
                let s0 = saw_phase(v.phase1);
                let f0 = base * v.frequency_multipliers[0];
                v.phase1 += TAU * f0 * dt;
                let s1 = saw_phase(v.phase2);
                let f1 = base * v.frequency_multipliers[1];
                v.phase2 += TAU * f1 * dt;
                let s2 = saw_phase(v.phase3);
                let f2 = base * v.frequency_multipliers[2];
                v.phase3 += TAU * f2 * dt;
                let bg = 1.0; // the mix in mono is just sum of saws
                let pre_l = (s0 + s1 * 0.5) * bg;
                let pre_r = (s2 + s1 * 0.5) * bg;
                let pre_sum = (s0 + s1 + s2) * bg;
                let mut fc = 2400.0 + vel * 900.0;
                let end_fc = 1100.0;
                let ramp_d = v.duration.min(0.22);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= dmath::powf(end_fc / fc, fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.4;
                let _ = v.filt.process(pre_sum, fc, q, sr, FilterMode::Lowpass);
                let y_l = v.filt_l.process(pre_l, fc, q, sr, FilterMode::Lowpass);
                let y_r = v.filt_r.process(pre_r, fc, q, sr, FilterMode::Lowpass);
                let peak = (if is_mel { 0.034 } else { 0.016 }) * v.velocity_gain;
                let env = compute_envelope(age, v.duration, peak, 0.62, 0.02, 0.14, 0.16);
                (y_l * env, y_r * env)
            }
            _ => {
                let c = Synth::generate_voice_sample(v, age, sr, dt);
                (c, c)
            }
        }
    }
}

pub fn events_starting_at(
    score: &PortableScore,
    section_id: &str,
    local_tick: u32,
    window: u32,
) -> Vec<MusicEvent> {
    score
        .section(section_id)
        .map(|section| {
            section
                .events
                .iter()
                .filter(|event| {
                    let start = event.start_tick();
                    start >= local_tick && start < local_tick + window
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Score time (in ticks) at a produced-sample position, as a float so callers
/// can schedule events inside the current buffer. Deriving the score clock a
/// pure function of produced samples is what keeps it locked to the audio
/// clock instead of accumulating rounding drift per call.
pub fn score_tick_at_sample(frames_produced: u64, sample_rate: f64, ticks_per_second: f64) -> f64 {
    if sample_rate <= 0.0 {
        return 0.0;
    }
    frames_produced as f64 / sample_rate * ticks_per_second
}

/// Integer form of [`score_tick_at_sample`].
pub fn tick_at_sample(frames_produced: u64, sample_rate: f64, ticks_per_second: f64) -> u32 {
    score_tick_at_sample(frames_produced, sample_rate, ticks_per_second) as u32
}

/// The first produced-sample position at which [`tick_at_sample`] reaches `tick`.
pub fn sample_at_tick(tick: u32, sample_rate: f64, ticks_per_second: f64) -> u64 {
    if ticks_per_second <= 0.0 {
        return 0;
    }
    (f64::from(tick) / ticks_per_second * sample_rate).ceil() as u64
}

fn midi_to_freq(pitch: u8) -> f32 {
    440.0 * dmath::powf(2.0, (pitch as f32 - 69.0) / 12.0)
}

fn velocity_curve(v: f32, e: f32) -> f32 {
    dmath::powf(v.max(0.02), e)
}

fn natural_decay(age: f32, time_constant: f32) -> f32 {
    dmath::exp(-age / time_constant.max(0.001))
}

fn harp_life(frequency: f32) -> f32 {
    let lower_note = (220.0 / frequency).clamp(0.25, 1.0);
    2.8 + lower_note * 1.25
}

fn bell_life(frequency: f32) -> f32 {
    let lower_note = (330.0 / frequency).clamp(0.35, 1.0);
    3.4 + lower_note * 1.4
}

fn rattle_burst(age: f32, start: f32, duration: f32) -> f32 {
    if age < start || age >= start + duration {
        return 0.0;
    }
    let progress = (age - start) / duration;
    let attack = (progress / 0.12).clamp(0.0, 1.0);
    attack * natural_decay(progress, 0.42)
}

fn clamp_f(v: f32, lo: f32, hi: f32) -> f32 {
    v.max(lo).min(hi)
}

fn compute_freq(base: f32, age: f32, pitch_drop: f32) -> f32 {
    if pitch_drop <= 0.0 {
        base
    } else {
        let start_f = base * (1.0 + pitch_drop);
        let ramp = 0.022;
        if age < ramp {
            let fr = age / ramp;
            start_f * dmath::powf(base / start_f, fr)
        } else {
            base
        }
    }
}

fn compute_envelope(
    age: f32,
    duration: f32,
    peak: f32,
    sustain: f32,
    attack: f32,
    decay: f32,
    release: f32,
) -> f32 {
    compute_envelope_with_cap(age, duration, peak, sustain, attack, decay, release, 0.24)
}

#[allow(clippy::too_many_arguments)]
fn compute_envelope_with_cap(
    age: f32,
    duration: f32,
    peak: f32,
    sustain: f32,
    attack: f32,
    decay: f32,
    release: f32,
    cap: f32,
) -> f32 {
    let attack_d = clamp_f(attack, 0.001, duration * 0.24);
    let decay_d = clamp_f(decay, 0.001, duration * 0.46);
    let attack_end = attack_d;
    let decay_end = (duration * 0.7).min(attack_end + decay_d);
    let release_t = clamp_f(release, 0.02, cap);
    let safe_p = peak.max(MIN_GAIN);
    let sus_g = (safe_p * sustain).max(MIN_GAIN);
    let t = age;
    if t <= 0.0 {
        MIN_GAIN
    } else if t < attack_end {
        let fr = t / attack_end.max(1e-6);
        MIN_GAIN * dmath::powf(safe_p / MIN_GAIN, fr)
    } else if t < decay_end {
        let fr = (t - attack_end) / (decay_end - attack_end).max(1e-6);
        safe_p * dmath::powf(sus_g / safe_p, fr)
    } else if t < duration {
        sus_g
    } else {
        let rel_end = duration + release_t;
        if t < rel_end {
            let fr = (t - duration) / release_t.max(1e-6);
            sus_g * dmath::powf(MIN_GAIN / sus_g, fr)
        } else {
            MIN_GAIN
        }
    }
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Saw,
    Triangle,
    Square,
}

fn generate_osc(phase: f32, wave: Wave) -> f32 {
    match wave {
        Wave::Sine => dmath::sin(phase),
        Wave::Saw => saw_phase(phase),
        Wave::Triangle => {
            let x = 2.0 * (phase / TAU).fract() - 1.0;
            1.0 - 2.0 * x.abs()
        }
        Wave::Square => {
            if (phase / TAU).fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

fn saw_phase(phase: f32) -> f32 {
    2.0 * (phase / TAU).fract() - 1.0
}

fn bitcrush(x: f32, steps: u32) -> f32 {
    let q = steps.max(2) as f32;
    let xc = x.clamp(-1.0, 1.0);
    (xc * q).round() / q
}

fn noise(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn deterministic_unit(seed: &str) -> f32 {
    let mut hash: u32 = 0x811c9dc5;
    for b in seed.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    (hash as f32) / (u32::MAX as f32)
}

fn deterministic_noise_state(seed: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for b in seed.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    if hash == 0 {
        0x1234_5678
    } else {
        hash
    }
}

use crate::score::PortableScore;

#[cfg(test)]
mod tests {
    use super::{Biquad, FilterMode, Solo, Synth};
    use crate::racing::{generate_racing, GenerateInput, InstrumentPalette, Style};
    use crate::score::MusicEvent;
    use std::time::{Duration, Instant};

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    }

    fn difference_ratio(samples: &[f32]) -> f32 {
        let signal = samples.iter().map(|sample| sample * sample).sum::<f32>();
        let differences = samples
            .windows(2)
            .map(|pair| {
                let difference = pair[1] - pair[0];
                difference * difference
            })
            .sum::<f32>();
        differences / signal.max(f32::EPSILON)
    }

    fn render_note(voice: &str, duration_ticks: u32, sample_count: usize) -> Vec<f32> {
        let mut synth = Synth::new(48000.0);
        synth.trigger(
            &MusicEvent::Note {
                id: format!("{voice}-test"),
                section: "journey".into(),
                lane: "melody".into(),
                start_tick: 0,
                duration_ticks,
                velocity: 0.6,
                pitch: 67,
                voice: voice.into(),
                role: Some("melody".into()),
            },
            960.0,
        );
        let mut samples = vec![0.0; sample_count];
        synth.fill(&mut samples);
        samples
    }

    fn render_percussion(voice: &str, id: &str) -> Vec<f32> {
        let mut synth = Synth::new(48000.0);
        synth.trigger(
            &MusicEvent::Percussion {
                id: id.into(),
                section: "combat".into(),
                lane: "percussion".into(),
                start_tick: 0,
                duration_ticks: 240,
                velocity: 0.65,
                voice: voice.into(),
            },
            960.0,
        );
        let mut samples = vec![0.0; 33600];
        synth.fill(&mut samples);
        assert!(synth.voices.is_empty(), "{voice} must clean up its voice");
        samples
    }

    fn benchmark_voice(voice: &str, percussion: bool) -> (Duration, usize) {
        const SAMPLE_RATE: f32 = 48_000.0;
        const PARALLEL_VOICES: usize = 8;
        let mut synth = Synth::new(SAMPLE_RATE);
        let event = if percussion {
            MusicEvent::Percussion {
                id: format!("{voice}-benchmark"),
                section: "benchmark".into(),
                lane: "benchmark".into(),
                start_tick: 0,
                duration_ticks: 960,
                velocity: 0.65,
                voice: voice.into(),
            }
        } else {
            MusicEvent::Note {
                id: format!("{voice}-benchmark"),
                section: "benchmark".into(),
                lane: "benchmark".into(),
                start_tick: 0,
                duration_ticks: 3840,
                velocity: 0.65,
                pitch: 67,
                voice: voice.into(),
                role: Some("melody".into()),
            }
        };
        synth.trigger(&event, 960.0);
        let prototype = synth.voices.pop().expect("benchmark voice must exist");
        let samples = if percussion {
            (prototype.life * SAMPLE_RATE) as usize
        } else {
            SAMPLE_RATE as usize
        };
        let dt = 1.0 / SAMPLE_RATE;
        let mut timings = (0..3)
            .map(|_| {
                let mut voices = vec![prototype.clone(); PARALLEL_VOICES];
                let mut checksum = 0.0;
                let started = Instant::now();
                for sample in 0..samples {
                    let age = sample as f32 * dt;
                    for active_voice in &mut voices {
                        checksum +=
                            Synth::generate_voice_sample(active_voice, age, SAMPLE_RATE, dt);
                    }
                }
                std::hint::black_box(checksum);
                started.elapsed()
            })
            .collect::<Vec<_>>();
        timings.sort_unstable();
        (timings[1], samples * PARALLEL_VOICES)
    }

    #[test]
    #[ignore = "release-only voice timing benchmark; run explicitly with --ignored --nocapture"]
    #[allow(clippy::assertions_on_constants)]
    fn voice_render_benchmark() {
        assert!(
            !cfg!(debug_assertions),
            "run this timing benchmark with cargo test --release"
        );
        let notes = [
            "warm",
            "glass",
            "pulse",
            "pluck",
            "felt",
            "dusk",
            "harp",
            "recorder",
            "vielle",
            "nylon-guitar",
            "bell",
            "bass",
            "epiano",
            "organ",
            "supersaw",
            "triangle",
            "chip",
        ];
        let percussion = [
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
        println!("median nanoseconds per generated voice sample (8 voices, 3 runs)");
        for (voice, is_percussion) in notes
            .into_iter()
            .map(|voice| (voice, false))
            .chain(percussion.into_iter().map(|voice| (voice, true)))
        {
            let (elapsed, voice_samples) = benchmark_voice(voice, is_percussion);
            let nanoseconds = elapsed.as_nanos() as f64 / voice_samples as f64;
            println!("{voice}: {nanoseconds:.2} ns / voice sample");
        }
    }

    #[test]
    fn cached_filter_coefficients_preserve_samples() {
        let mut cached = Biquad::new();
        let mut recalculated = Biquad::new();
        for index in 0..4096 {
            let input = ((index as f32 * 0.173).sin() * 0.7).clamp(-1.0, 1.0);
            let cached_sample = cached.process(input, 2350.0, 0.72, 48000.0, FilterMode::Bandpass);
            recalculated.coefficient_key = None;
            let recalculated_sample =
                recalculated.process(input, 2350.0, 0.72, 48000.0, FilterMode::Bandpass);
            assert_eq!(cached_sample.to_bits(), recalculated_sample.to_bits());
        }
    }

    #[test]
    fn pluck_render_is_bit_exact() {
        let hash = render_note("pluck", 960, 48000)
            .into_iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, sample| {
                (hash ^ u64::from(sample.to_bits())).wrapping_mul(0x0000_0100_0000_01b3)
            });
        assert_eq!(hash, 15_142_955_883_572_678_187);
    }

    #[test]
    fn ambient_voices_render_soft_tails_and_do_not_start_the_echo_early() {
        for voice in ["felt", "dusk"] {
            let mut synth = Synth::new(8000.0);
            synth.trigger(
                &MusicEvent::Note {
                    id: "ambient".into(),
                    section: "scan".into(),
                    lane: "atmosphere".into(),
                    start_tick: 0,
                    duration_ticks: 960,
                    velocity: 0.2,
                    pitch: 60,
                    voice: voice.into(),
                    role: Some("melody".into()),
                },
                960.0,
            );
            assert_eq!(synth.voices.len(), 2);
            let mut opening = vec![0.0; 800];
            synth.fill(&mut opening);
            let delayed = synth
                .voices
                .iter()
                .find(|note| note.start_phase > 0.0)
                .unwrap();
            assert_eq!(delayed.phase1, 0.0);
            let mut tail = vec![0.0; 24000];
            synth.fill(&mut tail);
            assert!(tail.iter().any(|sample| sample.abs() > 0.001));
            assert!(tail
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() < 0.1));
            assert!(synth.voices.is_empty(), "ambient tails must be cleaned up");
        }
    }

    #[test]
    fn procedural_noise_effects_have_opposite_envelopes_and_clean_up() {
        for voice in ["reverse-cymbal", "air-impact"] {
            let mut synth = Synth::new(48000.0);
            synth.trigger(
                &MusicEvent::Percussion {
                    id: "effect-test".into(),
                    section: "scan".into(),
                    lane: "effects".into(),
                    start_tick: 0,
                    duration_ticks: 960,
                    velocity: 0.2,
                    voice: voice.into(),
                },
                960.0,
            );
            let mut samples = vec![0.0; 96000];
            synth.fill(&mut samples);
            let early = rms(&samples[2100..10800]);
            let late = rms(&samples[37000..45700]);
            if voice == "reverse-cymbal" {
                assert!(late > early * 4.0);
            } else {
                assert!(early > late * 4.0);
            }
            assert!(samples
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() < 0.1));
            assert!(synth.voices.is_empty());
        }
    }

    #[test]
    fn acoustic_voices_render_audible_samples_with_finite_output() {
        for voice in ["harp", "recorder", "vielle", "nylon-guitar", "bell"] {
            let buffer = render_note(voice, 960, 48000);
            let energy: f32 = buffer.iter().map(|sample| sample.abs()).sum();
            let peak = buffer.iter().map(|sample| sample.abs()).fold(0.0, f32::max);
            assert!(energy > 1.0, "{voice} should be audible, got {energy}");
            assert!(
                buffer
                    .iter()
                    .all(|sample| sample.is_finite() && sample.abs() < 0.95),
                "{voice} must stay finite and in range"
            );
            assert!(
                peak < 0.4,
                "{voice} should retain moderate headroom, got {peak}"
            );
        }
        for voice in ["frame-drum", "tambourine", "bombo", "bombo-rim"] {
            let buffer = render_percussion(voice, "adventure-percussion");
            let energy: f32 = buffer.iter().map(|sample| sample.abs()).sum();
            let peak = buffer.iter().map(|sample| sample.abs()).fold(0.0, f32::max);
            assert!(energy > 1.0, "{voice} should be audible, got {energy}");
            assert!(
                buffer
                    .iter()
                    .all(|sample| sample.is_finite() && sample.abs() < 0.95),
                "{voice} must stay finite and in range"
            );
            assert!(
                peak < 0.4,
                "{voice} should retain moderate headroom, got {peak}"
            );
        }
    }

    #[test]
    fn folklore_voices_preserve_soft_hits_and_silence() {
        for voice in ["nylon-guitar", "bombo", "bombo-rim"] {
            let render = |velocity: f64| {
                let event = if voice == "nylon-guitar" {
                    MusicEvent::Note {
                        id: "dynamic-note".into(),
                        section: "test".into(),
                        lane: "guitar".into(),
                        start_tick: 0,
                        duration_ticks: 240,
                        velocity,
                        pitch: 64,
                        voice: voice.into(),
                        role: Some("melody".into()),
                    }
                } else {
                    MusicEvent::Percussion {
                        id: "dynamic-hit".into(),
                        section: "test".into(),
                        lane: "bombo".into(),
                        start_tick: 0,
                        duration_ticks: 240,
                        velocity,
                        voice: voice.into(),
                    }
                };
                let mut synth = Synth::new(48000.0);
                synth.trigger(&event, 960.0);
                let mut samples = vec![0.0; 48000];
                synth.fill(&mut samples);
                assert!(synth.voices.is_empty(), "{voice} must finish its release");
                samples
            };
            let soft = render(0.2);
            let loud = render(0.8);
            assert!(
                rms(&loud) > rms(&soft) * 2.0,
                "{voice} must respond to velocity"
            );
            assert!(
                render(0.0).iter().all(|sample| *sample == 0.0),
                "{voice} must respect zero velocity"
            );
            assert_eq!(soft, render(0.2), "{voice} must render deterministically");
        }
        let short_note = render_note("nylon-guitar", 120, 48000);
        assert!(short_note[16800..].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn folk_and_mallet_voices_are_finite_distinct_and_deterministic() {
        for voice in ["charango", "quena", "marimba"] {
            let buffer = render_note(voice, 960, 48000);
            let energy: f32 = buffer.iter().map(|sample| sample.abs()).sum();
            let peak = buffer.iter().map(|sample| sample.abs()).fold(0.0, f32::max);
            assert!(energy > 1.0, "{voice} should be audible, got {energy}");
            assert!(
                buffer
                    .iter()
                    .all(|sample| sample.is_finite() && sample.abs() < 0.95),
                "{voice} must stay finite and in range"
            );
            assert!(
                peak < 0.4,
                "{voice} should retain moderate headroom, got {peak}"
            );
            assert_eq!(
                buffer,
                render_note(voice, 960, 48000),
                "{voice} must render deterministically"
            );
        }
        assert_ne!(
            render_note("charango", 960, 48000),
            render_note("nylon-guitar", 960, 48000)
        );
        assert_ne!(
            render_note("quena", 960, 48000),
            render_note("recorder", 960, 48000)
        );
        assert_ne!(
            render_note("marimba", 960, 48000),
            render_note("harp", 960, 48000)
        );
        assert_ne!(
            render_note("charango", 960, 48000),
            render_note("quena", 960, 48000)
        );
        assert_ne!(
            render_note("quena", 960, 48000),
            render_note("marimba", 960, 48000)
        );
    }

    #[test]
    fn folk_and_mallet_voices_end_cleanly() {
        let mut synth = Synth::new(48000.0);
        for voice in ["charango", "quena", "marimba"] {
            synth.trigger(
                &MusicEvent::Note {
                    id: format!("{voice}-cleanup"),
                    section: "journey".into(),
                    lane: "melody".into(),
                    start_tick: 0,
                    duration_ticks: 240,
                    velocity: 0.5,
                    pitch: 64,
                    voice: voice.into(),
                    role: Some("melody".into()),
                },
                960.0,
            );
        }
        let mut buffer = vec![0.0f32; 96000];
        synth.fill(&mut buffer);
        assert!(buffer.iter().all(|sample| sample.is_finite()));
        assert!(synth.voices.is_empty(), "folk voices must clean up");
    }

    #[test]
    fn plucked_folk_and_mallet_voices_fade_before_cleanup() {
        for voice in ["charango", "marimba"] {
            for duration in [120, 960] {
                let buffer = render_note(voice, duration, 96000);
                let last = buffer
                    .iter()
                    .rposition(|sample| sample.abs() > f32::EPSILON)
                    .expect("note must be audible");
                assert!(last + 1 < buffer.len(), "{voice} must finish");
                assert!(
                    buffer[last].abs() < 0.001,
                    "{voice} {duration}: abrupt cutoff at {}",
                    buffer[last]
                );
            }
        }
    }

    #[test]
    fn harp_and_bell_decay_naturally_while_recorder_sustains() {
        let harp = render_note("harp", 3840, 144000);
        let recorder = render_note("recorder", 3840, 144000);
        let bell = render_note("bell", 3840, 144000);
        let early_range = 4800..14400;
        let late_range = 120000..129600;

        let harp_early = rms(&harp[early_range.clone()]);
        let bell_early = rms(&bell[early_range.clone()]);
        let harp_late = rms(&harp[late_range.clone()]);
        let bell_late = rms(&bell[late_range.clone()]);
        let recorder_late = rms(&recorder[late_range]);
        assert!(
            harp_late < harp_early * 0.12,
            "harp must not hold long chords"
        );
        assert!(
            bell_late < bell_early * 0.22,
            "bell modes must decay naturally"
        );
        assert!(
            recorder_late > harp_late * 5.0,
            "recorder should retain breath-supported sustain"
        );
    }

    #[test]
    fn acoustic_percussion_is_deterministic_and_spectrally_distinct() {
        let recorder = render_note("recorder", 960, 48000);
        let recorder_repeat = render_note("recorder", 960, 48000);
        let frame_drum = render_percussion("frame-drum", "percussion-seed");
        let frame_drum_repeat = render_percussion("frame-drum", "percussion-seed");
        let tambourine = render_percussion("tambourine", "percussion-seed");
        let tambourine_repeat = render_percussion("tambourine", "percussion-seed");
        let tambourine_variant = render_percussion("tambourine", "other-seed");

        assert_eq!(recorder, recorder_repeat);
        assert_eq!(frame_drum, frame_drum_repeat);
        assert_eq!(tambourine, tambourine_repeat);
        assert_ne!(tambourine, tambourine_variant);
        assert!(
            difference_ratio(&tambourine) > difference_ratio(&frame_drum) * 2.5,
            "tambourine rattles should carry more high-frequency energy than the drum body"
        );

        let opening = rms(&tambourine[..2205]);
        let tail = rms(&tambourine[8820..11025]);
        assert!(
            opening > tail * 3.0,
            "tambourine rattles must decay rather than loop"
        );
    }

    #[test]
    fn acoustic_voice_lifetimes_end_without_stuck_notes() {
        let mut synth = Synth::new(8000.0);
        for voice in ["harp", "recorder", "vielle", "bell"] {
            synth.trigger(
                &MusicEvent::Note {
                    id: format!("{voice}-cleanup"),
                    section: "journey".into(),
                    lane: "melody".into(),
                    start_tick: 0,
                    duration_ticks: 240,
                    velocity: 0.5,
                    pitch: 64,
                    voice: voice.into(),
                    role: Some("melody".into()),
                },
                960.0,
            );
        }
        let mut buffer = vec![0.0; 48000];
        synth.fill(&mut buffer);
        assert!(buffer.iter().all(|sample| sample.is_finite()));
        assert!(synth.voices.is_empty());
    }

    #[test]
    fn cruise_events_render_audible_samples() {
        let score = generate_racing(&GenerateInput {
            secret: "qa-secret".into(),
            seed: "qa-race".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.7,
            complexity: 0.6,
            brightness: 0.5,
            syncopation: 0.7,
        })
        .expect("synth test score must validate");
        let ticks_per_second = score.ticks_per_second();
        let mut synth = Synth::new(48000.0);
        for event in &score.section("cruise").unwrap().events {
            if event.start_tick() < 960 {
                synth.trigger(event, ticks_per_second);
            }
        }
        let mut buffer = vec![0.0_f32; 2205];
        synth.fill(&mut buffer);
        let energy: f32 = buffer.iter().map(|sample| sample.abs()).sum();
        assert!(energy > 1.0, "expected audible energy, got {energy}");
    }

    #[test]
    fn trigger_at_starts_the_voice_at_the_requested_offset() {
        let mut synth = Synth::new(48000.0);
        synth.trigger_at(
            &MusicEvent::Note {
                id: "offset-test".into(),
                section: "journey".into(),
                lane: "melody".into(),
                start_tick: 0,
                duration_ticks: 240,
                velocity: 0.6,
                pitch: 67,
                voice: "pluck".into(),
                role: Some("melody".into()),
            },
            2160.0,
            0.25,
        );
        assert_eq!(synth.voices.len(), 1, "one note should schedule one voice");
        assert!(
            (synth.voices[0].start_phase - 0.25).abs() < 1e-6,
            "voice must start at the requested offset, got {}",
            synth.voices[0].start_phase
        );
    }

    #[test]
    fn score_tick_at_sample_does_not_drift_over_a_long_run() {
        // Simulate ~9 minutes of variable-sized buffers at 48 kHz / 2160 tps.
        // The old `tick += ceil(span) + 1` scheme drifts tens of seconds; a
        // sample-derived tick must stay within one tick of the true score time.
        let sample_rate = 48000.0_f64;
        let ticks_per_second = 2160.0_f64;
        let mut produced = 0_u64;
        for buffer in 0..(60 * 60 * 9) {
            let frames = 800 + (buffer % 400);
            produced += frames as u64;
            let tick = f64::from(super::tick_at_sample(
                produced,
                sample_rate,
                ticks_per_second,
            ));
            let truth = produced as f64 / sample_rate * ticks_per_second;
            assert!(
                (tick - truth).abs() < 1.0,
                "sample-derived tick drifted from score time: {tick} vs {truth}"
            );
        }
        assert!(produced > 48_000 * 60 * 5, "simulation should span minutes");
    }

    /// A melody lead, an accompanying bed and a hat.
    fn solo_events() -> Vec<MusicEvent> {
        let note = |id: &str, voice: &str, role: Option<&str>| MusicEvent::Note {
            id: id.into(),
            section: "solo".into(),
            lane: "solo".into(),
            start_tick: 0,
            duration_ticks: 3840,
            velocity: 0.6,
            pitch: 64,
            voice: voice.into(),
            role: role.map(Into::into),
        };
        vec![
            note("lead", "glass", Some("melody")),
            note("bed", "warm", None),
            MusicEvent::Percussion {
                id: "hat".into(),
                section: "solo".into(),
                lane: "solo".into(),
                start_tick: 0,
                duration_ticks: 120,
                velocity: 0.6,
                voice: "hat".into(),
            },
        ]
    }

    fn render_solo(events: &[MusicEvent], solo: &Solo) -> Vec<f32> {
        let mut synth = Synth::new(48000.0);
        synth.set_solo(solo);
        for event in events {
            synth.trigger(event, 960.0);
        }
        let mut samples = vec![0.0; 4800];
        synth.fill(&mut samples);
        samples
    }

    fn only(ids: &[&str]) -> Vec<MusicEvent> {
        solo_events()
            .into_iter()
            .filter(|event| match event {
                MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => {
                    ids.contains(&id.as_str())
                }
            })
            .collect()
    }

    #[test]
    fn a_solo_lets_through_exactly_the_voices_it_names() {
        let all = solo_events();
        let voice = |voice: &str, mute| Solo::Voice {
            voice: voice.into(),
            mute,
        };
        for (solo, ids) in [
            (Solo::Melody, vec!["lead"]),
            (Solo::Rhythm, vec!["bed", "hat"]),
            (voice("warm", false), vec!["bed"]),
            (voice("warm", true), vec!["lead", "hat"]),
            (voice("hat", false), vec!["hat"]),
        ] {
            assert_eq!(
                render_solo(&all, &solo),
                render_solo(&only(&ids), &Solo::Full),
                "{solo:?}"
            );
        }
    }

    #[test]
    fn changing_the_solo_fades_the_other_voices_out() {
        let mut synth = Synth::new(48000.0);
        let mut lead_only = Synth::new(48000.0);
        for event in solo_events() {
            synth.trigger(&event, 960.0);
        }
        for event in only(&["lead"]) {
            lead_only.trigger(&event, 960.0);
        }
        let mut before = vec![0.0; 2400];
        synth.fill(&mut before);
        lead_only.fill(&mut before.clone());
        synth.set_solo(&Solo::Melody);
        let mut after = vec![0.0; 2400];
        let mut lead = vec![0.0; 2400];
        synth.fill(&mut after);
        lead_only.fill(&mut lead);
        let fade = (super::SOLO_FADE_SECONDS * 48000.0).ceil() as usize + 1;
        assert_ne!(
            after[8], lead[8],
            "the bed fades rather than stopping at once"
        );
        assert_eq!(
            after[fade..],
            lead[fade..],
            "after the fade only the melody sounds"
        );
    }
}
