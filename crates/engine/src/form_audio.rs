use crate::score::{MusicEvent, PortableScore};
use crate::synth::Synth;
use crate::transport::AdaptiveTransport;

pub struct FormAudio {
    sample_rate: f32,
    frames: u64,
    last_tick: Option<u32>,
    tonal: Vec<Synth>,
    origins: Vec<Option<u32>>,
    active: [Option<(usize, f32)>; 2],
    drums: Synth,
}

impl FormAudio {
    pub fn new(score: &PortableScore, sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frames: 0,
            last_tick: None,
            tonal: score
                .sections
                .iter()
                .map(|_| Synth::new(sample_rate))
                .collect(),
            origins: vec![None; score.sections.len()],
            active: [None, None],
            drums: Synth::new(sample_rate),
        }
    }

    pub fn tick(&self, ticks_per_second: f64) -> u32 {
        (self.frames as f64 * ticks_per_second / f64::from(self.sample_rate)).floor() as u32
    }

    pub fn fill(
        &mut self,
        score: &PortableScore,
        transport: &mut AdaptiveTransport,
        buffer: &mut [f32],
    ) {
        let ticks_per_second = score.ticks_per_second();
        for sample in buffer {
            let tick = self.tick(ticks_per_second);
            if self.last_tick != Some(tick) {
                transport.advance(tick);
                self.active = [None, None];
                for (slot, playback) in transport.playback_at(tick).into_iter().enumerate() {
                    let Some(playback) = playback else {
                        continue;
                    };
                    let Some(index) = score
                        .sections
                        .iter()
                        .position(|section| section.id == playback.section)
                    else {
                        continue;
                    };
                    let section = &score.sections[index];
                    if self.origins[index] != Some(playback.origin) {
                        self.tonal[index] = Synth::new(self.sample_rate);
                        self.origins[index] = Some(playback.origin);
                    }
                    let local = tick.saturating_sub(playback.origin) % section.length_ticks;
                    for event in section
                        .events
                        .iter()
                        .filter(|event| event.start_tick() == local)
                    {
                        match event {
                            MusicEvent::Note { .. } => {
                                self.tonal[index].trigger(event, ticks_per_second)
                            }
                            MusicEvent::Percussion { .. } if playback.percussion => {
                                self.drums.trigger(event, ticks_per_second)
                            }
                            _ => (),
                        }
                    }
                    self.active[slot] = Some((index, playback.gain));
                }
                self.last_tick = Some(tick);
            }
            let mut rhythm = [0.0];
            self.drums.fill(&mut rhythm);
            let mut mix = rhythm[0];
            for (index, gain) in self.active.into_iter().flatten() {
                let mut tonal = [0.0];
                self.tonal[index].fill(&mut tonal);
                mix += tonal[0] * gain;
            }
            *sample = mix;
            self.frames += 1;
        }
    }

    /// Stereo fill: uses the engine stereo path (with pans, split twins, opposite echoes).
    /// All game elements currently summed to centre in this helper (L=R) to preserve
    /// existing game balance while allowing the Godot player to consume two channels.
    /// (Full per-voice panning in game context can be enabled by routing the L/R here.)
    pub fn fill_stereo(
        &mut self,
        score: &PortableScore,
        transport: &mut AdaptiveTransport,
        left: &mut [f32],
        right: &mut [f32],
    ) {
        let len = left.len();
        assert_eq!(len, right.len());
        let ticks_per_second = score.ticks_per_second();
        for i in 0..len {
            let tick = self.tick(ticks_per_second);
            if self.last_tick != Some(tick) {
                transport.advance(tick);
                self.active = [None, None];
                for (slot, playback) in transport.playback_at(tick).into_iter().enumerate() {
                    let Some(playback) = playback else {
                        continue;
                    };
                    let Some(index) = score
                        .sections
                        .iter()
                        .position(|section| section.id == playback.section)
                    else {
                        continue;
                    };
                    let section = &score.sections[index];
                    if self.origins[index] != Some(playback.origin) {
                        self.tonal[index] = Synth::new(self.sample_rate);
                        self.origins[index] = Some(playback.origin);
                    }
                    let local = tick.saturating_sub(playback.origin) % section.length_ticks;
                    for event in section
                        .events
                        .iter()
                        .filter(|event| event.start_tick() == local)
                    {
                        match event {
                            MusicEvent::Note { .. } => {
                                self.tonal[index].trigger(event, ticks_per_second)
                            }
                            MusicEvent::Percussion { .. } if playback.percussion => {
                                self.drums.trigger(event, ticks_per_second)
                            }
                            _ => (),
                        }
                    }
                    self.active[slot] = Some((index, playback.gain));
                }
                self.last_tick = Some(tick);
            }
            let mut rl = [0.0f32];
            let mut rr = [0.0f32];
            self.drums.fill_stereo(&mut rl, &mut rr);
            let mut mix_l = rl[0];
            let mut mix_r = rr[0];
            for (index, gain) in self.active.into_iter().flatten() {
                let mut tl = [0.0f32];
                let mut tr = [0.0f32];
                self.tonal[index].fill_stereo(&mut tl, &mut tr);
                mix_l += tl[0] * gain;
                mix_r += tr[0] * gain;
            }
            left[i] = mix_l;
            right[i] = mix_r;
            self.frames += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_suspense_arrangement, SuspenseArrangement, SuspenseInput, SuspenseStyle};

    #[test]
    fn native_full_form_has_one_rhythm_owner_and_no_extra_opening_bars() {
        let score = generate_suspense_arrangement(
            &SuspenseInput {
                secret: "test".into(),
                seed: "flow-cycle".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            SuspenseArrangement::AllPhases,
        )
        .unwrap();
        let bar = score.bar_ticks();
        let form = score.form.as_ref().unwrap();
        let form_ticks: u32 = form
            .steps
            .iter()
            .map(|step| score.section(&step.section).unwrap().length_ticks * step.repeats)
            .sum();
        let mut transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut previous = "intro".to_string();
        let mut entered = 0;
        let mut first = true;
        let mut verse_entries = 0;
        // The pool's forms leave `origin` unset, so `advance` marks a section
        // entered at the end of its crossfade while the playback owner switches
        // at the crossfade start. Once the first (transition-less) opening is
        // past, each owner window is the authored length plus that transition —
        // never anything more.
        let transition = ((score.crossfade_bars * f64::from(bar)).round() as u32).max(bar);
        for tick in (0..3 * form_ticks + bar).step_by(480) {
            transport.advance(tick);
            let playback = transport.playback_at(tick);
            let owners: Vec<_> = playback
                .iter()
                .flatten()
                .filter(|part| part.percussion)
                .collect();
            assert_eq!(owners.len(), 1);
            let owner = owners[0];
            if owner.section != previous {
                let len = score.section(&previous).unwrap().length_ticks;
                let expected = if first { len } else { len + transition };
                assert_eq!(
                    tick - entered,
                    expected,
                    "boundary after {previous} took {} ticks",
                    tick - entered
                );
                first = false;
                entered = tick;
                previous = owner.section.to_string();
                if owner.section == "verse" {
                    verse_entries += 1;
                }
                if verse_entries == 3 {
                    break;
                }
            }
        }
        assert_eq!(verse_entries, 3);
    }

    #[test]
    fn form_audio_clock_and_samples_do_not_depend_on_buffer_size() {
        let score = generate_suspense_arrangement(
            &SuspenseInput {
                secret: "test".into(),
                seed: "flow".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            SuspenseArrangement::Seeded,
        )
        .unwrap();
        let frames =
            (score.bar_ticks() as f64 * 40.0 / score.ticks_per_second() * 8000.0).ceil() as usize;
        let mut one = FormAudio::new(&score, 8000.0);
        let mut chunked = FormAudio::new(&score, 8000.0);
        let mut first_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut second_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let opening = first_transport.current_section().to_string();
        let mut expected = vec![0.0; frames];
        let mut actual = vec![0.0; frames];
        one.fill(&score, &mut first_transport, &mut expected);
        for chunk in actual.chunks_mut(317) {
            chunked.fill(&score, &mut second_transport, chunk);
        }
        assert_eq!(actual, expected);
        assert!(actual.iter().any(|value| value.abs() > 0.02));
        assert_ne!(
            first_transport.current_section(),
            opening,
            "the form must advance past its opening section"
        );
        assert_eq!(
            chunked.tick(score.ticks_per_second()),
            one.tick(score.ticks_per_second())
        );
    }
}
