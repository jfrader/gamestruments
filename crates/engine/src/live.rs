//! The live player every binding drives: it plays a generated score in real
//! time, routes cues and game state to the transport, and blends to new music
//! on the bar. Godot and the browser Lab both play through it, so they sound
//! and queue the same.
//!
//! One blend runs at a time, a section blend or a music blend. A request made
//! while one runs waits, and only the latest request of each kind is kept: one
//! music change and one section. When both are waiting they become one blend,
//! the new music opening on that section. The clock keeps counting through a
//! music blend.

use serde::{Deserialize, Serialize};

use crate::adventure::select_adventure_section;
use crate::handoff::{crossfade_gains, crossfade_sample_count, Handoff};
use crate::master::{MasterChain, MasterConfig};
use crate::score::{AdventureState, GameState, PortableScore, TraceState};
use crate::suspense::select_trace_section;
use crate::synth::{events_starting_at, sample_at_tick, tick_at_sample, Solo, Synth};
use crate::transport::{select_section, TransitionPlan};
use crate::{AdaptiveTransport, FormAudio};

/// A game-state update, in the shape of the recipe that is playing.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameUpdate {
    Race(GameState),
    Trace(TraceState),
    Adventure(AdventureState),
}

/// What the player is sounding, for a UI to show.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatus {
    /// The playing score; new music waiting for its turn is not shown yet.
    pub score_id: String,
    pub tick: u32,
    pub current_section: String,
    /// The section waiting for the running blend, or for new music to start.
    pub pending_section: Option<String>,
    /// The music waiting for the running blend and the next bar.
    pub pending_score_id: Option<String>,
    pub transition: Option<TransitionPlan>,
    pub form_held: bool,
    pub next_form_section: Option<String>,
    /// Every section sounding now, with its gain.
    pub mix: Vec<SectionGain>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SectionGain {
    pub section: String,
    pub gain: f32,
    /// The tick the section's own phrase started at.
    pub origin: u32,
}

/// A request to move to another section.
#[derive(Clone, Debug)]
enum SectionRequest {
    Cue(String),
    Update(GameUpdate),
}

impl SectionRequest {
    /// The section this request selects in `score`.
    fn section_in(&self, score: &PortableScore) -> Option<String> {
        match self {
            SectionRequest::Cue(section) => score.section(section).map(|_| section.clone()),
            SectionRequest::Update(update) => section_for_update(score, update),
        }
    }

    fn apply(&self, transport: &mut AdaptiveTransport, tick: u32) {
        match self {
            SectionRequest::Cue(section) => {
                transport.request_section(section, tick);
            }
            SectionRequest::Update(GameUpdate::Race(state)) => {
                transport.request_state(state, tick);
            }
            SectionRequest::Update(GameUpdate::Trace(state)) => {
                transport.request_trace_state(state, tick);
            }
            SectionRequest::Update(GameUpdate::Adventure(state)) => {
                transport.request_adventure_state(state, tick);
            }
        }
    }
}

/// The section `update` selects in `score`.
fn section_for_update(score: &PortableScore, update: &GameUpdate) -> Option<String> {
    match update {
        GameUpdate::Race(state) => Some(select_section(score, state)),
        GameUpdate::Trace(state) => {
            select_trace_section(&score.rules, state).map(|(section, _)| section)
        }
        GameUpdate::Adventure(state) => Some(select_adventure_section(state).to_string()),
    }
}

