use gamestruments_engine::{
    generate_pocket_circuit, AdaptiveTransport, GameState, GenerateInput, InstrumentPalette,
    PortableScore, Style, Synth,
};
use godot::classes::{AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayer};
use godot::prelude::*;

struct GamestrumentsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GamestrumentsExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct GamestrumentsPlayer {
    /// Per-title secret. Do not use a public name like "pocket-circuit".
    #[export]
    project_secret: GString,
    #[export]
    style: GString,
    #[export]
    melody_voice: GString,
    #[export]
    harmony_voice: GString,
    #[export]
    drive_voice: GString,
    #[export]
    bass_voice: GString,
    #[export]
    energy: f64,
    #[export]
    complexity: f64,
    #[export]
    brightness: f64,
    #[export]
    syncopation: f64,
    score: Option<PortableScore>,
    transport: Option<AdaptiveTransport>,
    synth: Synth,
    ticks_per_second: f64,
    tick: u32,
    sample_rate: f32,
    player: Option<Gd<AudioStreamPlayer>>,
    playback: Option<Gd<AudioStreamGeneratorPlayback>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for GamestrumentsPlayer {
    fn init(base: Base<Node>) -> Self {
        Self {
            project_secret: GString::new(),
            style: "funk".into(),
            melody_voice: GString::new(),
            harmony_voice: GString::new(),
            drive_voice: GString::new(),
            bass_voice: GString::new(),
            energy: 0.62,
            complexity: 0.6,
            brightness: 0.52,
            syncopation: 0.7,
            score: None,
            transport: None,
            synth: Synth::new(22050.0),
            ticks_per_second: 2160.0,
            tick: 0,
            sample_rate: 22050.0,
            player: None,
            playback: None,
            base,
        }
    }

    fn ready(&mut self) {
        let mut generator = AudioStreamGenerator::new_gd();
        generator.set_mix_rate(self.sample_rate);
        generator.set_buffer_length(0.1);
        let mut player = AudioStreamPlayer::new_alloc();
        player.set_name("LiveStream");
        player.set_stream(&generator);
        player.set_bus("Music");
        self.to_gd().add_child(&player);
        player.play();
        self.playback = player
            .get_stream_playback()
            .and_then(|playback| playback.try_cast::<AudioStreamGeneratorPlayback>().ok());
        self.player = Some(player);
    }

    fn process(&mut self, _delta: f64) {
        let Some(score) = self.score.as_ref() else {
            return;
        };
        let Some(playback) = self.playback.as_mut() else {
            return;
        };
        let frames = playback.get_frames_available();
        if frames <= 0 {
            return;
        }
        let mut buffer = vec![0.0_f32; frames as usize];
        let window_ticks = ((frames as f64 / f64::from(self.sample_rate)) * self.ticks_per_second)
            .ceil() as u32
            + 1;
        if let Some(transport) = self.transport.as_mut() {
            transport.advance(self.tick);
            let section = transport.current_section().to_string();
            let length = score.section(&section).map(|s| s.length_ticks).unwrap_or(1);
            let local = self.tick % length.max(1);
            for event in gamestruments_engine::synth::events_starting_at(
                score,
                &section,
                local,
                window_ticks.max(1),
            ) {
                self.synth.trigger(&event, self.ticks_per_second);
            }
        }
        self.synth.fill(&mut buffer);
        for sample in buffer {
            playback.push_frame(Vector2::new(sample, sample));
        }
        self.tick = self.tick.wrapping_add(window_ticks.max(1));
    }
}

#[godot_api]
impl GamestrumentsPlayer {
    #[func]
    fn generate(&mut self, seed: GString) {
        let Ok(style) = Style::parse(&self.style.to_string()) else {
            godot_error!("Unknown Gamestruments style");
            return;
        };
        if self.project_secret.is_empty() {
            godot_error!("GamestrumentsPlayer.project_secret is empty");
            return;
        }
        let score = generate_pocket_circuit(&GenerateInput {
            secret: self.project_secret.to_string(),
            seed: seed.to_string(),
            style,
            palette: InstrumentPalette {
                melody: self.melody_voice.to_string(),
                harmony: self.harmony_voice.to_string(),
                drive: self.drive_voice.to_string(),
                bass: self.bass_voice.to_string(),
            },
            energy: self.energy,
            complexity: self.complexity,
            brightness: self.brightness,
            syncopation: self.syncopation,
        });
        self.ticks_per_second = score.ticks_per_second();
        self.tick = 0;
        self.synth = Synth::new(self.sample_rate);
        match AdaptiveTransport::new(score.clone(), Some("garage")) {
            Ok(transport) => {
                self.transport = Some(transport);
                self.score = Some(score);
            }
            Err(error) => godot_error!("{error}"),
        }
    }

    #[func]
    fn set_race_state(&mut self, phase: GString, intensity: f64, pressure: f64, final_lap: bool) {
        let Some(transport) = self.transport.as_mut() else {
            return;
        };
        transport.request_state(
            &GameState {
                intensity,
                position_pressure: pressure,
                final_lap,
                race_phase: phase.to_string(),
                finish_result: "none".into(),
            },
            self.tick,
        );
    }
}
