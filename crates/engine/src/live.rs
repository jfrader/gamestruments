//! The live player every binding drives: it plays a generated score in real
//! time, parks a replacement score until the next bar, crossfades the handoff,
//! and routes cues and game state to the transport. Godot and the browser Lab
//! both play through it, so they sound the same.

use crate::handoff::{crossfade_gains, crossfade_sample_count, Handoff};
use crate::master::{MasterChain, MasterConfig};
use crate::score::{AdventureState, GameState, PortableScore, TraceState};
use crate::synth::{events_starting_at, tick_at_sample, Synth};
use crate::{AdaptiveTransport, FormAudio};

/// Live playback state for one generated score.
struct Voice {
    score: PortableScore,
    transport: AdaptiveTransport,
    synth: Synth,
    form_audio: Option<FormAudio>,
    master: MasterChain,
    ticks_per_second: f64,
    tick: u32,
    frames_produced: u64,
    /// The seed this score was generated from.
    seed: String,
    /// Root pitch class the score sounds in, so a replacement can be
    /// transposed into the same key.
    root_pitch_class: i32,
    /// Recipe this voice was generated from; a replacement only inherits the
    /// key and tempo when it is the same recipe.
    recipe: String,
    /// Reused render target, so two voices render without allocating.
    scratch: Vec<f32>,
}

impl Voice {
    fn new(
        score: PortableScore,
        seed: String,
        recipe: String,
        root_pitch_class: i32,
        opening_section: Option<&str>,
        sample_rate: f32,
    ) -> Result<Self, String> {
        let initial = opening_section
            .unwrap_or(&score.default_section)
            .to_string();
        if score.section(&initial).is_none() {
            return Err(format!("Unknown opening section: {initial}"));
        }
        let transport = AdaptiveTransport::new(score.clone(), Some(&initial))?;
        // Any score with a form blends through the aligned renderer.
        let form_audio = score
            .form
            .is_some()
            .then(|| FormAudio::new(&score, sample_rate));
        Ok(Self {
            ticks_per_second: score.ticks_per_second(),
            score,
            transport,
            synth: Synth::new(sample_rate),
            form_audio,
            master: MasterChain::new(sample_rate as u32, MasterConfig::default()),
            tick: 0,
            frames_produced: 0,
            seed,
            root_pitch_class,
            recipe,
            scratch: Vec::new(),
        })
    }

    /// Render `frames` samples into the voice's own scratch, sample-accurately.
    fn fill(&mut self, frames: usize, sample_rate: f64) {
        self.scratch.clear();
        self.scratch.resize(frames, 0.0);
        if let Some(form_audio) = self.form_audio.as_mut() {
            form_audio.fill(&self.score, &mut self.transport, &mut self.scratch);
            self.tick = form_audio.tick(self.ticks_per_second);
            return;
        }
        // The score clock is the produced-sample count mapped to ticks, so
        // score time and synth phase advance together and never drift.
        let start_tick = self.tick;
        let next_tick = tick_at_sample(
            self.frames_produced + frames as u64,
            sample_rate,
            self.ticks_per_second,
        );
        // The window is exactly the ticks this buffer covers: contiguous with
        // the previous buffer, so no event is dropped or doubled.
        let span_ticks = next_tick.saturating_sub(start_tick).max(1);
        self.transport.advance(start_tick);
        let section = self.transport.current_section().to_string();
        let length = self
            .score
            .section(&section)
            .map(|s| s.length_ticks)
            .unwrap_or(1);
        let local = if self.transport.has_form() {
            self.transport.phrase_tick(start_tick)
        } else {
            start_tick % length.max(1)
        };
        for event in events_starting_at(&self.score, &section, local, span_ticks) {
            // Place the event at its true sample offset inside this buffer
            // instead of starting the whole window at once.
            let offset_seconds =
                (f64::from(event.start_tick()) - f64::from(local)) / self.ticks_per_second;
            self.synth
                .trigger_at(&event, self.ticks_per_second, offset_seconds.max(0.0));
        }
        self.synth.fill(&mut self.scratch);
        self.tick = next_tick;
    }
}

/// Plays generated scores in real time, one mono buffer at a time.
pub struct LivePlayer {
    sample_rate: f32,
    /// The voice that is playing.
    active: Option<Voice>,
    /// A replacement waiting for the active voice to reach a bar boundary.
    pending: Option<Voice>,
    /// The voice fading out during a handoff.
    outgoing: Option<Voice>,
    /// Crossfade progress between the outgoing and the active voice.
    handoff: Handoff,
}

