use gamestruments_engine::{
    generate_pocket_circuit, AdaptiveTransport, GameState, GenerateInput, InstrumentPalette,
    PortableScore, Style, Synth,
};
use godot::classes::{
    notify::NodeNotification, AudioStream, AudioStreamGenerator, AudioStreamGeneratorPlayback,
    AudioStreamPlayer,
};
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
        self.base_mut().add_child(&player);
        player.play();
        // Drop local Gd immediately. The child is now owned by the scene tree.
        // We will look it up by name ("LiveStream") on demand in process/exit_tree
        // using temporary Gd handles only. Never storing Gd<Audio...> in the
        // struct eliminates the ObjectDB leaks of playbacks.
    }

    fn exit_tree(&mut self) {
        // Look up child using only a temporary Gd (via to_gd which is cheap
        // ref). Stop + null its stream so Godot releases the generator and
        // playback RefCounteds, then temps drop. Never storing Gd<Audio*>
        // in the struct avoids keeping refs alive across exit_tree.
        self.cleanup_audio_child();
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {
            self.cleanup_audio_child();
        }
    }

    fn process(&mut self, _delta: f64) {
        let Some(score) = self.score.as_ref() else {
            return;
        };
        // Look up the stream player child by name and obtain its current
        // playback via get_stream_playback() on every process tick. Using
        // only short-lived temporary Gd<> (never stored in struct) ensures
        // we do not keep AudioStreamGeneratorPlayback refs alive past
        // exit_tree / free, eliminating our contribution to ObjectDB leaks.
        let this = self.base();
        let player = match this
            .get_node_or_null("LiveStream")
            .and_then(|n| n.try_cast::<AudioStreamPlayer>().ok())
        {
            Some(p) => p,
            None => return,
        };
        let mut playback = match player
            .get_stream_playback()
            .and_then(|p| p.try_cast::<AudioStreamGeneratorPlayback>().ok())
        {
            Some(pb) => pb,
            None => return,
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

// Non-exposed helper for robust shutdown cleanup. Called from exit_tree and
// on_notification to ensure the AudioStreamPlayer child (and its
// AudioStreamGenerator / Playback) have their resources released.
impl GamestrumentsPlayer {
    fn cleanup_audio_child(&mut self) {
        let this = self.base();
        if let Some(mut p) = this
            .get_node_or_null("LiveStream")
            .and_then(|n| n.try_cast::<AudioStreamPlayer>().ok())
        {
            p.stop();
            p.set_stream(Gd::<AudioStream>::null_arg());
            // Explicitly detach + free the child AudioStreamPlayer after
            // clearing its stream. This forces release of any internal
            // AudioStreamGeneratorPlayback (RefCounted) that Godot may be
            // holding with refcount 1. Using transient lookup (no stored Gd)
            // + explicit free after stop/null + on_notification (0.5 NodeNotification)
            // is the 0.5 port to match noop baseline.
            if let Some(mut parent) = p.get_parent() {
                parent.remove_child(&p);
            }
            p.free();
        }
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
    fn set_race_state(
        &mut self,
        phase: GString,
        intensity: f64,
        pressure: f64,
        final_lap: bool,
        #[opt(default = "none")]
        finish_result: GString,
    ) {
        let Some(transport) = self.transport.as_mut() else {
            return;
        };
        transport.request_state(
            &GameState {
                intensity,
                position_pressure: pressure,
                final_lap,
                race_phase: phase.to_string(),
                finish_result: if finish_result.is_empty() {
                    "none".into()
                } else {
                    finish_result.to_string()
                },
            },
            self.tick,
        );
    }
}

// Rust Drop: attempt to participate in cleanup (per investigation request).
// In gdext 0.4.5 / Godot 4.7, Drop runs when the Rust data for the instance
// is dropped (after Godot notifications like PREDELETE). We do NOT call
// to_gd().free() or godot methods here, as the Base/Gd handle is typically
// already invalidated or the object is mid-deletion; doing so can panic or
// double-free. Primary reliable hooks remain exit_tree + on_notification.
impl Drop for GamestrumentsPlayer {
    fn drop(&mut self) {
        // Safe no-op: do not call to_gd() / godot methods or cleanup here.
        // Drop can run during free() / deletion where Gd casts or binds will
        // fail (see "downcast ... failed" and "bind_mut already bound").
        // All cleanup is performed from exit_tree (which fires before delete
        // for normal tree removal and many quit paths). This + explicit
        // child free() after stop+null is our attempt to minimize leaks.
    }
}
