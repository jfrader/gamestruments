use gamestruments_engine::{
    apply_automatic_arrangement, generate_adventure, generate_racing_arrangement,
    generate_suspense_arrangement,
    handoff::{crossfade_gains, crossfade_sample_count, Handoff},
    master::{MasterChain, MasterConfig},
    racing_root_pitch_class, AdaptiveTransport, AdventureInput, AdventureState, AdventureStyle,
    ArrangementRecipe, FormAudio, GameState, GenerateInput, InstrumentPalette, PortableScore,
    RacingArrangement, Style, SuspenseArrangement, SuspenseInput, SuspenseStyle, Synth, TraceState,
};
use godot::classes::{
    AudioServer, AudioStream, AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayer,
};
use godot::prelude::*;

struct GamestrumentsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GamestrumentsExtension {}

/// Live playback state for one generated score. Extracted so the player can
/// hold an active voice + optional outgoing voice for crossfade handoff.
struct Voice {
    score: Option<PortableScore>,
    transport: Option<AdaptiveTransport>,
    synth: Synth,
    form_audio: Option<FormAudio>,
    master: Option<MasterChain>,
    ticks_per_second: f64,
    tick: u32,
    frames_produced: u64,
    /// The seed passed to generate(), used to detect "already playing this seed".
    seed: String,
    /// Root pitch class this voice's score is in, so a replacement can be
    /// transposed into the same key.
    root_pitch_class: i32,
    /// Recipe this voice was generated from; a replacement only inherits the
    /// key and tempo when it is the same recipe.
    recipe: String,
    /// Reused per-voice mono scratch so two voices can render without
    /// allocating a fresh Vec on every audio frame.
    scratch: Vec<f32>,
}

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
    /// Active (incoming) voice. All live state lives here; the old top-level
    /// fields were replaced by this (no parallel copies).
    active: Option<Voice>,
    /// A new voice parked here to wait for the active voice to reach a bar boundary.
    pending: Option<Voice>,
    /// The voice that is fading out during a handoff. Stays until the fade
    /// sample count (computed from *its* score) is reached.
    outgoing: Option<Voice>,
    /// Crossfade progress between the outgoing and incoming voice. Owns the
    /// sample counter so a re-targeted handoff keeps its fade continuous.
    handoff: Handoff,
    /// Reused mix buffer (mono) for final output or crossfade sum. The per-voice
    /// scratches inside Voice are the render targets for synth/form.
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
            sample_rate: 48000.0,
            active: None,
            pending: None,
            outgoing: None,
            handoff: Handoff::idle(),
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
        let sample_rate = f64::from(self.resolved_sample_rate());

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

        // A voice parked by `generate` is promoted once the playing score reaches
        // its next bar boundary, so a seed change swaps on the bar like a cue.
        let promote = match (self.pending.is_some(), self.active.as_ref()) {
            (true, Some(active)) => {
                let bar_ticks = active.score.as_ref().map(|s| s.bar_ticks()).unwrap_or(1);
                let current_bar = active.tick / bar_ticks;
                let next_tick = gamestruments_engine::synth::tick_at_sample(
                    active.frames_produced + frames as u64,
                    sample_rate,
                    active.ticks_per_second,
                );
                next_tick / bar_ticks > current_bar || active.tick == 0
            }
            _ => false,
        };
        if promote {
            let was_handoff = self.outgoing.is_some() && self.handoff.is_active();
            if was_handoff {
                // A crossfade is already running: keep the voice that is fading
                // out and drop the incoming it was replacing.
                let _ = self.active.take();
            } else {
                self.outgoing = self.active.take();
            }
            self.active = self.pending.take();
            let rate = self.resolved_sample_rate() as u32;
            let total = self
                .outgoing
                .as_ref()
                .and_then(|out| out.score.as_ref())
                .map(|sc| crossfade_sample_count(sc, rate))
                .unwrap_or(0);
            if was_handoff {
                self.handoff.retarget(total);
            } else {
                self.handoff.begin(total);
            }
        }

        let Some(active) = self.active.as_mut() else {
            return;
        };

        // Prepare the mix buffer we will eventually push (reused, no alloc).
        self.scratch.clear();
        self.scratch.resize(frames, 0.0);

        let in_handoff = self.outgoing.is_some() && self.handoff.is_active();

        if in_handoff {
            let out_v = self.outgoing.as_mut().unwrap();

            // Render each voice into *its own* reused scratch (two-voice case).
            out_v.scratch.clear();
            out_v.scratch.resize(frames, 0.0);
            GamestrumentsPlayer::fill_voice_buffer(out_v, frames, sample_rate);

            active.scratch.clear();
            active.scratch.resize(frames, 0.0);
            GamestrumentsPlayer::fill_voice_buffer(active, frames, sample_rate);

            // Each voice's own stateful MasterChain (compressor, limiter
            // lookahead) runs on its own buffer *before* mixing, so the outgoing
            // voice keeps the dynamics it had while it was the sole voice.
            if let Some(master) = out_v.master.as_mut() {
                master.process(&mut out_v.scratch);
            }
            if let Some(master) = active.master.as_mut() {
                master.process(&mut active.scratch);
            }

            // Hold the outgoing at full gain until the incoming voice has
            // produced signal at or above the musical floor (or the hold bound
            // elapses), then fade from zero. While holding, `progress` is pinned
            // at zero so the mix below keeps the outgoing at full gain and the
            // incoming silent.
            self.handoff.poll(&active.scratch);

            // Mix using the provided crossfade gains over the outgoing's sample
            // count. The incoming continues at its natural level: no gain-match is
            // applied, so its applied gain is exactly the fade law's `g_act` and
            // reaches 1.0 at the fade's end with no residual scale factor.
            for i in 0..frames {
                let progress = self.handoff.progress(i as u64);
                let (g_out, g_act) = crossfade_gains(progress);
                self.scratch[i] = g_out * out_v.scratch[i] + g_act * active.scratch[i];
            }

            // Advance clocks for both voices (outgoing keeps its audible time).
            out_v.frames_produced = out_v.frames_produced.saturating_add(frames as u64);
            active.frames_produced = active.frames_produced.saturating_add(frames as u64);

            self.handoff.advance(frames as u64);

            // Both voices were individually ceiling'd by their own chains, but
            // the crossfade gains need not sum to exactly 1.0, so clamp the
            // mixed sum to the master ceiling as a backstop.
            let ceiling = active
                .master
                .as_ref()
                .map(MasterChain::ceiling_linear)
                .unwrap_or(1.0);
            for sample in &mut self.scratch {
                *sample = sample.clamp(-ceiling, ceiling);
            }

            if self.handoff.finished() {
                let _ = self.outgoing.take();
                self.handoff.reset();
            }
        } else {
            // Single active voice — render directly into its scratch then copy to mix.
            active.scratch.clear();
            active.scratch.resize(frames, 0.0);
            GamestrumentsPlayer::fill_voice_buffer(active, frames, sample_rate);

            self.scratch.copy_from_slice(&active.scratch[..]);

            active.frames_produced = active.frames_produced.saturating_add(frames as u64);

            if let Some(master) = active.master.as_mut() {
                master.process(&mut self.scratch);
            }
        }

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

    /// Fill a voice's own scratch using the exact original sample-accurate logic
    /// (form_audio path or the non-form event window path). The caller has
    /// already sized voice.scratch to `frames`.
    fn fill_voice_buffer(voice: &mut Voice, frames: usize, sample_rate: f64) {
        let ticks_per_second = voice.ticks_per_second;
        let Some(score) = voice.score.as_ref() else {
            return;
        };

        if let Some(form_audio) = voice.form_audio.as_mut() {
            if let Some(transport) = voice.transport.as_mut() {
                form_audio.fill(score, transport, &mut voice.scratch);
                voice.tick = form_audio.tick(ticks_per_second);
            }
        } else {
            // The score clock is the produced-sample count mapped to ticks, so
            // score time and synth phase advance together and never drift.
            let start_tick = voice.tick;
            let next_tick = gamestruments_engine::synth::tick_at_sample(
                voice.frames_produced + frames as u64,
                sample_rate,
                ticks_per_second,
            );
            // The window is exactly the ticks this buffer covers: contiguous
            // with the previous buffer, so no event is dropped or doubled.
            let span_ticks = next_tick.saturating_sub(start_tick).max(1);
            if let Some(transport) = voice.transport.as_mut() {
                transport.advance(start_tick);
                let section = transport.current_section().to_string();
                let length = score.section(&section).map(|s| s.length_ticks).unwrap_or(1);
                let local = if transport.has_form() {
                    transport.phrase_tick(start_tick)
                } else {
                    start_tick % length.max(1)
                };
                for event in gamestruments_engine::synth::events_starting_at(
                    score, &section, local, span_ticks,
                ) {
                    // Place the event at its true sample offset inside this
                    // buffer instead of starting the whole window at once.
                    let offset_seconds =
                        (f64::from(event.start_tick()) - f64::from(local)) / ticks_per_second;
                    voice
                        .synth
                        .trigger_at(&event, ticks_per_second, offset_seconds.max(0.0));
                }
            }
            voice.synth.fill(&mut voice.scratch);
            voice.tick = next_tick;
        }
    }
}

