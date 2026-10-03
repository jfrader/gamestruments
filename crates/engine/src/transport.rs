use serde::Serialize;

use crate::handoff::RAW_MUSICAL_FLOOR;
use crate::score::{AdventureState, FormOrigin, GameState, PortableScore, TraceState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionPlan {
    pub from: String,
    pub to: String,
    pub start_tick: u32,
    pub end_tick: u32,
    /// Whether this transition holds the outgoing at full gain until the
    /// incoming section's rendered level reaches the musical floor (a game
    /// section cue), rather than crossfading on the authored schedule (an
    /// automatic form transition).
    pub hold: bool,
}

/// Why the active transition is running. An authoritative game-state update
/// supersedes a form transition that is still in flight — the transport's
/// autonomous progression (automatic or a manual [`AdaptiveTransport::advance_form`])
/// is stale once the game reports a new current state — while a held cue is
/// never dropped mid-hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransitionSource {
    /// Automatic form progression (`maybe_advance_form`).
    Automatic,
    /// A manual form step (`advance_form`).
    Form,
    /// An explicit request: a game-state update (`set_race_state`,
    /// `set_trace_state`, `set_adventure_state`) or a direct section cue
    /// (`cue_section`).
    Explicit,
}

impl TransitionSource {
    /// Whether this source follows the song form rather than answering an
    /// explicit request, and is therefore superseded by a later game-state
    /// update.
    fn is_form(self) -> bool {
        matches!(self, TransitionSource::Automatic | TransitionSource::Form)
    }
}

pub struct SectionPlayback<'a> {
    pub section: &'a str,
    pub origin: u32,
    pub gain: f32,
    pub drum_gain: f32,
    pub percussion: bool,
}

pub struct AdaptiveTransport {
    score: PortableScore,
    bar_ticks: u32,
    current_section: String,
    pending_section: Option<String>,
    transition: Option<TransitionPlan>,
    form_step_index: usize,
    /// Where the form counts the current section's length from.
    section_entered_at: u32,
    /// The tick the current section's phrase started at: where it began
    /// fading in. A finished blend or a form repeat moves the form's count
    /// (`section_entered_at`), never this, so the renderer never restarts a
    /// section that is still sounding and cuts its ringing notes.
    phrase_origin: u32,
    cue_target: Option<String>,
    form_held: bool,
    form_not_before: u32,
    transition_source: TransitionSource,
    /// The tick at which the active transition's crossfade actually began:
    /// `None` while the crossfade is holding (the outgoing at full gain and the
    /// incoming silent until the incoming section's rendered level reaches
    /// [`MUSICAL_FLOOR`]), otherwise the tick the hold released. Reset whenever
    /// the transition is replaced or cleared.
    release_tick: Option<u32>,
    /// Whether the renderer has reported the incoming section's level during
    /// this transition. Until it has (a structural simulation that never
    /// renders), a held cue does not extend: it still completes at the plan's
    /// authored end.
    incoming_reported: bool,
}

impl AdaptiveTransport {
    pub fn new(score: PortableScore, initial_section: Option<&str>) -> Result<Self, String> {
        let initial = initial_section
            .unwrap_or(score.default_section.as_str())
            .to_string();
        if score.section(&initial).is_none() {
            return Err(format!("Unknown initial section: {initial}"));
        }
        let bar_ticks = score.bar_ticks();
        let form_step_index = form_index_for(&score, &initial).unwrap_or(0);
        Ok(Self {
            score,
            bar_ticks,
            current_section: initial,
            pending_section: None,
            transition: None,
            form_step_index,
            section_entered_at: 0,
            phrase_origin: 0,
            cue_target: None,
            form_held: false,
            form_not_before: 0,
            transition_source: TransitionSource::Explicit,
            release_tick: None,
            incoming_reported: false,
        })
    }

    pub fn has_form(&self) -> bool {
        self.score.form.is_some()
    }

    pub fn form_step_index(&self) -> usize {
        self.form_step_index
    }

    pub fn phrase_tick(&self, at_tick: u32) -> u32 {
        if self.score.form.is_none() {
            let length = self
                .score
                .section(&self.current_section)
                .map(|section| section.length_ticks)
                .unwrap_or(1)
                .max(1);
            return at_tick % length;
        }
        at_tick.saturating_sub(self.section_entered_at)
    }

    pub fn current_section(&self) -> &str {
        &self.current_section
    }

    pub fn request_state(&mut self, state: &GameState, at_tick: u32) -> Option<TransitionPlan> {
        let target = select_section(&self.score, state);
        self.request_section_as(&target, at_tick, TransitionSource::Explicit, true)
    }

    pub fn request_trace_state(
        &mut self,
        state: &TraceState,
        at_tick: u32,
    ) -> Option<TransitionPlan> {
        self.advance(at_tick);
        let Some((section, hold)) = crate::suspense::select_trace_section(&self.score.rules, state)
        else {
            self.cue_target = None;
            return None;
        };
        if hold {
            self.cue_target = None;
            return self.request_section_as(&section, at_tick, TransitionSource::Explicit, true);
        }
        let already_cued = self.cue_target.as_deref() == Some(section.as_str())
            && self.current_section == section
            && self.transition.is_none();
        if already_cued {
            return None;
        }
        self.cue_target = Some(section.clone());
        self.request_section_as(&section, at_tick, TransitionSource::Explicit, true)
    }

    pub fn request_adventure_state(
        &mut self,
        state: &AdventureState,
        at_tick: u32,
    ) -> Option<TransitionPlan> {
        let target = crate::adventure::select_adventure_section(state);
        self.request_section_as(target, at_tick, TransitionSource::Explicit, true)
    }

    /// A direct section cue (`cue_section`): defers to whatever transition is
    /// already in flight, so a held cue is never dropped mid-hold.
    pub fn request_section(&mut self, target: &str, at_tick: u32) -> Option<TransitionPlan> {
        self.request_section_as(target, at_tick, TransitionSource::Explicit, false)
    }