/// New music waiting for its turn.
struct PendingMusic {
    score: PortableScore,
    seed: String,
    recipe: String,
    root_pitch_class: i32,
    opening_section: String,
}

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
    /// A voice for `music` whose clock starts at `start_tick`.
    fn new(music: PendingMusic, start_tick: u32, sample_rate: f32) -> Result<Self, String> {
        let PendingMusic {
            score,
            seed,
            recipe,
            root_pitch_class,
            opening_section,
        } = music;
        let mut transport = AdaptiveTransport::new(score.clone(), Some(&opening_section))?;
        transport.start_at(start_tick);
        let ticks_per_second = score.ticks_per_second();
        let start_frame = sample_at_tick(start_tick, f64::from(sample_rate), ticks_per_second);
        // Any score with a form blends through the aligned renderer.
        let form_audio = score.form.is_some().then(|| {
            let mut form_audio = FormAudio::new(&score, sample_rate);
            form_audio.start_at_frame(start_frame);
            form_audio
        });
        Ok(Self {
            ticks_per_second,
            score,
            transport,
            synth: Synth::new(sample_rate),
            form_audio,
            master: MasterChain::new(sample_rate as u32, MasterConfig::default()),
            tick: start_tick,
            frames_produced: start_frame,
            seed,
            root_pitch_class,
            recipe,
            scratch: Vec::new(),
        })
    }

    fn set_solo(&mut self, solo: &Solo) {
        self.synth.set_solo(solo);
        if let Some(form_audio) = self.form_audio.as_mut() {
            form_audio.set_solo(solo);
        }
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
    /// The voice fading out during a music blend.
    outgoing: Option<Voice>,
    /// Crossfade progress between the outgoing and the active voice.
    handoff: Handoff,
    /// The latest music change, waiting for the running blend and the bar.
    pending_music: Option<PendingMusic>,
    /// The latest section request, held while music waits or blends. With no
    /// music in the way, section requests go straight to the transport, which
    /// keeps its own latest one behind a running section blend.
    pending_section: Option<SectionRequest>,
    /// Which voices every score lets through.
    solo: Solo,
}