#[godot_api]
impl GamestrumentsPlayer {
    /// The synth/output rate actually used, clamped to a sane audio range.
    fn resolved_sample_rate(&self) -> f32 {
        (self.sample_rate as f32).clamp(8000.0, 96000.0)
    }

    /// Build a fresh Voice exactly as the old generate did for its top-level
    /// state. The handoff/park decision is done by the caller (generate).
    fn voice_from_score(
        &self,
        score: PortableScore,
        seed: String,
        opening_section: Option<&str>,
    ) -> Result<Voice, String> {
        let rate_f = self.resolved_sample_rate();
        let rate = rate_f as u32;
        let mut voice = Voice {
            score: None,
            transport: None,
            synth: Synth::new(rate_f),
            form_audio: None,
            master: Some(MasterChain::new(rate, MasterConfig::default())),
            ticks_per_second: score.ticks_per_second(),
            tick: 0,
            frames_produced: 0,
            seed,
            root_pitch_class: 0,
            recipe: String::new(),
            scratch: Vec::new(),
        };
        let initial = opening_section
            .unwrap_or(&score.default_section)
            .to_string();
        if score.section(&initial).is_none() {
            return Err(format!("Unknown opening section: {initial}"));
        }
        match AdaptiveTransport::new(score.clone(), Some(&initial)) {
            Ok(transport) => {
                // Any score with a form blends through the aligned renderer.
                // Seeded and pool forms leave `origin` unset; gating on
                // TransitionStart skipped that blend and hard-cut phase cues.
                voice.form_audio = if score.form.is_some() {
                    Some(FormAudio::new(&score, rate_f))
                } else {
                    None
                };
                voice.transport = Some(transport);
                voice.score = Some(score);
                Ok(voice)
            }
            Err(e) => Err(e),
        }
    }