    /// Route a section request. An authoritative game-state update
    /// (`supersede_form`) replaces a form transition that is still in flight —
    /// the transport's autonomous progression is stale once the game reports a
    /// new current state — while a direct cue defers to it. Either way the new
    /// transition still holds the outgoing at full gain until the incoming
    /// clears the musical floor, so nothing hard-cuts or dips to silence.
    fn request_section_as(
        &mut self,
        target: &str,
        at_tick: u32,
        source: TransitionSource,
        supersede_form: bool,
    ) -> Option<TransitionPlan> {
        self.score.section(target)?;
        self.advance(at_tick);
        if target == self.current_section
            && self.transition.is_none()
            && self.pending_section.is_none()
        {
            return None;
        }
        if let Some(plan) = &self.transition {
            if plan.to == target {
                self.pending_section = None;
                return None;
            }
            let supersede = supersede_form && self.transition_source.is_form();
            if at_tick < plan.start_tick || supersede {
                // The transition has not begun (or is a stale form transition):
                // replace it outright instead of queueing the request behind it.
                // A queued cue was relative to the transition being replaced, so
                // it is dropped rather than resurfacing after the new one lands.
                if target == self.current_section {
                    self.clear_transition();
                    self.pending_section = None;
                    self.sync_form_to(&self.current_section.clone());
                    return None;
                }
                self.pending_section = None;
                let next = self.create_plan(&self.current_section, target, at_tick);
                self.begin_transition(next.clone(), source);
                return Some(next);
            }
            self.pending_section = Some(target.to_string());
            return None;
        }
        let plan = self.create_plan(&self.current_section, target, at_tick);
        self.begin_transition(plan.clone(), source);
        Some(plan)
    }

    /// Drop a queued cue, and the active transition if it has not started yet.
    /// Returns the cancelled transition.
    pub fn cancel_pending(&mut self, at_tick: u32) -> Option<TransitionPlan> {
        self.pending_section = None;
        if self
            .transition
            .as_ref()
            .is_none_or(|plan| at_tick >= plan.start_tick)
        {
            return None;
        }
        let plan = self.clear_transition();
        self.sync_form_to(&self.current_section.clone());
        self.cue_target = None;
        plan
    }

    /// The section queued behind the active transition.
    pub fn pending_section(&self) -> Option<&str> {
        self.pending_section.as_deref()
    }

    /// The active transition, ending where it really ends: a held cue runs
    /// past its authored end while the incoming section is still quiet.
    pub fn transition(&self) -> Option<TransitionPlan> {
        self.transition.as_ref().map(|plan| TransitionPlan {
            end_tick: self.effective_end(plan),
            ..plan.clone()
        })
    }

    pub fn advance(&mut self, at_tick: u32) {
        if let Some(plan) = &self.transition {
            // A held cue whose incoming never reaches the floor still must
            // finish: release the hold at the plan's authored end. Only once
            // the renderer has observed the incoming, so a structural
            // simulation still completes at the authored end.
            if plan.hold
                && self.incoming_reported
                && self.release_tick.is_none()
                && at_tick >= plan.end_tick
            {
                self.release_tick = Some(plan.end_tick);
            }
            if at_tick >= self.effective_end(plan) {
                self.current_section = plan.to.clone();
                self.cue_target = None;
                self.section_entered_at = if self.score.form.as_ref().and_then(|form| form.origin)
                    == Some(FormOrigin::TransitionStart)
                {
                    plan.start_tick
                } else if self.score.form.is_some() {
                    plan.end_tick
                } else {
                    at_tick
                };
                self.phrase_origin = plan.start_tick;
                self.sync_form_to(&self.current_section.clone());
                self.clear_transition();
                self.form_not_before = 0;
                if let Some(pending) = self.pending_section.take() {
                    if pending != self.current_section {
                        let next = self.create_plan(&self.current_section, &pending, at_tick);
                        self.begin_transition(next, TransitionSource::Explicit);
                    }
                }
            }
        }
        self.maybe_advance_form(at_tick);
    }

    pub fn section_gain(&self, section_id: &str, at_tick: u32) -> f32 {
        if let Some(plan) = &self.transition {
            if at_tick >= plan.start_tick && at_tick < self.effective_end(plan) {
                let progress = self.fade_progress(plan, at_tick);
                if section_id == plan.from {
                    return 1.0 - progress;
                }
                if section_id == plan.to {
                    return progress;
                }
                return 0.0;
            }
        }
        if section_id == self.current_section {
            1.0
        } else {
            0.0
        }
    }

    pub fn is_form_held(&self) -> bool {
        self.form_held
    }

    pub fn set_form_held(&mut self, held: bool, at_tick: u32) -> Option<TransitionPlan> {
        if self.score.form.is_none() || self.form_held == held {
            return None;
        }
        self.form_held = held;
        if !held {
            self.form_not_before = at_tick;
        }
        if held
            && self.transition_source == TransitionSource::Automatic
            && self
                .transition
                .as_ref()
                .is_some_and(|plan| at_tick < plan.start_tick)
        {
            let plan = self.clear_transition();
            self.pending_section = None;
            self.sync_form_to(&self.current_section.clone());
            return plan;
        }
        None
    }

    pub fn next_form_section(&self, at_tick: u32) -> Option<String> {
        let form = self.score.form.as_ref()?;
        let current = self
            .transition
            .as_ref()
            .filter(|plan| at_tick >= plan.start_tick)
            .map_or(self.current_section.as_str(), |plan| plan.to.as_str());
        let index = form_index_from(&self.score, current, self.form_step_index)?;
        let mut next = index;
        for _ in 0..form.steps.len() {
            next = if next + 1 < form.steps.len() {
                next + 1
            } else {
                form.loop_from? as usize
            };
            let target = &form.steps.get(next)?.section;
            if target != current {
                return Some(target.clone());
            }
        }
        None
    }

    pub fn advance_form(&mut self, at_tick: u32) -> Option<TransitionPlan> {
        let target = self.next_form_section(at_tick)?;
        self.request_section_as(&target, at_tick, TransitionSource::Form, false)
    }

