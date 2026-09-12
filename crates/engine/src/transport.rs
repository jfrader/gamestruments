use crate::score::{AdventureState, FormOrigin, GameState, PortableScore, TraceState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionPlan {
    pub from: String,
    pub to: String,
    pub start_tick: u32,
    pub end_tick: u32,
}

pub struct SectionPlayback<'a> {
    pub section: &'a str,
    pub origin: u32,
    pub gain: f32,
    pub percussion: bool,
}

pub struct AdaptiveTransport {
    score: PortableScore,
    bar_ticks: u32,
    current_section: String,
    pending_section: Option<String>,
    transition: Option<TransitionPlan>,
    form_step_index: usize,
    section_entered_at: u32,
    cue_target: Option<String>,
    form_held: bool,
    form_not_before: u32,
    automatic_transition: bool,
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
            cue_target: None,
            form_held: false,
            form_not_before: 0,
            automatic_transition: false,
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
        self.request_section(&target, at_tick)
    }

    pub fn request_trace_state(
        &mut self,
        state: &TraceState,
        at_tick: u32,
    ) -> Option<TransitionPlan> {
        self.advance(at_tick);
        let Some((section, hold)) = crate::suspense::select_trace_section(state) else {
            self.cue_target = None;
            return None;
        };
        if hold {
            self.cue_target = None;
            return self.request_section(section, at_tick);
        }
        let already_cued = self.cue_target.as_deref() == Some(section)
            && self.current_section == section
            && self.transition.is_none();
        if already_cued {
            return None;
        }
        self.cue_target = Some(section.to_string());
        self.request_section(section, at_tick)
    }

    pub fn request_adventure_state(
        &mut self,
        state: &AdventureState,
        at_tick: u32,
    ) -> Option<TransitionPlan> {
        let target = crate::adventure::select_adventure_section(state);
        self.request_section(target, at_tick)
    }

    pub fn request_section(&mut self, target: &str, at_tick: u32) -> Option<TransitionPlan> {
        self.score.section(target)?;
        self.advance(at_tick);
        self.automatic_transition = false;
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
            if at_tick < plan.start_tick {
                if target == self.current_section {
                    self.transition = None;
                    self.pending_section = None;
                    self.sync_form_to(&self.current_section.clone());
                    return None;
                }
                let next = self.create_plan(&self.current_section, target, at_tick);
                self.transition = Some(next.clone());
                return Some(next);
            }
            self.pending_section = Some(target.to_string());
            return None;
        }
        let plan = self.create_plan(&self.current_section, target, at_tick);
        self.transition = Some(plan.clone());
        Some(plan)
    }

    pub fn advance(&mut self, at_tick: u32) {
        if let Some(plan) = &self.transition {
            if at_tick >= plan.end_tick {
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
                self.sync_form_to(&self.current_section.clone());
                self.transition = None;
                self.automatic_transition = false;
                self.form_not_before = 0;
                if let Some(pending) = self.pending_section.take() {
                    if pending != self.current_section {
                        let next = self.create_plan(&self.current_section, &pending, at_tick);
                        self.transition = Some(next);
                    }
                }
            }
        }
        self.maybe_advance_form(at_tick);
    }

    pub fn section_gain(&self, section_id: &str, at_tick: u32) -> f32 {
        if let Some(plan) = &self.transition {
            if at_tick >= plan.start_tick && at_tick < plan.end_tick {
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
            && self.automatic_transition
            && self
                .transition
                .as_ref()
                .is_some_and(|plan| at_tick < plan.start_tick)
        {
            let plan = self.transition.take();
            self.automatic_transition = false;
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
        let index = form_index_for(&self.score, current)?;
        let next = if index + 1 < form.steps.len() {
            index + 1
        } else {
            form.loop_from? as usize
        };
        let target = &form.steps.get(next)?.section;
        (target != current).then(|| target.clone())
    }

    pub fn advance_form(&mut self, at_tick: u32) -> Option<TransitionPlan> {
        let target = self.next_form_section(at_tick)?;
        self.request_section(&target, at_tick)
    }

    pub fn playback_at(&self, at_tick: u32) -> [Option<SectionPlayback<'_>>; 2] {
        if let Some(plan) = &self.transition {
            if at_tick >= plan.end_tick {
                return [
                    Some(SectionPlayback {
                        section: &plan.to,
                        origin: plan.start_tick,
                        gain: 1.0,
                        percussion: true,
                    }),
                    None,
                ];
            }
            if at_tick >= plan.start_tick && at_tick < plan.end_tick {
                let progress = self.fade_progress(plan, at_tick);
                return [
                    Some(SectionPlayback {
                        section: &plan.from,
                        origin: self.section_entered_at,
                        gain: 1.0 - progress,
                        percussion: false,
                    }),
                    Some(SectionPlayback {
                        section: &plan.to,
                        origin: plan.start_tick,
                        gain: progress,
                        percussion: true,
                    }),
                ];
            }
        }
        [
            Some(SectionPlayback {
                section: &self.current_section,
                origin: self.section_entered_at,
                gain: 1.0,
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
        }
    }

    fn transition_length(&self) -> u32 {
        let length = (self.score.crossfade_bars * f64::from(self.bar_ticks)).round() as u32;
        length.max(self.bar_ticks)
    }

    fn fade_progress(&self, plan: &TransitionPlan, at_tick: u32) -> f32 {
        (at_tick - plan.start_tick) as f32 / (plan.end_tick - plan.start_tick).max(1) as f32
    }

    fn sync_form_to(&mut self, section: &str) {
        if let Some(index) = form_index_for(&self.score, section) {
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
        self.form_step_index = next_index;
        if next_section == self.current_section {
            self.section_entered_at = boundary_tick;
            return;
        }
        let length = self.transition_length();
        let plan = TransitionPlan {
            from: self.current_section.clone(),
            to: next_section,
            start_tick: boundary_tick,
            end_tick: boundary_tick.saturating_add(length),
        };
        self.transition = Some(plan);
        self.automatic_transition = true;
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

pub fn select_section(score: &PortableScore, state: &GameState) -> String {
    if state.finish_result == "win" && state.race_phase == "finish" {
        return "victory".into();
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
        "finish" => "victory".into(),
        _ => score.default_section.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::AdaptiveTransport;
    use crate::racing::{generate_racing, GenerateInput, InstrumentPalette, Style};
    use crate::score::{GameState, TraceState};
    use crate::suspense::{generate_suspense, SuspenseInput, SuspenseStyle};

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
    fn game_hold_advances_to_the_variation_and_resumes_on_a_future_loop_boundary() {
        let score = crate::generate_suspense_arrangement(
            &SuspenseInput {
                secret: "qa".into(),
                seed: "game-controls".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            crate::SuspenseArrangement::Extended,
        )
        .unwrap();
        let bar = score.bar_ticks();
        let length = score.section("verse").unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, Some("verse")).unwrap();
        transport.set_form_held(true, 0);
        transport.advance(5 * length);
        assert_eq!(transport.current_section(), "verse");
        assert!(transport.transition.is_none());
        let plan = transport.advance_form(5 * length + 100).unwrap();
        assert_eq!(plan.to, "scan-ii");
        assert_eq!(plan.start_tick % bar, 0);
        transport.advance(plan.end_tick);
        assert!(transport.is_form_held());
        assert_eq!(transport.current_section(), "scan-ii");
        let now = 20 * length + 100;
        transport.advance(now);
        transport.set_form_held(false, now);
        transport.advance(now);
        assert!(transport.transition.is_none());
        let boundary = transport.section_entered_at
            + (now - transport.section_entered_at).div_ceil(length) * length;
        transport.advance(boundary - 1);
        assert!(transport.transition.is_none());
        transport.advance(boundary);
        assert_eq!(transport.transition.as_ref().unwrap().start_tick, boundary);
        assert_eq!(transport.transition.as_ref().unwrap().to, "pre-chorus");
    }

    #[test]
    fn holding_during_an_automatic_blend_keeps_the_incoming_section() {
        let score = crate::generate_suspense_arrangement(
            &SuspenseInput {
                secret: "qa".into(),
                seed: "hold-blend".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            crate::SuspenseArrangement::Extended,
        )
        .unwrap();
        let length = score.section("verse").unwrap().length_ticks;
        let mut transport = AdaptiveTransport::new(score, Some("verse")).unwrap();
        transport.advance(length);
        let end = transport.transition.as_ref().unwrap().end_tick;
        assert!(transport.set_form_held(true, length + 1).is_none());
        transport.advance(end);
        transport.advance(10 * length);
        assert_eq!(transport.current_section(), "scan-ii");
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

    #[test]
    fn trace_selector_and_serialized_rules_agree_across_states() {
        use crate::suspense::{extended_trace_rules, select_trace_section};
        let phases = ["boot", "scan", "exploit", "alert", "extract", "complete"];
        for phase in phases {
            for heat in [0.0, 0.5, 0.76, 1.0] {
                for focus in [0.0, 0.5, 0.71, 1.0] {
                    for progress in [0.0, 0.5, 0.81, 0.96, 1.0] {
                        let state = TraceState {
                            phase: phase.into(),
                            heat,
                            focus,
                            progress,
                        };
                        let selected: Option<(String, bool)> = select_trace_section(&state)
                            .map(|(section, hold)| (section.to_string(), hold));
                        let ruled: Option<(String, bool)> = extended_trace_rules()
                            .into_iter()
                            .find(|rule| {
                                let phase_ok = rule
                                    .when
                                    .categorical
                                    .get("tracePhase")
                                    .is_none_or(|expected| expected == &state.phase);
                                let numeric = rule.when.numeric.as_object().unwrap();
                                let numeric_ok = [
                                    ("heat", state.heat),
                                    ("focus", state.focus),
                                    ("progress", state.progress),
                                ]
                                .into_iter()
                                .all(|(key, value)| {
                                    numeric
                                        .get(key)
                                        .is_none_or(|spec| value >= spec["min"].as_f64().unwrap())
                                });
                                phase_ok && numeric_ok
                            })
                            .map(|rule| (rule.target.clone(), rule.hold.unwrap_or(true)));
                        assert_eq!(
                            selected, ruled,
                            "{phase} heat={heat} focus={focus} progress={progress}"
                        );
                    }
                }
            }
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
            crate::SuspenseArrangement::Extended,
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
}