    #[func]
    fn generate(&mut self, seed: GString, #[opt(default = "")] opening_section: GString) -> bool {
        if self.project_secret.is_empty() {
            godot_error!("GamestrumentsPlayer.project_secret is empty");
            return false;
        }
        let seed_str = seed.to_string();
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.seed == seed_str)
        {
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

        // A replacement score inherits the playing score's key and tempo, so a
        // seed change blends at the seam like a section cue instead of clashing
        // two keys and two tempos.
        let mut score = score;
        let previous = self
            .pending
            .as_ref()
            .or(self.active.as_ref())
            .filter(|voice| voice.recipe == recipe);
        let carried = previous.and_then(|voice| {
            voice
                .score
                .as_ref()
                .map(|previous_score| (voice.root_pitch_class, previous_score.bpm))
        });
        if let Some((key, bpm)) = carried {
            score.transpose((key - new_root).rem_euclid(12));
            score.bpm = bpm;
        }

        let requested_opening = opening_section.to_string();
        let requested_opening = (!requested_opening.is_empty()).then_some(requested_opening);
        match self.voice_from_score(score, seed.to_string(), requested_opening.as_deref()) {
            Ok(mut v) => {
                // Record the key the score is actually in, so a later seed change
                // carries from the sounding key rather than the generated one.
                v.root_pitch_class = carried.map_or(new_root, |(key, _)| key);
                v.recipe = recipe.clone();
                if self.active.is_some() || self.pending.is_some() {
                    // Park the replacement; `process` promotes it on the next bar
                    // boundary so the change stays bar-aligned and click-free.
                    self.pending = Some(v);
                    return true;
                }
                self.active = Some(v);
                self.handoff.reset();
                true
            }
            Err(error) => {
                godot_error!("Gamestruments transport failed: {error}");
                false
            }
        }
    }