    pub fn playback_at(&self, at_tick: u32) -> [Option<SectionPlayback<'_>>; 2] {
        if let Some(plan) = &self.transition {
            if at_tick >= self.effective_end(plan) {
                return [
                    Some(SectionPlayback {
                        section: &plan.to,
                        origin: plan.start_tick,
                        gain: 1.0,
                        drum_gain: 1.0,
                        percussion: true,
                    }),
                    None,
                ];
            }
            if at_tick >= plan.start_tick {
                let progress = self.fade_progress(plan, at_tick);
                let (gain_from, gain_to) = (1.0 - progress, progress);
                return [
                    Some(SectionPlayback {
                        section: &plan.from,
                        origin: self.phrase_origin,
                        gain: gain_from,
                        drum_gain: if plan.hold { gain_from } else { 1.0 },
                        percussion: plan.hold,
                    }),
                    Some(SectionPlayback {
                        section: &plan.to,
                        origin: plan.start_tick,
                        gain: gain_to,
                        drum_gain: if plan.hold { gain_to } else { 1.0 },
                        percussion: true,
                    }),
                ];
            }
        }
        [
            Some(SectionPlayback {
                section: &self.current_section,
                origin: self.phrase_origin,
                gain: 1.0,
                drum_gain: 1.0,
                percussion: true,
            }),
            None,
        ]
    }

    fn create_plan(&self, from: &str, to: &str, at_tick: u32) -> TransitionPlan {
        let start = at_tick
            .div_ceil(self.bar_ticks)
            .saturating_mul(self.bar_ticks);
        let start = start.max(at_tick);
        TransitionPlan {
            from: from.to_string(),
            to: to.to_string(),
            start_tick: start,
            end_tick: start.saturating_add(self.transition_length()),
            hold: true,
        }
    }

    fn transition_length(&self) -> u32 {
        let length = (self.score.crossfade_bars * f64::from(self.bar_ticks)).round() as u32;
        length.max(self.bar_ticks)
    }

    /// Start a new transition, dropping any in-flight hold.
    fn begin_transition(&mut self, plan: TransitionPlan, source: TransitionSource) {
        self.release_tick = None;
        self.incoming_reported = false;
        self.transition_source = source;
        self.transition = Some(plan);
    }

    /// End the active transition and return it, dropping any in-flight hold.
    fn clear_transition(&mut self) -> Option<TransitionPlan> {
        self.release_tick = None;
        self.incoming_reported = false;
        self.transition_source = TransitionSource::Explicit;
        self.transition.take()
    }

    /// The tick at which the active transition actually finishes. A held cue
    /// whose incoming has been observed by the renderer begins its crossfade at
    /// the release tick (or holds until the plan's authored end) and then runs
    /// the full crossfade length, so it extends past
    /// [`TransitionPlan::end_tick`] when the incoming opens quietly; a
    /// never-sounding incoming still completes (bounded by one crossfade length
    /// past the plan end). A transition the renderer has not observed (or an
    /// automatic form transition) ends at the authored tick.
    fn effective_end(&self, plan: &TransitionPlan) -> u32 {
        if !plan.hold || !self.incoming_reported {
            return plan.end_tick;
        }
        let release = self.release_tick.unwrap_or(plan.end_tick);
        release.saturating_add(self.transition_length())
    }

    fn fade_progress(&self, plan: &TransitionPlan, at_tick: u32) -> f32 {
        if !plan.hold {
            return (at_tick.saturating_sub(plan.start_tick)) as f32
                / self.transition_length().max(1) as f32;
        }
        let Some(release) = self.release_tick else {
            return 0.0;
        };
        (at_tick.saturating_sub(release)) as f32 / self.transition_length().max(1) as f32
    }

    /// Report the incoming section's rendered level (RMS) for the active
    /// transition. The level is the section's pre-master render (see
    /// [`crate::form_audio::FormAudio`]), so it is compared against
    /// [`RAW_MUSICAL_FLOOR`]. While a held cue's incoming has not reached that
    /// floor the crossfade holds at progress zero so the outgoing stays at full
    /// gain; the first report at or above the floor releases the hold and starts
    /// the crossfade at zero. The renderer calls this once per buffer, so the
    /// hold releases within a buffer of the incoming sounding. Automatic form
    /// transitions ignore reports and crossfade on schedule.
    pub fn report_incoming_level(&mut self, level: f32, at_tick: u32) {
        let Some(plan) = &self.transition else {
            return;
        };
        if !plan.hold {
            return;
        }
        self.incoming_reported = true;
        if self.release_tick.is_some() {
            return;
        }
        if at_tick < plan.start_tick {
            return;
        }
        if level >= RAW_MUSICAL_FLOOR {
            self.release_tick = Some(at_tick.min(plan.end_tick));
        }
    }

    fn sync_form_to(&mut self, section: &str) {
        if let Some(index) = form_index_from(&self.score, section, self.form_step_index) {
            self.form_step_index = index;
        }
    }

    fn maybe_advance_form(&mut self, at_tick: u32) {
        if self.form_held || self.transition.is_some() || self.pending_section.is_some() {
            return;
        }
        let Some(form) = self.score.form.as_ref() else {
            return;
        };
        if form.steps.is_empty() {
            return;
        }
        let step = match form.steps.get(self.form_step_index) {
            Some(step) if step.section == self.current_section => step,
            _ => return,
        };
        let Some(section) = self.score.section(&self.current_section) else {
            return;
        };
        let repeats = step.repeats.max(1);
        let cycles = repeats.max(
            self.form_not_before
                .saturating_sub(self.section_entered_at)
                .div_ceil(section.length_ticks),
        );
        let duration = section.length_ticks.saturating_mul(cycles);
        let boundary_tick = self.section_entered_at.saturating_add(duration);
        if at_tick.saturating_sub(self.section_entered_at) < duration {
            return;
        }
        let next_index = if self.form_step_index + 1 < form.steps.len() {
            self.form_step_index + 1
        } else {
            match form.loop_from {
                Some(index) if (index as usize) < form.steps.len() => index as usize,
                _ => return,
            }
        };
        let next_section = form.steps[next_index].section.clone();
        if next_section == self.current_section {
            self.form_step_index = next_index;
            self.section_entered_at = boundary_tick;
            return;
        }
        let length = self.transition_length();
        let plan = TransitionPlan {
            from: self.current_section.clone(),
            to: next_section,
            start_tick: boundary_tick,
            end_tick: boundary_tick.saturating_add(length),
            hold: false,
        };
        self.begin_transition(plan, TransitionSource::Automatic);
    }
}

