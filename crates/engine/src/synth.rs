use crate::score::{MusicEvent, PortableScore};

pub struct VoiceVoice {
    pub frequency: f32,
    pub remaining: f32,
    pub velocity: f32,
    pub kind: VoiceKind,
}

#[derive(Clone, Copy)]
pub enum VoiceKind {
    Lead,
    Pad,
    Bass,
    Kick,
    Snare,
    Hat,
}

pub struct Synth {
    sample_rate: f32,
    phase: f32,
    voices: Vec<VoiceVoice>,
    noise_state: u32,
}

impl Synth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            voices: Vec::new(),
            noise_state: 0x1234_5678,
        }
    }

    pub fn trigger(&mut self, event: &MusicEvent, ticks_per_second: f64) {
        let duration = event.duration_ticks() as f64 / ticks_per_second;
        let velocity = event.velocity() as f32;
        if let Some(pitch) = event.pitch() {
            let frequency = 440.0 * 2_f32.powf((f32::from(pitch) - 69.0) / 12.0);
            let kind = if event.voice() == "bass" {
                VoiceKind::Bass
            } else if event.is_melody() {
                VoiceKind::Lead
            } else {
                VoiceKind::Pad
            };
            self.voices.push(VoiceVoice {
                frequency,
                remaining: duration as f32 + 0.06,
                velocity,
                kind,
            });
            return;
        }
        let kind = match event.voice() {
            "kick" => VoiceKind::Kick,
            "snare" => VoiceKind::Snare,
            "hat" => VoiceKind::Hat,
            _ => VoiceKind::Kick,
        };
        self.voices.push(VoiceVoice {
            frequency: 80.0,
            remaining: 0.12,
            velocity,
            kind,
        });
    }

    pub fn fill(&mut self, buffer: &mut [f32]) {
        let step = 1.0 / self.sample_rate;
        for sample in buffer.iter_mut() {
            let mut mix = 0.0;
            for voice in &mut self.voices {
                if voice.remaining <= 0.0 {
                    continue;
                }
                let envelope = (voice.remaining * 8.0).clamp(0.0, 1.0);
                mix += match voice.kind {
                    VoiceKind::Lead => {
                        sine(voice.frequency, self.phase) * 0.07 * voice.velocity * envelope
                    }
                    VoiceKind::Pad => {
                        sine(voice.frequency, self.phase) * 0.03 * voice.velocity * envelope
                    }
                    VoiceKind::Bass => {
                        (sine(voice.frequency, self.phase)
                            + 0.25 * sine(voice.frequency * 2.0, self.phase))
                            * 0.1
                            * voice.velocity
                            * envelope
                    }
                    VoiceKind::Kick => {
                        sine(lerpf(150.0, 50.0, 1.0 - envelope), self.phase)
                            * 0.2
                            * voice.velocity
                            * envelope
                    }
                    VoiceKind::Snare => {
                        noise(&mut self.noise_state) * 0.12 * voice.velocity * envelope
                    }
                    VoiceKind::Hat => {
                        noise(&mut self.noise_state) * 0.04 * voice.velocity * envelope
                    }
                };
                voice.remaining -= step;
            }
            self.voices.retain(|voice| voice.remaining > 0.0);
            *sample = mix.clamp(-0.95, 0.95);
            self.phase += step;
        }
    }
}

pub fn events_starting_at(
    score: &PortableScore,
    section_id: &str,
    local_tick: u32,
    window: u32,
) -> Vec<MusicEvent> {
    score
        .section(section_id)
        .map(|section| {
            section
                .events
                .iter()
                .filter(|event| {
                    let start = event.start_tick();
                    start >= local_tick && start < local_tick + window
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn sine(frequency: f32, time: f32) -> f32 {
    (std::f32::consts::TAU * frequency * time).sin()
}

fn lerpf(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t.clamp(0.0, 1.0)
}

fn noise(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::Synth;
    use crate::pocket_circuit::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};

    #[test]
    fn cruise_events_render_audible_samples() {
        let score = generate_pocket_circuit(&GenerateInput {
            secret: "qa-secret".into(),
            seed: "qa-race".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.7,
            complexity: 0.6,
            brightness: 0.5,
            syncopation: 0.7,
        });
        let ticks_per_second = score.ticks_per_second();
        let mut synth = Synth::new(22050.0);
        for event in &score.section("cruise").unwrap().events {
            if event.start_tick() < 960 {
                synth.trigger(event, ticks_per_second);
            }
        }
        let mut buffer = vec![0.0_f32; 2205];
        synth.fill(&mut buffer);
        let energy: f32 = buffer.iter().map(|sample| sample.abs()).sum();
        assert!(energy > 1.0, "expected audible energy, got {energy}");
    }
}