    /// Apply a state update to every voice that can be live this bar: the
    /// playing voice and any replacement parked for the next bar boundary, so a
    /// cue is not lost while the replacement waits.
    fn for_each_live_voice(
        &mut self,
        mut update: impl FnMut(&mut AdaptiveTransport, u32) -> bool,
    ) -> bool {
        let mut accepted = false;
        for voice in [self.active.as_mut(), self.pending.as_mut()]
            .into_iter()
            .flatten()
        {
            let tick = voice.tick;
            if let Some(transport) = voice.transport.as_mut() {
                accepted |= update(transport, tick);
            }
        }
        accepted
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
        let accepted = self.for_each_live_voice(|transport, tick| {
            transport.request_state(&state, tick);
            true
        });
        if !accepted {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_race_state");
        }
        accepted
    }

    #[func]
    fn cue_section(&mut self, section: GString) -> bool {
        let target = section.to_string();
        let known = self
            .active
            .as_ref()
            .and_then(|v| v.score.as_ref())
            .and_then(|score| score.section(&target))
            .or_else(|| {
                self.pending
                    .as_ref()
                    .and_then(|v| v.score.as_ref())
                    .and_then(|score| score.section(&target))
            })
            .is_some();
        if !known {
            godot_error!("Unknown music section or no generated score: {target}");
            return false;
        }
        self.for_each_live_voice(|transport, tick| {
            transport.request_section(&target, tick);
            true
        })
    }

    #[func]
    fn set_form_hold(&mut self, held: bool) -> bool {
        self.for_each_live_voice(|transport, tick| {
            if transport.has_form() {
                transport.set_form_held(held, tick);
                true
            } else {
                false
            }
        })
    }

    #[func]
    fn advance_form(&mut self) -> bool {
        self.for_each_live_voice(|transport, tick| {
            if transport.next_form_section(tick).is_none() {
                false
            } else {
                transport.advance_form(tick);
                true
            }
        })
    }

    #[func]
    fn is_form_held(&self) -> bool {
        [self.active.as_ref(), self.pending.as_ref()]
            .into_iter()
            .flatten()
            .any(|v| {
                v.transport
                    .as_ref()
                    .is_some_and(AdaptiveTransport::is_form_held)
            })
    }

    #[func]
    fn get_current_section(&self) -> GString {
        self.active
            .as_ref()
            .and_then(|active| {
                active.transport.as_ref().and_then(|transport| {
                    transport
                        .playback_at(active.tick)
                        .into_iter()
                        .flatten()
                        .last()
                        .map(|part| GString::from(part.section))
                })
            })
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
        let accepted = self.for_each_live_voice(|transport, tick| {
            transport.request_trace_state(&state, tick);
            true
        });
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
        let accepted = self.for_each_live_voice(|transport, tick| {
            transport.request_adventure_state(&state, tick);
            true
        });
        if !accepted {
            godot_error!("GamestrumentsPlayer.generate must succeed before set_adventure_state");
        }
        accepted
    }
}