fn form_index_for(score: &PortableScore, section: &str) -> Option<usize> {
    score
        .form
        .as_ref()?
        .steps
        .iter()
        .position(|step| step.section == section)
}

fn form_index_from(score: &PortableScore, section: &str, current: usize) -> Option<usize> {
    let form = score.form.as_ref()?;
    let steps = &form.steps;
    if steps
        .get(current)
        .is_some_and(|step| step.section == section)
    {
        return Some(current);
    }
    let loop_from = (form.loop_from.unwrap_or(0) as usize).min(current);
    (current + 1..steps.len())
        .chain(loop_from..current.min(steps.len()))
        .chain(0..loop_from)
        .find(|&index| steps[index].section == section)
}

pub fn select_section(score: &PortableScore, state: &GameState) -> String {
    if state.race_phase == "finish" {
        if state.finish_result == "win" {
            return "victory".into();
        }
        // A loss or DNF cues the defeat outro where the score carries one;
        // original/extended keep victory as the only finish.
        if matches!(state.finish_result.as_str(), "loss" | "dnf")
            && score.section("defeat").is_some()
        {
            return "defeat".into();
        }
        return "victory".into();
    }
    // Race incidents: recovery (a reset) and wrong-way. They outrank the
    // intensity and final-lap rules so an explicit incident cue always wins.
    if state.race_phase == "recovery" && score.section("recovery").is_some() {
        return "recovery".into();
    }
    if state.race_phase == "wrong-way" && score.section("wrong-way").is_some() {
        return "wrong-way".into();
    }
    if state.final_lap {
        return "final-lap".into();
    }
    if state.position_pressure >= 0.68 || state.intensity >= 0.72 {
        return "attack".into();
    }
    match state.race_phase.as_str() {
        "race" => "cruise".into(),
        "grid" => "grid".into(),
        "garage" => "garage".into(),
        _ => score.default_section.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{select_section, AdaptiveTransport};
    use crate::adventure::{generate_adventure, AdventureInput, AdventureStyle};
    use crate::arrangement::{apply_automatic_arrangement, ArrangementRecipe};
    use crate::racing::{generate_racing, GenerateInput, InstrumentPalette, Style};
    use crate::racing_arrangement::{generate_racing_arrangement, RacingArrangement};
    use crate::score::{AdventureState, GameState, SongForm, SongFormStep, TraceState};
    use crate::suspense::{generate_suspense, SuspenseInput, SuspenseStyle};

    #[test]
    fn repeated_form_occurrences_reach_the_final_step() {
        let mut generated = composed_score();
        let ids: Vec<String> = generated
            .sections
            .iter()
            .take(3)
            .map(|s| s.id.clone())
            .collect();
        generated.form = Some(SongForm {
            steps: [0, 1, 0, 2]
                .map(|i| SongFormStep {
                    section: ids[i].clone(),
                    repeats: 1,
                })
                .to_vec(),
            loop_from: None,
            origin: Some(crate::score::FormOrigin::TransitionStart),
        });
        let mut transport = AdaptiveTransport::new(generated.clone(), None).unwrap();
        let mut tick = 0;
        for index in [1, 0, 2] {
            tick += generated
                .section(transport.current_section())
                .unwrap()
                .length_ticks;
            transport.advance(tick);
            let plan = transport.transition().unwrap();
            assert_eq!(plan.to, ids[index]);
            tick = plan.end_tick;
            transport.advance(tick);
            assert_eq!(transport.current_section(), ids[index]);
            if index == 0 {
                assert_eq!(
                    transport.next_form_section(tick).as_deref(),
                    Some(ids[2].as_str())
                );
            }
        }
        assert_eq!(transport.next_form_section(tick), None);
    }

    #[test]
    fn repeated_form_loop_returns_to_its_loop_occurrence() {
        let mut generated = composed_score();
        let ids: Vec<String> = generated
            .sections
            .iter()
            .take(3)
            .map(|s| s.id.clone())
            .collect();
        generated.form = Some(SongForm {
            steps: [0, 1, 0, 2]
                .map(|i| SongFormStep {
                    section: ids[i].clone(),
                    repeats: 1,
                })
                .to_vec(),
            loop_from: Some(2),
            origin: Some(crate::score::FormOrigin::TransitionStart),
        });
        for manual in [false, true] {
            let mut transport = AdaptiveTransport::new(generated.clone(), None).unwrap();
            transport.set_form_held(manual, 0);
            let mut tick = 0;
            for index in [1, 0, 2, 0, 2] {
                tick += generated
                    .section(transport.current_section())
                    .unwrap()
                    .length_ticks;
                if manual {
                    transport.advance_form(tick);
                } else {
                    transport.advance(tick);
                }
                let plan = transport.transition().unwrap();
                assert_eq!(plan.to, ids[index]);
                tick = plan.end_tick;
                transport.advance(tick);
                if index == 0 {
                    assert_eq!(
                        transport.next_form_section(tick).as_deref(),
                        Some(ids[2].as_str())
                    );
                }
            }
        }
    }

    #[test]
    fn cancelling_repeated_form_transition_preserves_outgoing_occurrence() {
        let mut generated = composed_score();
        let ids: Vec<String> = generated
            .sections
            .iter()
            .take(3)
            .map(|s| s.id.clone())
            .collect();
        generated.form = Some(SongForm {
            steps: [0, 1, 0, 2]
                .map(|i| SongFormStep {
                    section: ids[i].clone(),
                    repeats: 1,
                })
                .to_vec(),
            loop_from: None,
            origin: Some(crate::score::FormOrigin::TransitionStart),
        });
        let mut transport = AdaptiveTransport::new(generated.clone(), Some(&ids[1])).unwrap();
        transport.set_form_held(true, 0);
        let cue = transport.advance_form(1).unwrap();
        assert_eq!(cue.to, ids[0]);
        transport.cancel_pending(2);
        assert_eq!(transport.current_section(), ids[1]);
        assert_eq!(
            transport.next_form_section(2).as_deref(),
            Some(ids[0].as_str())
        );
        let cue = transport.advance_form(3).unwrap();
        assert_eq!(cue.to, ids[0]);
        transport.advance(cue.end_tick);
        assert_eq!(
            transport.next_form_section(cue.end_tick).as_deref(),
            Some(ids[2].as_str())
        );
    }

    #[test]
    fn manual_next_skips_identical_steps_but_autoplay_counts_them() {
        let mut generated = composed_score();
        let ids: Vec<String> = generated
            .sections
            .iter()
            .take(2)
            .map(|s| s.id.clone())
            .collect();
        generated.form = Some(SongForm {
            steps: [0, 0, 1]
                .map(|i| SongFormStep {
                    section: ids[i].clone(),
                    repeats: 1,
                })
                .to_vec(),
            loop_from: None,
            origin: Some(crate::score::FormOrigin::TransitionStart),
        });
        let mut manual = AdaptiveTransport::new(generated.clone(), None).unwrap();
        assert_eq!(
            manual.next_form_section(0).as_deref(),
            Some(ids[1].as_str())
        );
        let plan = manual.advance_form(1).unwrap();
        manual.advance(plan.end_tick);
        assert_eq!(manual.current_section(), ids[1]);

        let length = generated.section(&ids[0]).unwrap().length_ticks;
        let mut automatic = AdaptiveTransport::new(generated, None).unwrap();
        automatic.advance(length);
        assert_eq!(automatic.current_section(), ids[0]);
        assert!(automatic.transition().is_none());
        automatic.advance(2 * length);
        assert_eq!(automatic.transition().unwrap().to, ids[1]);
    }

    #[test]
    fn folklore_form_reaches_its_cierre() {
        let score = crate::folklore::generate_folklore(
            &crate::folklore::FolkloreInput {
                secret: "form-test".into(),
                seed: "form-test".into(),
                energy: 0.5,
                complexity: 0.5,
                brightness: 0.5,
                syncopation: 0.5,
            },
            "seeded",
        )
        .unwrap();
        let order: Vec<String> = score
            .form
            .as_ref()
            .unwrap()
            .steps
            .iter()
            .map(|step| step.section.clone())
            .collect();
        let mut transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut tick = 0;
        for target in order.iter().skip(1) {
            tick += score
                .section(transport.current_section())
                .unwrap()
                .length_ticks;
            transport.advance(tick);
            let transition = transport.transition().unwrap();
            assert_eq!(&transition.to, target);
            tick = transition.end_tick;
            transport.advance(tick);
            assert_eq!(transport.current_section(), target);
        }
        assert_eq!(transport.current_section(), "cierre");
        let loop_target = score
            .form
            .as_ref()
            .unwrap()
            .loop_from
            .map(|index| order[index as usize].as_str());
        assert_eq!(transport.next_form_section(tick).as_deref(), loop_target);
    }

    fn score() -> crate::score::PortableScore {
        generate_racing(&GenerateInput {
            secret: "qa-secret".into(),
            seed: "qa-race".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.6,
            complexity: 0.5,
            brightness: 0.5,
            syncopation: 0.6,
        })
        .expect("transport test score must validate")
    }

    fn composed_score() -> crate::score::PortableScore {
        generate_racing_arrangement(
            &GenerateInput {
                secret: "qa-secret".into(),
                seed: "qa-race".into(),
                style: Style::Funk,
                palette: InstrumentPalette::default(),
                energy: 0.6,
                complexity: 0.5,
                brightness: 0.5,
                syncopation: 0.6,
            },
            RacingArrangement::Seeded,
        )
        .expect("composed transport test score must validate")
    }

    #[test]
    fn race_state_crosses_into_cruise() {
        let generated = score();
        let bar = generated.bar_ticks();
        let mut transport = AdaptiveTransport::new(generated, None).unwrap();
        assert_eq!(transport.current_section(), "garage");
        transport.request_state(
            &GameState {
                intensity: 0.4,
                position_pressure: 0.2,
                final_lap: false,
                race_phase: "race".into(),
                finish_result: "none".into(),
            },
            0,
        );
        transport.advance(bar * 4);
        assert_eq!(transport.current_section(), "cruise");
    }

    #[test]
    fn a_sounding_section_keeps_its_phrase_origin() {
        // A section's renderer restarts whenever its origin moves, cutting the
        // notes still ringing; that was the glitch heard as a blend finished.
        let score = crate::suspense_arrangement::generate_suspense_arrangement(
            &SuspenseInput {
                secret: "qa-secret".into(),
                seed: "origins".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.5,
                heat: 0.5,
                mystery: 0.5,
                pulse: 0.5,
            },
            crate::suspense_arrangement::SuspenseArrangement::Seeded,
        )
        .unwrap();
        let bar = score.bar_ticks();
        let target = score
            .sections
            .iter()
            .map(|section| section.id.clone())
            .find(|section| *section != score.default_section)
            .unwrap();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        let mut sounding = std::collections::HashMap::new();
        for tick in (0..bar * 64).step_by(24) {
            if tick == bar * 3 + 96 {
                transport.request_section(&target, tick);
            }
            transport.advance(tick);
            transport.report_incoming_level(1.0, tick);
            let now: std::collections::HashMap<String, u32> = transport
                .playback_at(tick)
                .into_iter()
                .flatten()
                .map(|part| (part.section.to_string(), part.origin))
                .collect();
            for (section, origin) in &now {
                if let Some(previous) = sounding.get(section) {
                    assert_eq!(
                        previous, origin,
                        "{section} moved its origin at tick {tick}"
                    );
                }
            }
            sounding = now;
        }
    }

    #[test]
    fn cancel_pending_drops_a_cue_that_has_not_started() {
        let generated = score();
        let bar = generated.bar_ticks();
        let mut transport = AdaptiveTransport::new(generated, None).unwrap();
        transport.request_section("cruise", 1);
        assert_eq!(transport.transition().unwrap().to, "cruise");
        let cancelled = transport.cancel_pending(2).unwrap();
        assert_eq!(cancelled.to, "cruise");
        assert!(transport.transition().is_none());
        transport.advance(bar * 4);
        assert_eq!(transport.current_section(), "garage");
    }

    #[test]
    fn cancel_pending_keeps_a_transition_already_crossing() {
        let generated = score();
        let bar = generated.bar_ticks();
        let mut transport = AdaptiveTransport::new(generated, None).unwrap();
        transport.request_section("cruise", 1);
        transport.request_section("attack", bar + 1);
        assert_eq!(transport.pending_section(), Some("attack"));
        assert!(transport.cancel_pending(bar + 2).is_none());
        assert_eq!(transport.pending_section(), None);
        assert_eq!(transport.transition().unwrap().to, "cruise");
    }

    #[test]
    fn finish_result_selects_victory_on_a_win_and_defeat_on_a_loss() {
        let composed = composed_score();
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "finish".into(),
                    finish_result: "win".into(),
                    ..GameState::default()
                },
            ),
            "victory"
        );
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "finish".into(),
                    finish_result: "loss".into(),
                    ..GameState::default()
                },
            ),
            "defeat"
        );
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "finish".into(),
                    finish_result: "dnf".into(),
                    ..GameState::default()
                },
            ),
            "defeat"
        );
    }

    #[test]
    fn original_score_keeps_victory_for_a_loss() {
        // Original carries no defeat section, so a loss still resolves to
        // victory — the frozen original behaviour is untouched.
        let original = score();
        assert_eq!(
            select_section(
                &original,
                &GameState {
                    race_phase: "finish".into(),
                    finish_result: "loss".into(),
                    ..GameState::default()
                },
            ),
            "victory"
        );
        assert!(original.section("defeat").is_none());
    }

    #[test]
    fn recovery_and_wrong_way_cue_their_phase_and_resolve_back_to_cruise() {
        let composed = composed_score();
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "recovery".into(),
                    finish_result: "none".into(),
                    ..GameState::default()
                },
            ),
            "recovery"
        );
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "wrong-way".into(),
                    finish_result: "none".into(),
                    ..GameState::default()
                },
            ),
            "wrong-way"
        );
        // A subsequent race signal resolves back into the flow groove.
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    race_phase: "race".into(),
                    finish_result: "none".into(),
                    ..GameState::default()
                },
            ),
            "cruise"
        );
    }

    #[test]
    fn recovery_incident_outranks_intensity_and_final_lap() {
        let composed = composed_score();
        // Even under max pressure on the final lap, an explicit recovery cue
        // wins the selection.
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    intensity: 0.9,
                    position_pressure: 0.9,
                    final_lap: true,
                    race_phase: "recovery".into(),
                    finish_result: "none".into(),
                },
            ),
            "recovery"
        );
        assert_eq!(
            select_section(
                &composed,
                &GameState {
                    intensity: 0.9,
                    position_pressure: 0.9,
                    final_lap: true,
                    race_phase: "wrong-way".into(),
                    finish_result: "none".into(),
                },
            ),
            "wrong-way"
        );
    }

    fn suspense_pool(seed: &str) -> crate::score::PortableScore {
        crate::generate_suspense_arrangement(
            &SuspenseInput {
                secret: "qa".into(),
                seed: seed.into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            crate::SuspenseArrangement::AllPhases,
        )
        .unwrap()
    }

    #[test]
    fn game_hold_advances_to_the_next_form_step_and_resumes_on_a_boundary() {
        let score = suspense_pool("game-controls");
        let bar = score.bar_ticks();
        let form = score.form.as_ref().unwrap().steps.clone();
        let length = score.section("verse").unwrap().length_ticks;
        let verse_index = form
            .iter()
            .position(|step| step.section == "verse")
            .expect("the pool form plays verse");
        let expected_next = form[verse_index + 1].section.clone();
        let following = form[verse_index + 2].section.clone();
        let next_len = score.section(&expected_next).unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, Some("verse")).unwrap();
        transport.set_form_held(true, 0);
        transport.advance(5 * length);
        assert_eq!(transport.current_section(), "verse");
        assert!(transport.transition.is_none());
        let plan = transport.advance_form(5 * length + 100).unwrap();
        assert_eq!(plan.to, expected_next);
        assert_eq!(plan.start_tick % bar, 0);
        transport.advance(plan.end_tick);
        assert!(transport.is_form_held());
        assert_eq!(transport.current_section(), expected_next);
        let now = transport.section_entered_at + 100;
        transport.advance(now);
        transport.set_form_held(false, now);
        transport.advance(now);
        assert!(transport.transition.is_none());
        let boundary = transport.section_entered_at
            + (now - transport.section_entered_at).div_ceil(next_len) * next_len;
        transport.advance(boundary - 1);
        assert!(transport.transition.is_none());
        transport.advance(boundary);
        assert_eq!(transport.transition.as_ref().unwrap().start_tick, boundary);
        assert_eq!(transport.transition.as_ref().unwrap().to, following);
    }

    #[test]
    fn holding_during_an_automatic_blend_keeps_the_incoming_section() {
        let score = suspense_pool("hold-blend");
        let form = score.form.as_ref().unwrap().steps.clone();
        let verse_index = form
            .iter()
            .position(|step| step.section == "verse")
            .expect("the pool form plays verse");
        let expected_next = form[verse_index + 1].section.clone();
        let length = score.section("verse").unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, Some("verse")).unwrap();
        transport.advance(length);
        let end = transport.transition.as_ref().unwrap().end_tick;
        assert!(transport.set_form_held(true, length + 1).is_none());
        transport.advance(end);
        transport.advance(10 * length);
        assert_eq!(transport.current_section(), expected_next);
        assert!(transport.transition.is_none());
    }

    #[test]
    fn late_form_poll_keeps_the_authored_boundary() {
        let score = generate_suspense(&SuspenseInput {
            secret: "transport-regression".into(),
            seed: "scan-seam".into(),
            style: SuspenseStyle::Terminal,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.72,
            pulse: 0.55,
        })
        .unwrap();
        let mut transport = AdaptiveTransport::new(score, Some("verse")).unwrap();
        transport.advance(30719);
        assert!(transport.transition.is_none());
        transport.advance(30726);
        let plan = transport.transition.as_ref().unwrap();
        assert_eq!(plan.to, "pre-chorus");
        assert_eq!(plan.start_tick, 30720);
        assert_eq!(plan.end_tick, 38400);
        transport.advance(38406);
        assert_eq!(transport.section_entered_at, 38400);
    }

    fn trace_input() -> SuspenseInput {
        SuspenseInput {
            secret: "qa".into(),
            seed: "trace-rules".into(),
            style: SuspenseStyle::Terminal,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.72,
            pulse: 0.55,
        }
    }

    type TraceContractCase = (&'static str, f64, f64, f64, Option<(&'static str, bool)>);

    /// Hardcoded contract for `select_trace_section`: (phase, heat, focus,
    /// progress) -> expected (section, hold). The retired Extended preset used to
    /// add a `progress >= 0.8 -> outro` rule; the pool only serializes the base
    /// rules, so progress alone never cues outro.
    #[test]
    fn trace_selector_obeys_the_serialized_rule_contract() {
        use crate::suspense::select_trace_section;
        let cases: &[TraceContractCase] = &[
            // complete wins everywhere
            ("complete", 0.1, 0.1, 1.0, Some(("coda", true))),
            // extract routes to outro (hold)
            ("extract", 0.1, 0.1, 0.2, Some(("outro", true))),
            // alert routes to bridge (no hold)
            ("alert", 0.5, 0.2, 0.1, Some(("bridge", false))),
            // exploit with focus >= 0.7 routes to chorus; below the bound it does not
            ("exploit", 0.4, 0.71, 0.1, Some(("chorus", false))),
            ("exploit", 0.4, 0.69, 0.1, None),
            // progress alone has no rule: 0.8/0.94/0.79 all fall through
            ("scan", 0.5, 0.5, 0.8, None),
            ("scan", 0.5, 0.5, 0.94, None),
            ("scan", 0.5, 0.5, 0.79, None),
            // progress 0.95 reaches coda
            ("scan", 0.5, 0.5, 0.95, Some(("coda", true))),
            // heat >= 0.75 cues bridge; heat 0.74 does not (and has no outro net)
            ("scan", 0.75, 0.5, 0.8, Some(("bridge", false))),
            ("scan", 0.74, 0.5, 0.8, None),
        ];
        let score = suspense_pool("trace-rules");
        for &(phase, heat, focus, progress, expected) in cases {
            let state = TraceState {
                phase: phase.into(),
                heat,
                focus,
                progress,
            };
            let selected = select_trace_section(&score.rules, &state);
            assert_eq!(
                selected,
                expected.map(|(section, hold)| (section.to_string(), hold)),
                "{phase} heat={heat} focus={focus} progress={progress}"
            );
        }
    }

    #[test]
    fn trace_transport_progress_and_heat_obey_serialized_rules() {
        // A neutral phase plus progress 0.8/0.94 never cues outro: the retired
        // Extended rule is gone, so neither pool arrangement carries it.
        for arrangement in [
            crate::SuspenseArrangement::Seeded,
            crate::SuspenseArrangement::AllPhases,
        ] {
            for progress in [0.8, 0.94] {
                let progress_only = TraceState {
                    phase: "scan".into(),
                    heat: 0.5,
                    focus: 0.5,
                    progress,
                };
                let score =
                    crate::generate_suspense_arrangement(&trace_input(), arrangement).unwrap();
                let mut transport = AdaptiveTransport::new(score, None).unwrap();
                assert!(
                    transport.request_trace_state(&progress_only, 0).is_none(),
                    "{arrangement:?} has no outro progress rule, so progress {progress} must not cue outro"
                );
            }
        }

        // Heat >= 0.75 still outranks everything and cues bridge.
        let hot = TraceState {
            phase: "scan".into(),
            heat: 0.8,
            focus: 0.5,
            progress: 0.8,
        };
        for arrangement in [
            crate::SuspenseArrangement::Seeded,
            crate::SuspenseArrangement::AllPhases,
        ] {
            let score = crate::generate_suspense_arrangement(&trace_input(), arrangement).unwrap();
            let mut transport = AdaptiveTransport::new(score, None).unwrap();
            let plan = transport
                .request_trace_state(&hot, 0)
                .expect("heat rule must cue bridge");
            assert_eq!(plan.to, "bridge", "{arrangement:?} heat cues bridge");
        }
    }

    #[test]
    fn trace_transport_holds_and_rearms_across_arrangements() {
        for arrangement in [
            crate::SuspenseArrangement::Seeded,
            crate::SuspenseArrangement::AllPhases,
        ] {
            let score = crate::generate_suspense_arrangement(&trace_input(), arrangement).unwrap();

            let mut complete = AdaptiveTransport::new(score.clone(), None).unwrap();
            let plan = complete
                .request_trace_state(
                    &TraceState {
                        phase: "complete".into(),
                        heat: 0.1,
                        focus: 0.1,
                        progress: 1.0,
                    },
                    0,
                )
                .expect("complete cues coda");
            assert_eq!(plan.to, "coda", "{arrangement:?} complete");
            complete.advance(plan.end_tick);
            assert_eq!(
                complete.current_section(),
                "coda",
                "{arrangement:?} complete holds"
            );

            let mut extract = AdaptiveTransport::new(score.clone(), None).unwrap();
            let plan = extract
                .request_trace_state(
                    &TraceState {
                        phase: "extract".into(),
                        heat: 0.1,
                        focus: 0.1,
                        progress: 0.2,
                    },
                    0,
                )
                .expect("extract cues outro");
            assert_eq!(plan.to, "outro", "{arrangement:?} extract");

            let alert = TraceState {
                phase: "alert".into(),
                heat: 0.8,
                focus: 0.2,
                progress: 0.1,
            };
            let mut rearm = AdaptiveTransport::new(score, None).unwrap();
            let plan = rearm
                .request_trace_state(&alert, 0)
                .expect("alert cues bridge");
            assert_eq!(plan.to, "bridge", "{arrangement:?} alert");
            rearm.advance(plan.end_tick);
            assert_eq!(
                rearm.current_section(),
                "bridge",
                "{arrangement:?} alert lands"
            );
            assert!(
                rearm.request_trace_state(&alert, plan.end_tick).is_none(),
                "{arrangement:?} one-shot alert must not re-cue"
            );
        }
    }

    #[test]
    fn a_stale_alert_cue_does_not_swallow_later_alerts() {
        let score = crate::generate_suspense_arrangement(
            &SuspenseInput {
                secret: "qa".into(),
                seed: "alert-rearm".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            crate::SuspenseArrangement::AllPhases,
        )
        .unwrap();
        let bar = score.bar_ticks();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        let alert = TraceState {
            phase: "alert".into(),
            heat: 0.8,
            focus: 0.2,
            progress: 0.1,
        };
        let plan = transport
            .request_trace_state(&alert, 0)
            .expect("first alert cues bridge");
        assert_eq!(plan.to, "bridge");
        transport.advance(plan.end_tick);
        assert_eq!(transport.current_section(), "bridge");
        assert!(transport
            .request_trace_state(&alert, plan.end_tick)
            .is_none());
        let mut far = plan.end_tick + bar;
        for _ in 0..64 {
            transport.advance(far);
            if transport.current_section() == "solo"
                || transport.current_section() == "chorus-final"
            {
                break;
            }
            far += bar;
        }
        assert_ne!(transport.current_section(), "bridge");
        let rearmed = transport
            .request_trace_state(&alert, far)
            .expect("later alert must re-cue bridge");
        assert_eq!(rearmed.to, "bridge");
    }

    /// The Adventure autoplay form (camp → explore → … → victory) attached to a
    /// plain `generate_adventure` score, matching what the Godot player builds
    /// for `recipe = "adventure"`, `arrangement = "original"`, `autoplay = true`.
    fn adventure_autoplay_score() -> crate::score::PortableScore {
        apply_automatic_arrangement(
            generate_adventure(&AdventureInput {
                secret: "transport-regression".into(),
                seed: "adventure-tour".into(),
                style: AdventureStyle::Folk,
                wonder: 0.6,
                danger: 0.5,
                mystery: 0.6,
                motion: 0.58,
            })
            .expect("adventure fixture must validate"),
            ArrangementRecipe::Adventure,
            true,
        )
        .expect("adventure autoplay form must validate")
    }

    fn quest_complete_state() -> AdventureState {
        AdventureState {
            area_phase: "explore".into(),
            discovery: 0.4,
            threat: 0.1,
            quest_complete: true,
        }
    }

    #[test]
    fn quest_complete_supersedes_an_inflight_manual_form_step() {
        let score = adventure_autoplay_score();
        let bar = score.bar_ticks();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        assert_eq!(transport.current_section(), "camp");

        // A manual form step commits at the next bar boundary.
        let step = transport.advance_form(bar / 2).unwrap();
        assert_eq!(step.to, "explore");
        assert_eq!(step.start_tick, bar);
        // The form step is now in flight: past its start tick but not complete.
        transport.advance(step.start_tick);
        assert_eq!(transport.current_section(), "camp");

        // quest_complete arrives mid-transition and must supersede the stale
        // form step instead of queueing behind it.
        let victory = transport
            .request_adventure_state(&quest_complete_state(), step.start_tick)
            .expect("quest_complete must supersede the in-flight form step");
        assert_eq!(victory.to, "victory");
        // The superseded transition reaches victory within its authored span
        // (a structural simulation never reports the incoming, so it completes
        // at the plan end) rather than waiting out the old transition first.
        transport.advance(victory.end_tick);
        assert_eq!(transport.current_section(), "victory");
    }

    #[test]
    fn quest_complete_supersedes_an_inflight_automatic_form_transition() {
        let score = adventure_autoplay_score();
        let camp_len = score.section("camp").unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        assert_eq!(transport.current_section(), "camp");

        // Let the form auto-advance camp -> explore.
        transport.advance(camp_len);
        let automatic = transport
            .transition
            .as_ref()
            .expect("the form must auto-advance past camp")
            .clone();
        assert_eq!(automatic.to, "explore");
        assert!(
            !automatic.hold,
            "automatic form transitions crossfade on schedule"
        );

        // quest_complete mid-flight must supersede the automatic progression.
        let victory = transport
            .request_adventure_state(&quest_complete_state(), automatic.start_tick + 1)
            .expect("quest_complete supersedes the automatic transition");
        assert_eq!(victory.to, "victory");
        transport.advance(victory.end_tick);
        assert_eq!(transport.current_section(), "victory");
    }

    #[test]
    fn a_held_cue_still_defers_to_the_active_cue_after_the_supersede_change() {
        // An explicit cue (not a form step) in flight is never superseded by a
        // later game-state request: the hold guarantee stands.
        let score = adventure_autoplay_score();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        let cue = transport.request_section("explore", 0).unwrap();
        transport.advance(cue.start_tick);
        // quest_complete during an explicit held cue defers (queues) behind it.
        assert!(
            transport
                .request_adventure_state(&quest_complete_state(), cue.start_tick + 1)
                .is_none(),
            "a held cue is never dropped mid-hold, even for an authoritative request"
        );
        assert_eq!(transport.current_section(), "camp");
    }

    #[test]
    fn a_queued_cue_does_not_outlive_a_superseding_game_state() {
        let score = adventure_autoplay_score();
        let bar = score.bar_ticks();
        let mut transport = AdaptiveTransport::new(score, None).unwrap();
        assert_eq!(transport.current_section(), "camp");

        // A manual form step commits at the next bar boundary and is now in flight.
        let step = transport.advance_form(bar / 2).unwrap();
        transport.advance(step.start_tick);
        assert_eq!(transport.current_section(), "camp");

        // A direct cue during the in-flight form step defers (queues) behind it.
        assert!(
            transport.request_section("town", step.start_tick).is_none(),
            "a direct cue queues behind an in-flight form step"
        );

        // quest_complete supersedes the form step.
        let victory = transport
            .request_adventure_state(&quest_complete_state(), step.start_tick)
            .expect("quest_complete supersedes the in-flight form step");
        assert_eq!(victory.to, "victory");

        // The superseded transition lands on victory, and the stale queued cue
        // must not then drag the transport on to town.
        transport.advance(victory.end_tick);
        assert_eq!(transport.current_section(), "victory");
        assert!(
            transport.transition.is_none(),
            "the superseding game state drops the stale queued cue"
        );
    }
}