impl LivePlayer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            active: None,
            outgoing: None,
            handoff: Handoff::idle(),
            pending_music: None,
            pending_section: None,
            solo: Solo::Full,
        }
    }

    /// Audition part of the mix: only the voices `solo` names keep sounding.
    pub fn set_solo(&mut self, solo: Solo) {
        for voice in [&mut self.active, &mut self.outgoing].into_iter().flatten() {
            voice.set_solo(&solo);
        }
        self.solo = solo;
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

    /// Start playing `score`, or, when music is already playing, make it the
    /// next music: it waits for a running blend and the next bar, and replaces
    /// any music still waiting. A replacement of the same recipe inherits the
    /// playing score's key and tempo, so it blends at the seam instead of
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
        let opening_section = opening_section
            .unwrap_or(&score.default_section)
            .to_string();
        if score.section(&opening_section).is_none() {
            return Err(format!("Unknown opening section: {opening_section}"));
        }
        let carried = self
            .active
            .as_ref()
            .filter(|voice| voice.recipe == recipe)
            .map(|voice| (voice.root_pitch_class, voice.score.bpm));
        if let Some((key, bpm)) = carried {
            score.transpose((key - root_pitch_class).rem_euclid(12));
            score.bpm = bpm;
        }
        // Record the key the score actually sounds in, so a later change
        // carries from the sounding key rather than the generated one.
        let music = PendingMusic {
            score,
            seed: seed.to_string(),
            recipe: recipe.to_string(),
            root_pitch_class: carried.map_or(root_pitch_class, |(key, _)| key),
            opening_section,
        };
        let Some(active) = self.active.as_mut() else {
            let mut voice = Voice::new(music, 0, self.sample_rate)?;
            voice.set_solo(&self.solo);
            self.active = Some(voice);
            self.handoff.reset();
            return Ok(());
        };
        // A section the playing music was about to move to now waits for the
        // new music instead, so the two become one blend.
        if self.pending_section.is_none() {
            self.pending_section = active
                .transport
                .requested_section(active.tick)
                .map(|section| SectionRequest::Cue(section.to_string()));
        }
        active.transport.cancel_pending(active.tick);
        self.pending_music = Some(music);
        Ok(())
    }

    /// Render the next `out.len()` mono samples.
    pub fn fill(&mut self, out: &mut [f32]) {
        let frames = out.len();
        let sample_rate = f64::from(self.sample_rate);
        out.fill(0.0);
        self.start_pending_music(frames, sample_rate);
        let Some(active) = self.active.as_mut() else {
            return;
        };

        let Some(outgoing) = self.outgoing.as_mut() else {
            active.fill(frames, sample_rate);
            out.copy_from_slice(&active.scratch);
            active.frames_produced = active.frames_produced.saturating_add(frames as u64);
            active.master.process(out);
            return;
        };

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
            self.release_pending_section();
        }
    }

    /// Start the waiting music once no blend is running and the playing score
    /// reaches its next bar within the coming `frames`. It opens on the
    /// waiting section, if any, at the same point in the bar the playing score
    /// has reached.
    fn start_pending_music(&mut self, frames: usize, sample_rate: f64) {
        let (Some(_), Some(active), None) = (
            self.pending_music.as_ref(),
            self.active.as_ref(),
            self.outgoing.as_ref(),
        ) else {
            return;
        };
        if active.transport.is_blending(active.tick) {
            return;
        }
        let bar_ticks = active.score.bar_ticks();
        let next_tick = tick_at_sample(
            active.frames_produced + frames as u64,
            sample_rate,
            active.ticks_per_second,
        );
        if next_tick / bar_ticks <= active.tick / bar_ticks && active.tick != 0 {
            return;
        }
        let Some(mut music) = self.pending_music.take() else {
            return;
        };
        if let Some(section) = self
            .pending_section
            .take()
            .and_then(|request| request.section_in(&music.score))
        {
            music.opening_section = section;
        }
        let start_tick = (u64::from(active.tick) * u64::from(music.score.bar_ticks())
            / u64::from(bar_ticks)) as u32;
        let held = active.transport.is_form_held();
        let Ok(mut voice) = Voice::new(music, start_tick, self.sample_rate) else {
            return;
        };
        voice.set_solo(&self.solo);
        if held {
            voice.transport.set_form_held(true, start_tick);
        }
        let mut outgoing = self.active.replace(voice).expect("a playing voice");
        // Nothing the old music had planned starts now.
        outgoing.transport.cancel_pending(outgoing.tick);
        self.handoff.begin(crossfade_sample_count(
            &outgoing.score,
            self.sample_rate as u32,
        ));
        self.outgoing = Some(outgoing);
    }

    /// Hand the held section request to the playing music once nothing is in
    /// its way.
    fn release_pending_section(&mut self) {
        if self.pending_music.is_some() {
            return;
        }
        if let (Some(request), Some(active)) = (self.pending_section.take(), self.active.as_mut()) {
            request.apply(&mut active.transport, active.tick);
        }
    }

    /// Whether section requests wait in the player: music is waiting or
    /// blending.
    fn holds_sections(&self) -> bool {
        self.pending_music.is_some() || self.outgoing.is_some()
    }

    /// Route a section request: hold it while music waits or blends, otherwise
    /// hand it to the playing score's transport.
    fn request_section(&mut self, request: SectionRequest) -> bool {
        if self.active.is_none() {
            return false;
        }
        if self.holds_sections() {
            self.pending_section = Some(request);
            return true;
        }
        let active = self.active.as_mut().expect("a playing voice");
        request.apply(&mut active.transport, active.tick);
        true
    }

    /// Whether the playing score, or the music waiting, has `section`.
    pub fn has_section(&self, section: &str) -> bool {
        self.active
            .as_ref()
            .map(|voice| &voice.score)
            .into_iter()
            .chain(self.pending_music.as_ref().map(|music| &music.score))
            .any(|score| score.section(section).is_some())
    }

    pub fn cue_section(&mut self, section: &str) -> bool {
        self.request_section(SectionRequest::Cue(section.to_string()))
    }

    /// Send a game-state update to the music it will play on.
    pub fn request(&mut self, update: &GameUpdate) -> bool {
        self.request_section(SectionRequest::Update(update.clone()))
    }

    /// The section `update` selects in the playing score, without moving to it.
    pub fn section_for(&self, update: &GameUpdate) -> Option<String> {
        section_for_update(&self.active.as_ref()?.score, update)
    }

    /// Drop the waiting section, and a section blend that has not started yet.
    pub fn cancel_pending(&mut self) -> bool {
        let held = self.pending_section.take().is_some();
        let Some(active) = self.active.as_mut() else {
            return held;
        };
        let queued = active.transport.requested_section(active.tick).is_some();
        active.transport.cancel_pending(active.tick);
        held || queued
    }

    pub fn set_form_held(&mut self, held: bool) -> bool {
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        if !active.transport.has_form() {
            return false;
        }
        active.transport.set_form_held(held, active.tick);
        true
    }

    pub fn advance_form(&mut self) -> bool {
        let Some(active) = self.active.as_ref() else {
            return false;
        };
        let Some(next) = active.transport.next_form_section(active.tick) else {
            return false;
        };
        if self.holds_sections() {
            self.pending_section = Some(SectionRequest::Cue(next));
            return true;
        }
        let active = self.active.as_mut().expect("a playing voice");
        active.transport.advance_form(active.tick);
        true
    }

    pub fn is_form_held(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|voice| voice.transport.is_form_held())
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

    /// What the playing score is sounding now.
    pub fn status(&self) -> Option<PlaybackStatus> {
        let voice = self.active.as_ref()?;
        let transport = &voice.transport;
        let pending_score = self
            .pending_music
            .as_ref()
            .map_or(&voice.score, |music| &music.score);
        let pending_section = match &self.pending_section {
            Some(request) => request.section_in(pending_score),
            None => transport.pending_section().map(str::to_string),
        };
        Some(PlaybackStatus {
            score_id: voice.score.id.clone(),
            tick: voice.tick,
            current_section: transport.current_section().to_string(),
            pending_section,
            pending_score_id: self
                .pending_music
                .as_ref()
                .map(|music| music.score.id.clone()),
            transition: transport.transition(),
            form_held: transport.is_form_held(),
            next_form_section: transport.next_form_section(voice.tick),
            mix: transport
                .playback_at(voice.tick)
                .into_iter()
                .flatten()
                .map(|part| SectionGain {
                    section: part.section.to_string(),
                    gain: part.gain,
                    origin: part.origin,
                })
                .collect(),
        })
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
        player
            .load(suspense("a"), "a", "suspense", 0, None)
            .unwrap();
        play(&mut player, 0.3);
        let tempo = player.active.as_ref().unwrap().score.bpm;
        player
            .load(suspense("b"), "b", "suspense", 0, None)
            .unwrap();
        assert!(
            player.is_playing_seed("a"),
            "the new seed waits for the bar"
        );
        // Longer than one bar at any Suspense tempo.
        play(&mut player, 4.0);
        assert!(player.is_playing_seed("b"));
        assert_eq!(player.active.as_ref().unwrap().score.bpm, tempo);
    }

    #[test]
    fn status_shows_the_sounding_section_and_a_cancellable_cue() {
        let score = suspense("a");
        let id = score.id.clone();
        let opening = score.default_section.clone();
        let target = score
            .sections
            .iter()
            .map(|section| section.id.clone())
            .find(|section| *section != opening)
            .unwrap();
        let mut player = LivePlayer::new(RATE);
        assert!(player.status().is_none());
        player.load(score, "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.5);
        let status = player.status().unwrap();
        assert_eq!(status.score_id, id);
        assert_eq!(status.current_section, opening);
        assert!(status.transition.is_none());
        assert_eq!(status.mix.len(), 1);
        assert_eq!(status.mix[0].gain, 1.0);

        assert!(player.cue_section(&target));
        assert_eq!(player.status().unwrap().transition.unwrap().to, target);
        assert!(player.cancel_pending());
        assert!(player.status().unwrap().transition.is_none());
    }

    /// Render one buffer at a time until `done` holds, at most `seconds`.
    fn play_until(
        player: &mut LivePlayer,
        seconds: f32,
        mut done: impl FnMut(&LivePlayer) -> bool,
    ) -> bool {
        let mut buffer = [0.0; 512];
        for _ in 0..(RATE * seconds / 512.0) as usize {
            if done(player) {
                return true;
            }
            player.fill(&mut buffer);
        }
        done(player)
    }

    /// Sections both scores have, other than `except`.
    fn shared_sections(a: &PortableScore, b: &PortableScore, except: &str) -> Vec<String> {
        a.sections
            .iter()
            .map(|section| section.id.clone())
            .filter(|id| id != except && b.section(id).is_some())
            .collect()
    }

    fn music_blending(player: &LivePlayer) -> bool {
        player.outgoing.is_some()
    }

    fn section_blending(player: &LivePlayer) -> bool {
        player
            .active
            .as_ref()
            .is_some_and(|voice| voice.transport.is_blending(voice.tick))
    }

    #[test]
    fn new_music_waits_for_a_running_section_blend() {
        let score = suspense("a");
        let target = shared_sections(&score, &suspense("b"), &score.default_section)[0].clone();
        let mut player = LivePlayer::new(RATE);
        player.load(score, "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.3);
        player.cue_section(&target);
        assert!(play_until(&mut player, 10.0, section_blending));

        // Like the Lab, ask the new music to carry on in the sounding section.
        player
            .load(suspense("b"), "b", "suspense", 0, Some(&target))
            .unwrap();
        assert!(player.status().unwrap().pending_score_id.is_some());
        let mut overlapped = false;
        let started = play_until(&mut player, 30.0, |player| {
            overlapped |= music_blending(player) && section_blending(player);
            player.is_playing_seed("b")
        });
        assert!(started, "the new music starts once the section blend ends");
        assert!(
            !overlapped,
            "a music blend never runs on top of a section blend"
        );
        assert_eq!(player.current_section(), Some(target.as_str()));
    }

    #[test]
    fn a_waiting_music_change_and_section_become_one_blend() {
        let (a, b) = (suspense("a"), suspense("b"));
        let sections = shared_sections(&a, &b, &a.default_section);
        let (first_cue, latest_cue) = (sections[0].clone(), sections[1].clone());
        let mut player = LivePlayer::new(RATE);
        player.load(a, "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.3);
        // Within one bar: a cue, new music, then another cue. The latest cue wins.
        player.cue_section(&first_cue);
        player.load(b, "b", "suspense", 0, None).unwrap();
        player.cue_section(&latest_cue);
        let status = player.status().unwrap();
        assert_eq!(status.pending_section.as_deref(), Some(latest_cue.as_str()));
        assert!(
            status.transition.is_none(),
            "the old music does not blend first"
        );

        assert!(play_until(&mut player, 10.0, |player| player.is_playing_seed("b")));
        assert!(music_blending(&player));
        assert_eq!(player.current_section(), Some(latest_cue.as_str()));
        let mut section_blend = false;
        play_until(&mut player, 30.0, |player| {
            section_blend |= section_blending(player);
            !music_blending(player)
        });
        assert!(
            !section_blend,
            "the new music opens on the section: one blend"
        );
        assert_eq!(player.current_section(), Some(latest_cue.as_str()));
    }

    #[test]
    fn the_clock_keeps_counting_through_new_music() {
        let mut player = LivePlayer::new(RATE);
        player
            .load(suspense("a"), "a", "suspense", 0, None)
            .unwrap();
        play(&mut player, 3.0);
        let before = player.current_tick().unwrap();
        player
            .load(suspense("b"), "b", "suspense", 0, None)
            .unwrap();
        assert!(play_until(&mut player, 10.0, |player| player.is_playing_seed("b")));
        assert!(player.current_tick().unwrap() >= before);
    }

    #[test]
    fn a_section_requested_during_a_music_blend_waits_for_it() {
        let (a, b) = (suspense("a"), suspense("b"));
        let target = shared_sections(&a, &b, &b.default_section)[0].clone();
        let mut player = LivePlayer::new(RATE);
        player.load(a, "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.3);
        player.load(b, "b", "suspense", 0, None).unwrap();
        assert!(play_until(&mut player, 10.0, music_blending));

        assert!(player.cue_section(&target));
        let status = player.status().unwrap();
        assert_eq!(status.pending_section.as_deref(), Some(target.as_str()));
        assert!(status.transition.is_none());

        assert!(play_until(&mut player, 30.0, |player| !music_blending(
            player
        )));
        assert_eq!(
            player.status().unwrap().transition.map(|plan| plan.to),
            Some(target)
        );
    }

    #[test]
    fn only_the_latest_music_change_plays() {
        let mut player = LivePlayer::new(RATE);
        player
            .load(suspense("a"), "a", "suspense", 0, None)
            .unwrap();
        play(&mut player, 0.3);
        player
            .load(suspense("b"), "b", "suspense", 0, None)
            .unwrap();
        player
            .load(suspense("c"), "c", "suspense", 0, None)
            .unwrap();
        let mut heard_b = false;
        assert!(play_until(&mut player, 30.0, |player| {
            heard_b |= player.is_playing_seed("b");
            player.is_playing_seed("c") && !music_blending(player)
        }));
        assert!(!heard_b);
    }

    #[test]
    fn cancel_drops_a_section_waiting_for_new_music() {
        let (a, b) = (suspense("a"), suspense("b"));
        let target = shared_sections(&a, &b, &b.default_section)[0].clone();
        let mut player = LivePlayer::new(RATE);
        player.load(a, "a", "suspense", 0, None).unwrap();
        play(&mut player, 0.3);
        player.load(b, "b", "suspense", 0, None).unwrap();
        player.cue_section(&target);
        assert!(player.cancel_pending());
        assert!(player.status().unwrap().pending_section.is_none());
        assert!(play_until(&mut player, 10.0, |player| player.is_playing_seed("b")));
        assert_eq!(
            player.current_section(),
            Some(
                player
                    .active
                    .as_ref()
                    .unwrap()
                    .score
                    .default_section
                    .as_str()
            )
        );
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
