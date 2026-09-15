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
            SuspenseArrangement::Extended,
        )
        .unwrap();
        let mut transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut previous = "intro".to_string();
        let mut entered = 0;
        let mut scan_entries = 0;
        for tick in (0..1000 * score.bar_ticks()).step_by(480) {
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
                assert_eq!(
                    tick - entered,
                    score.section(&previous).unwrap().length_ticks
                );
                entered = tick;
                previous = owner.section.to_string();
                if owner.section == "verse" {
                    scan_entries += 1;
                }
                if scan_entries == 3 {
                    break;
                }
            }
            let section = score.section(owner.section).unwrap();
            let local = tick.saturating_sub(owner.origin) % section.length_ticks;
            if section.id == "intro" {
                continue;
            }
            let kick = section
                .events
                .iter()
                .filter(|event| event.voice() == "kick" && event.start_tick() == local)
                .count();
            let hat = section
                .events
                .iter()
                .filter(|event| event.voice() == "hat" && event.start_tick() == local)
                .count();
            if section.id == "break" {
                assert_eq!((kick, hat), (0, 0));
            } else {
                assert_eq!(kick, usize::from(local.is_multiple_of(1920)));
                assert_eq!(hat, usize::from(local % 960 == 480));
            }
        }
        assert_eq!(scan_entries, 3);
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
            SuspenseArrangement::Extended,
        )
        .unwrap();
        let frames =
            (score.bar_ticks() as f64 * 11.0 / score.ticks_per_second() * 8000.0).ceil() as usize;
        let mut one = FormAudio::new(&score, 8000.0);
        let mut chunked = FormAudio::new(&score, 8000.0);
        let mut first_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut second_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut expected = vec![0.0; frames];
        let mut actual = vec![0.0; frames];
        one.fill(&score, &mut first_transport, &mut expected);
        for chunk in actual.chunks_mut(317) {
            chunked.fill(&score, &mut second_transport, chunk);
        }
        assert_eq!(actual, expected);
        assert!(actual.iter().any(|value| value.abs() > 0.02));
        assert_eq!(first_transport.current_section(), "verse");
        assert_eq!(
            chunked.tick(score.ticks_per_second()),
            one.tick(score.ticks_per_second())
        );
    }
}
