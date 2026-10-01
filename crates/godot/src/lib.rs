use gamestruments_engine::{
    apply_automatic_arrangement, generate_adventure, generate_racing_arrangement,
    generate_suspense_arrangement, racing_root_pitch_class, AdventureInput, AdventureState,
    AdventureStyle, ArrangementRecipe, GameState, GenerateInput, InstrumentPalette, LivePlayer,
    RacingArrangement, Style, SuspenseArrangement, SuspenseInput, SuspenseStyle, TraceState,
};
use godot::classes::{
    AudioServer, AudioStream, AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayer,
};
use godot::prelude::*;

struct GamestrumentsExtension;

/// Output rate until the game sets `sample_rate`.
const DEFAULT_SAMPLE_RATE: f32 = 48000.0;

#[gdextension]
unsafe impl ExtensionLibrary for GamestrumentsExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct GamestrumentsPlayer {
    /// Deterministic per-title namespace, not a security credential.
    #[export]
    project_secret: GString,
    #[export]
    recipe: GString,
    #[export]
    arrangement: GString,
    #[export]
    autoplay: bool,
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
    /// Synth/output rate in Hz, read when the node enters the tree and when a
    /// score is generated (set it before `add_child`/`generate`). Lower it
    /// (e.g. 22050) to cut synthesis CPU when the game's material is
    /// band-limited. Defaults to 48000 so existing consumers are unchanged.
    #[export]
    sample_rate: f64,
    /// Plays the generated scores; shared with the browser Lab.
    live: LivePlayer,
    /// Reused mono render buffer.
    scratch: Vec<f32>,
    stereo_scratch: Vec<Vector2>,
    live_player: Option<Gd<AudioStreamPlayer>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for GamestrumentsPlayer {
    fn init(base: Base<Node>) -> Self {
        Self {
            project_secret: GString::new(),
            recipe: "racing".into(),
            // Empty means "the recipe's own default": Racing → original,
            // Suspense → the phase pool's seeded composer.
            arrangement: GString::new(),
            autoplay: false,
            style: "funk".into(),
            melody_voice: GString::new(),
            harmony_voice: GString::new(),
            drive_voice: GString::new(),
            bass_voice: GString::new(),
            energy: 0.62,
            complexity: 0.6,
            brightness: 0.52,
            syncopation: 0.7,
            sample_rate: f64::from(DEFAULT_SAMPLE_RATE),
            live: LivePlayer::new(DEFAULT_SAMPLE_RATE),
            scratch: Vec::new(),
            stereo_scratch: Vec::new(),
            live_player: None,
            base,
        }
    }

    fn ready(&mut self) {
        let mut generator = AudioStreamGenerator::new_gd();
        generator.set_mix_rate(self.resolved_sample_rate());
        generator.set_buffer_length(0.1);
        let mut player = AudioStreamPlayer::new_alloc();
        player.set_name("LiveStream");
        player.set_stream(&generator);
        let bus = if AudioServer::singleton().get_bus_index("Music") >= 0 {
            "Music"
        } else {
            "Master"
        };
        player.set_bus(bus);
        self.base_mut().add_child(&player);
        player.play();
        self.live_player = Some(player);
    }

    fn exit_tree(&mut self) {
        self.cleanup_audio_child();
    }

    fn process(&mut self, _delta: f64) {
        let player = match self.live_player.as_ref() {
            Some(player) if player.is_instance_valid() => player.clone(),
            _ => return,
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
        let frames = frames as usize;

        if !self.live.is_loaded() {
            return;
        }
        self.scratch.clear();
        self.scratch.resize(frames, 0.0);
        self.live.fill(&mut self.scratch);

        self.stereo_scratch.clear();
        self.stereo_scratch.extend(
            self.scratch
                .iter()
                .map(|sample| Vector2::new(*sample, *sample)),
        );
        let stereo = PackedVector2Array::from(self.stereo_scratch.as_slice());
        playback.push_buffer(&stereo);
    }
}

impl GamestrumentsPlayer {
    fn cleanup_audio_child(&mut self) {
        if let Some(mut p) = self.live_player.take() {
            if !p.is_instance_valid() {
                return;
            }
            p.stop();
            p.set_stream(Gd::<AudioStream>::null_arg());
            // Godot owns this child; stop its audio here and let parent teardown free it.
        }
    }
}

#[godot_api]
impl GamestrumentsPlayer {
    /// The synth/output rate actually used, clamped to a sane audio range.
    fn resolved_sample_rate(&self) -> f32 {
        (self.sample_rate as f32).clamp(8000.0, 96000.0)
    }

    #[func]
    fn generate(&mut self, seed: GString, #[opt(default = "")] opening_section: GString) -> bool {
        if self.project_secret.is_empty() {
            godot_error!("GamestrumentsPlayer.project_secret is empty");
            return false;
        }
        let seed_str = seed.to_string();
        if self.live.is_playing_seed(&seed_str) {
            // Already playing this seed. Do not recompose or restart the clock.
            return if opening_section.is_empty() {
                true
            } else {
                self.cue_section(opening_section)
            };
        }
        let recipe = self.recipe.to_string();
        // The key of the score about to be generated; a replacement is moved into
        // the playing score's key so a seed change blends at the seam.
        let mut new_root: i32 = 0;
        let (score, automatic_recipe) = match recipe.as_str() {
            "adventure" => {
                let style = if self.style.is_empty() {
                    AdventureStyle::Folk
                } else {
                    match AdventureStyle::parse(&self.style.to_string()) {
                        Ok(style) => style,
                        Err(_) => {
                            godot_error!(
                                "Unknown Gamestruments adventure style \"{}\"; use folk, dark, or orchestral",
                                self.style
                            );
                            return false;
                        }
                    }
                };
                (
                    generate_adventure(&AdventureInput {
                        secret: self.project_secret.to_string(),
                        seed: seed.to_string(),
                        style,
                        wonder: self.brightness,
                        danger: self.energy,
                        mystery: self.complexity,
                        motion: self.syncopation,
                    }),
                    Some(ArrangementRecipe::Adventure),
                )
            }
            "suspense" => {
                let style = if self.style.is_empty() {
                    SuspenseStyle::Terminal
                } else {
                    match SuspenseStyle::parse(&self.style.to_string()) {
                        Ok(style) => style,
                        Err(_) => {
                            godot_error!(
                                "Unknown Gamestruments suspense style \"{}\"; use terminal, cipher, noir, or trance",
                                self.style
                            );
                            return false;
                        }
                    }
                };
                let Ok(arrangement) = SuspenseArrangement::parse(&self.arrangement.to_string())
                else {
                    godot_error!(
                        "Unknown Gamestruments suspense arrangement \"{}\"; use all-phases or seeded",
                        self.arrangement
                    );
                    return false;
                };
                (
                    generate_suspense_arrangement(
                        &SuspenseInput {
                            secret: self.project_secret.to_string(),
                            seed: seed.to_string(),
                            style,
                            tension: self.energy,
                            heat: self.complexity,
                            mystery: self.brightness,
                            pulse: self.syncopation,
                        },
                        arrangement,
                    ),
                    None,
                )
            }
            "racing" => {
                let Ok(style) = Style::parse(&self.style.to_string()) else {
                    godot_error!("Unknown Gamestruments racing style");
                    return false;
                };
                let Ok(arrangement) = RacingArrangement::parse(&self.arrangement.to_string())
                else {
                    godot_error!("Unknown Gamestruments racing arrangement");
                    return false;
                };
                let recipe = match arrangement {
                    RacingArrangement::Original => Some(ArrangementRecipe::Racing),
                    RacingArrangement::Extended => Some(ArrangementRecipe::RacingExtended),
                    // All phases and the seeded composer already carry their own
                    // song form; no automatic arrangement layers on top of them.
                    RacingArrangement::AllPhases | RacingArrangement::Seeded => None,
                };
                let input = GenerateInput {
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
                };
                new_root = racing_root_pitch_class(&input);
                (generate_racing_arrangement(&input, arrangement), recipe)
            }
            other => {
                godot_error!("Unknown Gamestruments recipe \"{other}\"");
                return false;
            }
        };
        let score = score.and_then(|score| match automatic_recipe {
            Some(recipe) => apply_automatic_arrangement(score, recipe, self.autoplay),
            None => Ok(score),
        });
        let score = match score {
            Ok(score) => score,
            Err(error) => {
                godot_error!("Gamestruments generation failed: {error}");
                return false;
            }
        };

        let rate = self.resolved_sample_rate();
        if !self.live.is_loaded() && self.live.sample_rate() != rate {
            self.live = LivePlayer::new(rate);
        }
        let opening = opening_section.to_string();
        let opening = (!opening.is_empty()).then_some(opening);
        match self
            .live
            .load(score, &seed_str, &recipe, new_root, opening.as_deref())
        {
            Ok(()) => true,
            Err(error) => {
                godot_error!("Gamestruments transport failed: {error}");
                false
            }
        }
    }

    #[func]
    fn set_race_state(
        &mut self,
        phase: GString,
        intensity: f64,
        pressure: f64,
        final_lap: bool,
        #[opt(default = "none")] finish_result: GString,
    ) -> bool {
        if !intensity.is_finite()
            || !pressure.is_finite()
            || !(0.0..=1.0).contains(&intensity)
            || !(0.0..=1.0).contains(&pressure)
        {
            godot_error!("Gamestruments race intensity and pressure must be within 0.0..1.0");
            return false;
        }
        let state = GameState {
            intensity,
            position_pressure: pressure,
            final_lap,
            race_phase: phase.to_string(),
            finish_result: if finish_result.is_empty() {
                "none".into()
            } else {
                finish_result.to_string()
            },
        };
        let accepted = self.live.request_state(&state);
        if !accepted {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_race_state");
        }
        accepted
    }

    #[func]
    fn cue_section(&mut self, section: GString) -> bool {
        let target = section.to_string();
        if !self.live.has_section(&target) {
            godot_error!("Unknown music section or no generated score: {target}");
            return false;
        }
        self.live.cue_section(&target)
    }

    #[func]
    fn set_form_hold(&mut self, held: bool) -> bool {
        self.live.set_form_held(held)
    }

    #[func]
    fn advance_form(&mut self) -> bool {
        self.live.advance_form()
    }

    #[func]
    fn is_form_held(&self) -> bool {
        self.live.is_form_held()
    }

    #[func]
    fn get_current_section(&self) -> GString {
        self.live
            .current_section()
            .map(GString::from)
            .unwrap_or_default()
    }

    #[func]
    fn set_trace_state(&mut self, phase: GString, heat: f64, focus: f64, progress: f64) -> bool {
        if !heat.is_finite()
            || !focus.is_finite()
            || !progress.is_finite()
            || !(0.0..=1.0).contains(&heat)
            || !(0.0..=1.0).contains(&focus)
            || !(0.0..=1.0).contains(&progress)
        {
            godot_error!("Gamestruments trace heat, focus, and progress must be within 0.0..1.0");
            return false;
        }
        let state = TraceState {
            phase: phase.to_string(),
            heat,
            focus,
            progress,
        };
        let accepted = self.live.request_trace_state(&state);
        if !accepted {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_trace_state");
        }
        accepted
    }

    #[func]
    fn set_adventure_state(
        &mut self,
        area_phase: GString,
        discovery: f64,
        threat: f64,
        quest_complete: bool,
    ) -> bool {
        for (name, value) in [("discovery", discovery), ("threat", threat)] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                godot_error!("Gamestruments adventure {name} must be within 0.0..1.0");
                return false;
            }
        }
        let state = AdventureState {
            area_phase: area_phase.to_string(),
            discovery,
            threat,
            quest_complete,
        };
        let accepted = self.live.request_adventure_state(&state);
        if !accepted {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_adventure_state");
        }
        accepted
    }
}
