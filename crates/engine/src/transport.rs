use crate::score::{GameState, PortableScore, TraceState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionPlan {
    pub from: String,
    pub to: String,
    pub start_tick: u32,
    pub end_tick: u32,
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
        let target = trace_target(state);
        match target {
            Some(section) if !holds_trace_target(section) => {
                if self.cue_target.as_deref() == Some(section) {
                    return None;
                }
                self.cue_target = Some(section.to_string());
                self.sync_form_to(section);
                self.request_section(section, at_tick)
            }
            Some(section) => {
                self.cue_target = None;
                self.sync_form_to(section);
                self.request_section(section, at_tick)
            }
            None => {
                self.cue_target = None;
                None
            }
        }
    }

    pub fn request_section(&mut self, target: &str, at_tick: u32) -> Option<TransitionPlan> {
        self.advance(at_tick);
        if target == self.current_section
            && self.transition.is_none()
            && self.pending_section.is_none()
        {
            return None;
        }
        self.score.section(target)?;
        if let Some(plan) = &self.transition {
            if plan.to == target {
                self.pending_section = None;
                return None;
            }
            if at_tick < plan.start_tick {
                if target == self.current_section {
                    self.transition = None;
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
        self.sync_form_to(target);
        Some(plan)
    }

    pub fn advance(&mut self, at_tick: u32) {
        if let Some(plan) = &self.transition {
            if at_tick >= plan.end_tick {
                self.current_section = plan.to.clone();
                self.section_entered_at = at_tick;
                self.sync_form_to(&self.current_section.clone());
                self.transition = None;
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
                let progress = (at_tick - plan.start_tick) as f32
                    / (plan.end_tick - plan.start_tick).max(1) as f32;
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

    fn create_plan(&self, from: &str, to: &str, at_tick: u32) -> TransitionPlan {
        let start = at_tick.div_ceil(self.bar_ticks) * self.bar_ticks;
        let start = start.max(at_tick);
        let length = (self.score.crossfade_bars * f64::from(self.bar_ticks)).round() as u32;
        TransitionPlan {
            from: from.to_string(),
            to: to.to_string(),
            start_tick: start,
            end_tick: start + length.max(self.bar_ticks),
        }
    }

    fn sync_form_to(&mut self, section: &str) {
        if let Some(index) = form_index_for(&self.score, section) {
            self.form_step_index = index;
        }
    }

    fn maybe_advance_form(&mut self, at_tick: u32) {
        if self.transition.is_some() || self.pending_section.is_some() {
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
        let duration = section.length_ticks.saturating_mul(repeats);
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
            self.section_entered_at = at_tick;
            return;
        }
        let plan = self.create_plan(&self.current_section, &next_section, at_tick);
        self.transition = Some(plan);
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

fn trace_target(state: &TraceState) -> Option<&'static str> {
    if state.phase == "complete" || state.progress >= 0.95 {
        return Some("coda");
    }
    if state.phase == "extract" || state.progress >= 0.8 {
        return Some("outro");
    }
    if state.phase == "alert" || state.heat >= 0.75 {
        return Some("bridge");
    }
    if state.phase == "exploit" && state.focus >= 0.7 {
        return Some("chorus");
    }
    None
}

fn holds_trace_target(section: &str) -> bool {
    matches!(section, "coda" | "outro")
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
    use crate::pocket_circuit::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};
    use crate::score::GameState;

    fn score() -> crate::score::PortableScore {
        generate_pocket_circuit(&GenerateInput {
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
}