impl LivePlayer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            active: None,
            pending: None,
            outgoing: None,
            handoff: Handoff::idle(),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Whether a score is loaded.
    pub fn is_loaded(&self) -> bool {
        self.active.is_some()
    }

    /// Whether the playing score was generated from `seed`.
    pub fn is_playing_seed(&self, seed: &str) -> bool {
        self.active.as_ref().is_some_and(|voice| voice.seed == seed)
    }

    /// Start playing `score`, or park it until the next bar when a score is
    /// already playing. A replacement of the same recipe inherits the playing
    /// score's key and tempo, so a seed change blends at the seam instead of
    /// clashing two keys and two tempos. `root_pitch_class` is the key the
    /// score was generated in.
    pub fn load(
        &mut self,
        mut score: PortableScore,
        seed: &str,
        recipe: &str,
        root_pitch_class: i32,
        opening_section: Option<&str>,
    ) -> Result<(), String> {
        let carried = self
            .pending
            .as_ref()
            .or(self.active.as_ref())
            .filter(|voice| voice.recipe == recipe)
            .map(|voice| (voice.root_pitch_class, voice.score.bpm));
        if let Some((key, bpm)) = carried {
            score.transpose((key - root_pitch_class).rem_euclid(12));
            score.bpm = bpm;
        }
        // Record the key the score actually sounds in, so a later seed change
        // carries from the sounding key rather than the generated one.
        let sounding_root = carried.map_or(root_pitch_class, |(key, _)| key);
        let voice = Voice::new(
            score,
            seed.to_string(),
            recipe.to_string(),
            sounding_root,
            opening_section,
            self.sample_rate,
        )?;
        if self.active.is_some() || self.pending.is_some() {
            self.pending = Some(voice);
        } else {
            self.active = Some(voice);
            self.handoff.reset();
        }
        Ok(())
    }

    /// Render the next `out.len()` mono samples.
    pub fn fill(&mut self, out: &mut [f32]) {
        let frames = out.len();
        let sample_rate = f64::from(self.sample_rate);
        out.fill(0.0);
        self.promote_pending(frames, sample_rate);
        let Some(active) = self.active.as_mut() else {
            return;
        };

        let in_handoff = self.outgoing.is_some() && self.handoff.is_active();
        if !in_handoff {
            active.fill(frames, sample_rate);
            out.copy_from_slice(&active.scratch);
            active.frames_produced = active.frames_produced.saturating_add(frames as u64);
            active.master.process(out);
            return;
        }

        let outgoing = self
            .outgoing
            .as_mut()
            .expect("a handoff has an outgoing voice");
        outgoing.fill(frames, sample_rate);
        active.fill(frames, sample_rate);
        // Each voice's own stateful master chain runs on its own buffer before
        // mixing, so the outgoing keeps the dynamics it had as the sole voice.
        outgoing.master.process(&mut outgoing.scratch);
        active.master.process(&mut active.scratch);

        // Hold the outgoing at full gain until the incoming produces signal at
        // the musical floor (or the hold bound elapses), then fade from zero.
        self.handoff.poll(&active.scratch);
        for (i, sample) in out.iter_mut().enumerate() {
            let (g_out, g_act) = crossfade_gains(self.handoff.progress(i as u64));
            *sample = g_out * outgoing.scratch[i] + g_act * active.scratch[i];
        }
        outgoing.frames_produced = outgoing.frames_produced.saturating_add(frames as u64);
        active.frames_produced = active.frames_produced.saturating_add(frames as u64);
        self.handoff.advance(frames as u64);

        // Each voice was ceiling'd by its own chain, but the crossfade gains
        // need not sum to exactly 1.0: clamp the mix to the master ceiling.
        let ceiling = active.master.ceiling_linear();
        for sample in out.iter_mut() {
            *sample = sample.clamp(-ceiling, ceiling);
        }
        if self.handoff.finished() {
            self.outgoing = None;
            self.handoff.reset();
        }
    }

    /// Promote a parked replacement once the playing score reaches its next
    /// bar boundary within the coming `frames`, so a seed change swaps on the
    /// bar like a cue.
    fn promote_pending(&mut self, frames: usize, sample_rate: f64) {
        let (Some(_), Some(active)) = (self.pending.as_ref(), self.active.as_ref()) else {
            return;
        };
        let bar_ticks = active.score.bar_ticks();
        let next_tick = tick_at_sample(
            active.frames_produced + frames as u64,
            sample_rate,
            active.ticks_per_second,
        );
        if next_tick / bar_ticks <= active.tick / bar_ticks && active.tick != 0 {
            return;
        }
        let retarget = self.outgoing.is_some() && self.handoff.is_active();
        if retarget {
            // A crossfade is already running: keep the voice fading out and
            // drop the incoming it was replacing.
            self.active = None;
        } else {
            self.outgoing = self.active.take();
        }
        self.active = self.pending.take();
        let total = self
            .outgoing
            .as_ref()
            .map(|voice| crossfade_sample_count(&voice.score, self.sample_rate as u32))
            .unwrap_or(0);
        if retarget {
            self.handoff.retarget(total);
        } else {
            self.handoff.begin(total);
        }
    }

    /// Apply a transport update to every voice that can be live this bar: the
    /// playing voice and a replacement parked for the next bar, so a cue is not
    /// lost while the replacement waits. Returns whether any voice accepted it.
    fn for_each_live_voice(
        &mut self,
        mut update: impl FnMut(&mut AdaptiveTransport, u32) -> bool,
    ) -> bool {
        let mut accepted = false;
        for voice in [self.active.as_mut(), self.pending.as_mut()]
            .into_iter()
            .flatten()
        {
            accepted |= update(&mut voice.transport, voice.tick);
        }
        accepted
    }

    /// Whether a live score has `section`.
    pub fn has_section(&self, section: &str) -> bool {
        [self.active.as_ref(), self.pending.as_ref()]
            .into_iter()
            .flatten()
            .any(|voice| voice.score.section(section).is_some())
    }

    pub fn cue_section(&mut self, section: &str) -> bool {
        self.for_each_live_voice(|transport, tick| {
            transport.request_section(section, tick);
            true
        })
    }

    pub fn request_state(&mut self, state: &GameState) -> bool {
        self.for_each_live_voice(|transport, tick| {
            transport.request_state(state, tick);
            true
        })
    }

    pub fn request_trace_state(&mut self, state: &TraceState) -> bool {
        self.for_each_live_voice(|transport, tick| {
            transport.request_trace_state(state, tick);
            true
        })
    }

    pub fn request_adventure_state(&mut self, state: &AdventureState) -> bool {
        self.for_each_live_voice(|transport, tick| {
            transport.request_adventure_state(state, tick);
            true
        })
    }

    pub fn set_form_held(&mut self, held: bool) -> bool {
        self.for_each_live_voice(|transport, tick| {
            if transport.has_form() {
                transport.set_form_held(held, tick);
                true
            } else {
                false
            }
        })
    }

    pub fn advance_form(&mut self) -> bool {
        self.for_each_live_voice(|transport, tick| {
            if transport.next_form_section(tick).is_none() {
                false
            } else {
                transport.advance_form(tick);
                true
            }
        })
    }

    pub fn is_form_held(&self) -> bool {
        [self.active.as_ref(), self.pending.as_ref()]
            .into_iter()
            .flatten()
            .any(|voice| voice.transport.is_form_held())
    }

    /// The section the playing score is sounding now.
    pub fn current_section(&self) -> Option<&str> {
        let active = self.active.as_ref()?;
        active
            .transport
            .playback_at(active.tick)
            .into_iter()
            .flatten()
            .last()
            .map(|part| part.section)
    }

    /// The playing score's clock, in ticks.
    pub fn current_tick(&self) -> Option<u32> {
        self.active.as_ref().map(|voice| voice.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_suspense_arrangement, SuspenseArrangement, SuspenseInput, SuspenseStyle};

    const RATE: f32 = 48000.0;

    fn suspense(seed: &str) -> PortableScore {
        generate_suspense_arrangement(
            &SuspenseInput {
                secret: "live".into(),
                seed: seed.into(),
                style: SuspenseStyle::Terminal,
                tension: 0.5,
                heat: 0.5,
                mystery: 0.5,
                pulse: 0.5,
            },
            SuspenseArrangement::Seeded,
        )
        .unwrap()
    }

    fn play(player: &mut LivePlayer, seconds: f32) -> Vec<f32> {
        let mut out = vec![0.0; (RATE * seconds) as usize];
        for chunk in out.chunks_mut(512) {
            player.fill(chunk);
        }
        out
    }

    #[test]
    fn plays_the_loaded_score_from_its_opening_section() {
        let score = suspense("a");
        let opening = score.default_section.clone();
        let mut player = LivePlayer::new(RATE);
        assert!(!player.is_loaded());
        player.load(score, "a", "suspense", 0, None).unwrap();
        let audio = play(&mut player, 2.0);
        assert!(audio.iter().any(|sample| sample.abs() > 1e-3));
        assert_eq!(player.current_section(), Some(opening.as_str()));
        assert!(player.current_tick().unwrap() > 0);
    }

    #[test]
    fn a_new_seed_waits_for_the_bar_and_keeps_the_tempo() {
        let mut player = LivePlayer::new(RATE);
        player.load(suspense("a"), "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.3);
        let tempo = player.active.as_ref().unwrap().score.bpm;
        player.load(suspense("b"), "b", "suspense", 0, None).unwrap();
        assert!(player.is_playing_seed("a"), "the new seed waits for the bar");
        // Longer than one bar at any Suspense tempo.
        play(&mut player, 4.0);
        assert!(player.is_playing_seed("b"));
        assert_eq!(player.active.as_ref().unwrap().score.bpm, tempo);
    }

    #[test]
    fn rejects_an_unknown_opening_section() {
        let mut player = LivePlayer::new(RATE);
        let error = player
            .load(suspense("a"), "a", "suspense", 0, Some("nowhere"))
            .unwrap_err();
        assert_eq!(error, "Unknown opening section: nowhere");
        assert!(!player.is_loaded());
    }
}
