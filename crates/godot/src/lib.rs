use gamestruments_engine::{master::{MasterChain, MasterConfig},
    apply_automatic_arrangement, generate_adventure, generate_racing_arrangement,
    generate_suspense_arrangement, AdaptiveTransport, AdventureInput, AdventureState,
    AdventureStyle, ArrangementRecipe, FormAudio, GameState, GenerateInput, InstrumentPalette,
    PortableScore, RacingArrangement, Style, SuspenseArrangement, SuspenseInput, SuspenseStyle,
    Synth, TraceState,
};
use godot::classes::{
    AudioServer, AudioStream, AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayer,
};
use godot::prelude::*;

struct GamestrumentsExtension;

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
    score: Option<PortableScore>,
    transport: Option<AdaptiveTransport>,
    synth: Synth,
    form_audio: Option<FormAudio>,
    master: Option<MasterChain>,
    ticks_per_second: f64,
    tick: u32,
    sample_rate: f32,
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
            score: None,
            transport: None,
            synth: Synth::new(48000.0),
            form_audio: None,
            master: None,
            ticks_per_second: 2160.0,
            tick: 0,
            sample_rate: 48000.0,
            live_player: None,
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
        let Some(score) = self.score.as_ref() else {
            return;
        };
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
        let mut buffer = vec![0.0_f32; frames as usize];
        if let Some(form_audio) = self.form_audio.as_mut() {
            if let Some(transport) = self.transport.as_mut() {
                form_audio.fill(score, transport, &mut buffer);
                self.tick = form_audio.tick(self.ticks_per_second);
            }
            if let Some(master) = self.master.as_mut() {
                master.process(&mut buffer);
            }
            for sample in buffer {
                playback.push_frame(Vector2::new(sample, sample));
            }
            return;
        }
        let window_ticks = ((frames as f64 / f64::from(self.sample_rate)) * self.ticks_per_second)
            .ceil() as u32
            + 1;
        if let Some(transport) = self.transport.as_mut() {
            transport.advance(self.tick);
            let section = transport.current_section().to_string();
            let length = score.section(&section).map(|s| s.length_ticks).unwrap_or(1);
            let local = if transport.has_form() {
                transport.phrase_tick(self.tick)
            } else {
                self.tick % length.max(1)
            };
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
        if let Some(master) = self.master.as_mut() {
            master.process(&mut buffer);
        }
        for sample in buffer {
            playback.push_frame(Vector2::new(sample, sample));
        }
        self.tick = self.tick.wrapping_add(window_ticks.max(1));
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
    #[func]
    fn generate(&mut self, seed: GString) -> bool {
        if self.project_secret.is_empty() {
            godot_error!("GamestrumentsPlayer.project_secret is empty");
            return false;
        }
        let recipe = self.recipe.to_string();
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
                                "Unknown Gamestruments suspense style \"{}\"; use terminal, cipher, or noir",
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
                (
                    generate_racing_arrangement(
                        &GenerateInput {
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
                        },
                        arrangement,
                    ),
                    recipe,
                )
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
        self.ticks_per_second = score.ticks_per_second();
        self.tick = 0;
        self.synth = Synth::new(self.sample_rate);
        self.master = Some(MasterChain::new(self.sample_rate as u32, MasterConfig::default()));
        let initial = score.default_section.clone();
        match AdaptiveTransport::new(score.clone(), Some(&initial)) {
            Ok(transport) => {
                self.form_audio = if score.form.as_ref().and_then(|form| form.origin)
                    == Some(gamestruments_engine::score::FormOrigin::TransitionStart)
                {
                    Some(FormAudio::new(&score, self.sample_rate))
                } else {
                    None
                };
                self.transport = Some(transport);
                self.score = Some(score);
                true
            }
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
        let Some(transport) = self.transport.as_mut() else {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_race_state");
            return false;
        };
        if !intensity.is_finite()
            || !pressure.is_finite()
            || !(0.0..=1.0).contains(&intensity)
            || !(0.0..=1.0).contains(&pressure)
        {
            godot_error!("Gamestruments race intensity and pressure must be within 0.0..1.0");
            return false;
        }
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
        true
    }

    #[func]
    fn cue_section(&mut self, section: GString) -> bool {
        let target = section.to_string();
        if self
            .score
            .as_ref()
            .and_then(|score| score.section(&target))
            .is_none()
        {
            godot_error!("Unknown music section or no generated score: {target}");
            return false;
        }
        let Some(transport) = self.transport.as_mut() else {
            return false;
        };
        transport.request_section(&target, self.tick);
        true
    }

    #[func]
    fn set_form_hold(&mut self, held: bool) -> bool {
        let Some(transport) = self.transport.as_mut() else {
            return false;
        };
        if !transport.has_form() {
            return false;
        }
        transport.set_form_held(held, self.tick);
        true
    }

    #[func]
    fn advance_form(&mut self) -> bool {
        let Some(transport) = self.transport.as_mut() else {
            return false;
        };
        if transport.next_form_section(self.tick).is_none() {
            return false;
        }
        transport.advance_form(self.tick);
        true
    }

    #[func]
    fn is_form_held(&self) -> bool {
        self.transport
            .as_ref()
            .is_some_and(AdaptiveTransport::is_form_held)
    }

    #[func]
    fn get_current_section(&self) -> GString {
        self.transport
            .as_ref()
            .and_then(|transport| {
                transport
                    .playback_at(self.tick)
                    .into_iter()
                    .flatten()
                    .find(|part| part.percussion)
                    .map(|part| GString::from(part.section))
            })
            .unwrap_or_default()
    }

    #[func]
    fn set_trace_state(&mut self, phase: GString, heat: f64, focus: f64, progress: f64) -> bool {
        let Some(transport) = self.transport.as_mut() else {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_trace_state");
            return false;
        };
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
        transport.request_trace_state(
            &TraceState {
                phase: phase.to_string(),
                heat,
                focus,
                progress,
            },
            self.tick,
        );
        true
    }

    #[func]
    fn set_adventure_state(
        &mut self,
        area_phase: GString,
        discovery: f64,
        threat: f64,
        quest_complete: bool,
    ) -> bool {
        let Some(transport) = self.transport.as_mut() else {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_adventure_state");
            return false;
        };
        for (name, value) in [("discovery", discovery), ("threat", threat)] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                godot_error!("Gamestruments adventure {name} must be within 0.0..1.0");
                return false;
            }
        }
        transport.request_adventure_state(
            &AdventureState {
                area_phase: area_phase.to_string(),
                discovery,
                threat,
                quest_complete,
            },
            self.tick,
        );
        true
    }
}
