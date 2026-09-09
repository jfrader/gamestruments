use crate::score::PortableScore;
use crate::synth::{events_starting_at, Synth};

/// Render `phrases` repetitions of `section_id` from `score` as 16-bit mono PCM WAV.
/// Applies a short outer-boundary seam fade using smoothstep over 0.01s (matching
/// the lab reference implementation) to avoid clicks at the loop point of the result.
pub fn render_wav(
    score: &PortableScore,
    section_id: &str,
    phrases: usize,
    sample_rate: u32,
) -> Vec<u8> {
    let section = score
        .section(section_id)
        .expect("section_id must exist in score");
    let length_ticks = section.length_ticks;
    let tps = score.ticks_per_second();
    let phrase_sec = length_ticks as f64 / tps;
    let total_sec = phrase_sec * phrases as f64;
    let total_samples = (total_sec * sample_rate as f64).round() as usize;

    let mut synth = Synth::new(sample_rate as f32);
    let mut samples = Vec::with_capacity(total_samples);

    let mut pos = 0usize;
    let mut tick: u32 = 0;
    const CHUNK: usize = 256;

    while pos < total_samples {
        let chunk = (total_samples - pos).min(CHUNK);
        let window_ticks = (((chunk as f64 / sample_rate as f64) * tps).ceil() as u32).max(1) + 1;
        let local = tick % length_ticks.max(1);

        for ev in events_starting_at(score, section_id, local, window_ticks) {
            synth.trigger(&ev, tps);
        }

        let mut buf = vec![0.0f32; chunk];
        synth.fill(&mut buf);
        samples.extend_from_slice(&buf);

        pos += chunk;
        tick = tick.wrapping_add(window_ticks);
    }

    apply_outer_seam_fade(&mut samples, sample_rate, total_sec);
    encode_16bit_mono_wav(&samples, sample_rate)
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn apply_outer_seam_fade(samples: &mut [f32], sr: u32, total_dur: f64) {
    for (i, s) in samples.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        let fi = smoothstep(0.0, 0.01, t);
        let fo = smoothstep(0.0, 0.01, total_dur - t);
        *s *= (fi.min(fo)) as f32;
    }
}

fn encode_16bit_mono_wav(samples: &[f32], sr: u32) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_size as usize);

    wav.extend_from_slice(b"RIFF");
    let file_size = 36u32 + data_size;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sr.to_le_bytes());
    let byte_rate = sr * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // 16-bit
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        wav.extend_from_slice(&v.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocket_circuit::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};

    fn parse_wav_header(wav: &[u8]) -> (u32, u16, usize) {
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        let sr = u32::from_le_bytes(wav[24..28].try_into().unwrap());
        let channels = u16::from_le_bytes(wav[22..24].try_into().unwrap());
        let bits = u16::from_le_bytes(wav[34..36].try_into().unwrap());
        let data_size = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
        assert_eq!(channels, 1);
        assert_eq!(bits, 16);
        (sr, bits, data_size / 2)
    }

    fn decode_wav_samples(wav: &[u8]) -> Vec<f32> {
        let data_start = 44;
        let data_size = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
        let mut out = Vec::with_capacity(data_size / 2);
        for i in (0..data_size).step_by(2) {
            let s = i16::from_le_bytes(wav[data_start + i..data_start + i + 2].try_into().unwrap());
            out.push(s as f32 / 32768.0);
        }
        out
    }

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn rms(samples: &[f32]) -> f32 {
        let sq_sum: f32 = samples.iter().map(|s| s * s).sum();
        (sq_sum / samples.len() as f32).sqrt()
    }

    fn seam_delta(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            0.0
        } else {
            (samples[0] - samples[samples.len() - 1]).abs()
        }
    }

    fn envelope_variation(samples: &[f32], sample_rate: u32, lag_seconds: f64) -> f64 {
        let window_size = 1usize.max((sample_rate as f64 * 0.02).round() as usize);
        let wcount = samples.len() / window_size;
        let mut env = vec![0.0f64; wcount];
        for (wi, e) in env.iter_mut().enumerate().take(wcount) {
            let start = wi * window_size;
            let sum: f64 = samples[start..start + window_size]
                .iter()
                .map(|&s| s.abs() as f64)
                .sum();
            *e = sum / window_size as f64;
        }
        let lag_w =
            1usize.max((lag_seconds * sample_rate as f64 / window_size as f64).round() as usize);
        let mut diff = 0.0;
        let mut level = 0.0;
        for (wi, &cur) in env.iter().enumerate().take(wcount.saturating_sub(lag_w)) {
            let del = env[wi + lag_w];
            diff += (cur - del).abs();
            level += cur.max(del);
        }
        diff / level.max(0.000001)
    }

    #[test]
    fn render_header_sample_count_peak_seam_rms() {
        let score = generate_pocket_circuit(&GenerateInput {
            secret: "qa-secret".into(),
            seed: "qa-race".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.7,
            complexity: 0.6,
            brightness: 0.5,
            syncopation: 0.7,
        })
        .expect("render test score must validate");
        let wav = render_wav(&score, "cruise", 2, 22050);
        let (sr, _bits, n_samples) = parse_wav_header(&wav);
        assert_eq!(sr, 22050);

        let expected = ((score.section("cruise").unwrap().length_ticks as f64
            / score.ticks_per_second())
            * 2.0
            * 22050.0)
            .round() as usize;
        assert_eq!(n_samples, expected, "sample count must match duration * sr");

        let samples = decode_wav_samples(&wav);
        let p = peak(&samples);
        assert!(
            p > 0.05 && p < 0.9,
            "peak must be in (0.05, 0.9), got {}",
            p
        );

        let seam = seam_delta(&samples);
        assert!(seam < 0.02, "seam delta < 0.02, got {}", seam);

        let r = rms(&samples);
        assert!(r > 0.001 && r < 0.5, "RMS in sane range, got {}", r);
    }

    #[test]
    fn golden_catalog_grid_3phrases_22050() {
        let json = include_str!("../../../catalog/pocket-circuit/tiny-torque-level-004/score.json");
        let score: PortableScore =
            serde_json::from_str(json).expect("catalog JSON must parse with existing derives");
        let wav = render_wav(&score, "grid", 3, 22050);
        let samples = decode_wav_samples(&wav);
        let sr = 22050u32;

        let dur = samples.len() as f64 / sr as f64;
        assert!(
            (dur - 21.333).abs() < 0.05,
            "duration ≈ 21.333s ±0.05, got {}",
            dur
        );

        let p = peak(&samples);
        assert!(p < 0.9, "peak < 0.9, got {}", p);

        let var = envelope_variation(&samples, sr, 0.5);
        assert!(
            var > 0.08,
            "half-second envelope variation > 0.08, got {}",
            var
        );
    }
}
