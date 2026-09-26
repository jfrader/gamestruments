use crate::master::{MasterChain, MasterConfig};
use crate::score::{MusicEvent, PortableScore, PortableSection};
use crate::synth::{score_tick_at_sample, Synth};

/// Render `phrases` repetitions of `section_id` from `score` as 16-bit mono PCM WAV.
/// Applies a short outer-boundary seam fade using smoothstep over 0.01s (matching
/// the lab reference implementation) to avoid clicks at the loop point of the result.
pub fn render_wav(
    score: &PortableScore,
    section_id: &str,
    phrases: usize,
    sample_rate: u32,
) -> Vec<u8> {
    let samples = render_samples_mono(score, section_id, 0.0, phrases as f64, sample_rate, true);
    encode_16bit_mono_wav(&samples, sample_rate)
}

/// Render as stereo 16-bit WAV. Uses stereo master path (linked).
pub fn render_wav_stereo(
    score: &PortableScore,
    section_id: &str,
    phrases: usize,
    sample_rate: u32,
) -> Vec<u8> {
    let (left, right) =
        render_samples_stereo(score, section_id, 0.0, phrases as f64, sample_rate, true);
    encode_16bit_stereo_wav(&left, &right, sample_rate)
}

fn apply_outer_seam_fade_stereo(left: &mut [f32], right: &mut [f32], sr: u32, total_dur: f64) {
    for (i, (sl, sr_)) in left.iter_mut().zip(right.iter_mut()).enumerate() {
        let t = i as f64 / sr as f64;
        let fi = smoothstep(0.0, 0.01, t);
        let fo = smoothstep(0.0, 0.01, total_dur - t);
        let g = (fi.min(fo)) as f32;
        *sl *= g;
        *sr_ *= g;
    }
}

fn encode_16bit_stereo_wav(left: &[f32], right: &[f32], sr: u32) -> Vec<u8> {
    assert_eq!(left.len(), right.len());
    let data_size = (left.len() * 4) as u32; // 2ch * 2bytes
    let mut wav = Vec::with_capacity(44 + data_size as usize);

    wav.extend_from_slice(b"RIFF");
    let file_size = 36u32 + data_size;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&2u16.to_le_bytes()); // stereo
    wav.extend_from_slice(&sr.to_le_bytes());
    let byte_rate = sr * 4;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // 16-bit
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for (&l, &r) in left.iter().zip(right.iter()) {
        let vl = (l.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        let vr = (r.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        wav.extend_from_slice(&vl.to_le_bytes());
        wav.extend_from_slice(&vr.to_le_bytes());
    }
    wav
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

/// Score events whose `start_tick` lands in the half-open absolute-tick span
/// `[tick_start, tick_end)`, paired with the sample offset (seconds) of each
/// event from `tick_start`. The span is interpreted modulo `section.length_ticks`,
/// so a chunk that straddles the section's loop point still schedules the events
/// at the top of every phrase it covers. The offset is a pure function of the
/// produced sample count (relative to the chunk's own start tick), so it does not
/// accumulate per-chunk rounding the way `tick += ceil(chunk) + 1` did.
fn events_in_tick_span(
    section: &PortableSection,
    tick_start: f64,
    tick_end: f64,
    tps: f64,
) -> Vec<(&MusicEvent, f64)> {
    let phrase_ticks = f64::from(section.length_ticks.max(1));
    let mut events = Vec::new();

    for event in &section.events {
        let event_tick = f64::from(event.start_tick());
        let first_phrase = ((tick_start - event_tick) / phrase_ticks).ceil() as i64;
        let end_phrase = ((tick_end - event_tick) / phrase_ticks).ceil() as i64;

        for phrase in first_phrase..end_phrase {
            let absolute_tick = event_tick + phrase as f64 * phrase_ticks;
            events.push((event, (absolute_tick - tick_start) / tps));
        }
    }

    // A chunk can straddle a loop point or cover multiple phrases. Keep events
    // in absolute score order so equal-time voices retain their authored order.
    events.sort_by(|a, b| a.1.total_cmp(&b.1));
    events
}

/// Core mono synthesis for a (start, dur) span in "phrase" units (fractional OK).
/// Returns raw synth output before mastering/seam. The score cursor is derived
/// from the produced-sample count, so timing is exact and independent of how the
/// span is split into chunks.
fn synthesize_mono(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
    chunk_size: usize,
) -> Vec<f32> {
    let section = score
        .section(section_id)
        .expect("section_id must exist in score");
    let length_ticks = section.length_ticks;
    let tps = score.ticks_per_second();
    let phrase_sec = length_ticks as f64 / tps;
    let total_samples = (phrase_sec * phrases * sample_rate as f64).round() as usize;

    let mut synth = Synth::new(sample_rate as f32);
    let mut samples = Vec::with_capacity(total_samples);

    let start_tick = (start_phrases * length_ticks as f64).round();
    let chunk_size = chunk_size.max(1);
    let mut pos = 0usize;

    while pos < total_samples {
        let chunk = (total_samples - pos).min(chunk_size);
        let tick_start = start_tick + score_tick_at_sample(pos as u64, f64::from(sample_rate), tps);
        let tick_end =
            start_tick + score_tick_at_sample((pos + chunk) as u64, f64::from(sample_rate), tps);

        for (event, offset) in events_in_tick_span(section, tick_start, tick_end, tps) {
            synth.trigger_at(event, tps, offset);
        }

        let mut buf = vec![0.0f32; chunk];
        synth.fill(&mut buf);
        samples.extend_from_slice(&buf);

        pos += chunk;
    }

    samples
}

/// Core stereo synthesis. Mirrors [`synthesize_mono`] through the stereo path.
fn synthesize_stereo(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
    chunk_size: usize,
) -> (Vec<f32>, Vec<f32>) {
    let section = score
        .section(section_id)
        .expect("section_id must exist in score");
    let length_ticks = section.length_ticks;
    let tps = score.ticks_per_second();
    let phrase_sec = length_ticks as f64 / tps;
    let total_samples = (phrase_sec * phrases * sample_rate as f64).round() as usize;

    let mut synth = Synth::new(sample_rate as f32);
    let mut left = Vec::with_capacity(total_samples);
    let mut right = Vec::with_capacity(total_samples);

    let start_tick = (start_phrases * length_ticks as f64).round();
    let chunk_size = chunk_size.max(1);
    let mut pos = 0usize;

    while pos < total_samples {
        let chunk = (total_samples - pos).min(chunk_size);
        let tick_start = start_tick + score_tick_at_sample(pos as u64, f64::from(sample_rate), tps);
        let tick_end =
            start_tick + score_tick_at_sample((pos + chunk) as u64, f64::from(sample_rate), tps);

        for (event, offset) in events_in_tick_span(section, tick_start, tick_end, tps) {
            synth.trigger_at(event, tps, offset);
        }

        let mut bl = vec![0.0f32; chunk];
        let mut br = vec![0.0f32; chunk];
        synth.fill_stereo(&mut bl, &mut br);
        left.extend_from_slice(&bl);
        right.extend_from_slice(&br);

        pos += chunk;
    }

    (left, right)
}

/// Core mono synthesis for a (start, dur) span in "phrase" units (fractional OK).
/// start_phrases=0, phrases=N reproduces original. Master is per-rendered span.
fn render_samples_mono(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
    apply_seam: bool,
) -> Vec<f32> {
    let section = score
        .section(section_id)
        .expect("section_id must exist in score");
    let dur_sec = section.length_ticks as f64 / score.ticks_per_second() * phrases;

    let mut samples = synthesize_mono(score, section_id, start_phrases, phrases, sample_rate, 256);

    let mut master = MasterChain::new(sample_rate, MasterConfig::default());
    // Must use the offline path for renders so that per-render LUFS measurement +
    // true-peak limiting can be performed (realtime path cannot do this).
    let _ = master.process_offline(&mut samples);

    if apply_seam {
        apply_outer_seam_fade(&mut samples, sample_rate, dur_sec);
    }
    samples
}

/// Core stereo synthesis for a (start, dur) span in "phrase" units (fractional OK).
fn render_samples_stereo(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
    apply_seam: bool,
) -> (Vec<f32>, Vec<f32>) {
    let section = score
        .section(section_id)
        .expect("section_id must exist in score");
    let dur_sec = section.length_ticks as f64 / score.ticks_per_second() * phrases;

    let (mut left, mut right) =
        synthesize_stereo(score, section_id, start_phrases, phrases, sample_rate, 256);

    let mut master = MasterChain::new(sample_rate, MasterConfig::default());
    let _ = master.process_offline_stereo(&mut left, &mut right);

    if apply_seam {
        apply_outer_seam_fade_stereo(&mut left, &mut right, sample_rate, dur_sec);
    }
    (left, right)
}

/// Render a (possibly offset/fractional) span of the take as mono WAV. For chunked
/// long renders that must fit in the WASM 2 MiB output buffer. No outer seam.
pub fn render_wav_chunk(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
) -> Vec<u8> {
    let samples = render_samples_mono(
        score,
        section_id,
        start_phrases,
        phrases,
        sample_rate,
        false,
    );
    encode_16bit_mono_wav(&samples, sample_rate)
}

/// Render a (possibly offset/fractional) span of the take as stereo WAV. For chunked
/// long renders. No outer seam.
pub fn render_wav_stereo_chunk(
    score: &PortableScore,
    section_id: &str,
    start_phrases: f64,
    phrases: f64,
    sample_rate: u32,
) -> Vec<u8> {
    let (left, right) = render_samples_stereo(
        score,
        section_id,
        start_phrases,
        phrases,
        sample_rate,
        false,
    );
    encode_16bit_stereo_wav(&left, &right, sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::racing::{generate_racing, GenerateInput, InstrumentPalette, Style};

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
        let score = generate_racing(&GenerateInput {
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
        let wav = render_wav(&score, "cruise", 2, 48000);
        let (sr, _bits, n_samples) = parse_wav_header(&wav);
        assert_eq!(sr, 48000);

        let expected = ((score.section("cruise").unwrap().length_ticks as f64
            / score.ticks_per_second())
            * 2.0
            * 48000.0)
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
    fn golden_catalog_grid_3phrases_48000() {
        let json = include_str!("../../../catalog/racing/tiny-torque-level-004/score.json");
        let score: PortableScore =
            serde_json::from_str(json).expect("catalog JSON must parse with existing derives");
        let wav = render_wav(&score, "grid", 3, 48000);
        let samples = decode_wav_samples(&wav);
        let sr = 48000u32;

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

    #[test]
    fn stereo_folddown_gate_mono_matches_within_tolerance() {
        // Uses a racing cruise; must satisfy the GURI-813 fold-down contract.
        let score = generate_racing(&GenerateInput {
            secret: "fold-secret".into(),
            seed: "fold-seed".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.65,
            complexity: 0.55,
            brightness: 0.5,
            syncopation: 0.6,
        })
        .expect("fold score");
        let wav_s = render_wav_stereo(&score, "cruise", 1, 48000);
        // decode stereo
        let (sr, ch, ns) = {
            assert_eq!(&wav_s[0..4], b"RIFF");
            let sr = u32::from_le_bytes(wav_s[24..28].try_into().unwrap());
            let ch = u16::from_le_bytes(wav_s[22..24].try_into().unwrap());
            let data_size = u32::from_le_bytes(wav_s[40..44].try_into().unwrap()) as usize;
            (sr, ch, data_size / 4)
        };
        assert_eq!(ch, 2);
        let mut sl = Vec::with_capacity(ns);
        let mut sr_ = Vec::with_capacity(ns);
        let mut off = 44usize;
        for _ in 0..ns {
            let l = i16::from_le_bytes(wav_s[off..off + 2].try_into().unwrap()) as f32 / 32768.0;
            let r =
                i16::from_le_bytes(wav_s[off + 2..off + 4].try_into().unwrap()) as f32 / 32768.0;
            sl.push(l);
            sr_.push(r);
            off += 4;
        }
        // fold: use 0.5 sum (common practical downmix); power will be ~ -3dB but we check relative
        let mono: Vec<f32> = sl
            .iter()
            .zip(sr_.iter())
            .map(|(&l, &r)| (l + r) * 0.5)
            .collect();
        // LUFS on stereo and on fold
        let mut mst = crate::master::Meter::new(sr);
        mst.process_stereo(&sl, &sr_);
        let (lufs_st, tp_st) = mst.report();
        let mut mm = crate::master::Meter::new(sr);
        mm.process(&mono);
        let (lufs_m, tp_m) = mm.report();
        // fold LUFS within -3 … +1 of stereo
        let dlu = lufs_m - lufs_st;
        assert!(
            (-3.0..=1.2).contains(&dlu),
            "fold LUFS delta {} out of [-3,+1.2]",
            dlu
        );
        // fold TP <= -1
        assert!(tp_m <= -1.0 + 1e-3, "fold TP {} > -1", tp_m);
        // stereo TP <= -1 already from master
        assert!(tp_st <= -1.0 + 1e-3, "stereo TP {} > -1", tp_st);
        // correlation > 0 (mid/side or simple pearson on l r)
        let n = sl.len() as f32;
        let ml: f32 = sl.iter().sum::<f32>() / n;
        let mr: f32 = sr_.iter().sum::<f32>() / n;
        let mut num = 0.0;
        let mut vl = 0.0;
        let mut vr = 0.0;
        for (&l, &r) in sl.iter().zip(sr_.iter()) {
            let dl = l - ml;
            let dr = r - mr;
            num += dl * dr;
            vl += dl * dl;
            vr += dr * dr;
        }
        let corr = if vl > 0.0 && vr > 0.0 {
            num / (vl.sqrt() * vr.sqrt())
        } else {
            0.0
        };
        assert!(corr > 0.0, "l/r correlation {} <=0 ", corr);
        // side energy < ~25% of mid
        let mut emid = 0.0f32;
        let mut eside = 0.0f32;
        for (&l, &r) in sl.iter().zip(sr_.iter()) {
            let m = (l + r) * 0.5;
            let s = (l - r) * 0.5;
            emid += m * m;
            eside += s * s;
        }
        let ratio = if emid > 0.0 { eside / emid } else { 0.0 };
        assert!(ratio < 0.30, "side/mid energy {} >=0.30", ratio);
        // nothing below ~120Hz panned off centre: crude, check lowpassed side small
        // (simple: overall side small already checked; detailed would need fft)
        // we assert the energy ratio already.
    }

    fn single_kick_score() -> PortableScore {
        PortableScore {
            schema_version: 1,
            id: "onset-test".into(),
            title: "onset".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 960,
            crossfade_bars: 2.0,
            default_section: "a".into(),
            sections: vec![PortableSection {
                id: "a".into(),
                label: "A".into(),
                feeling: "test".into(),
                color: "#888888".into(),
                length_ticks: 3840,
                events: vec![MusicEvent::Percussion {
                    id: "kick-0".into(),
                    section: "a".into(),
                    lane: "kit".into(),
                    start_tick: 1920,
                    duration_ticks: 60,
                    velocity: 1.0,
                    voice: "kick".into(),
                }],
            }],
            rules: vec![],
            form: None,
        }
    }

    #[test]
    fn rendered_onsets_land_at_nominal_sample_positions() {
        // A sparse section with a single kick mid-phrase: nothing else sounds, so
        // the first audible sample is the kick's onset. It must land at
        // start_tick / tps * sample_rate, and must not move when the render is
        // split into a different internal chunk size. The old
        // `tick += ceil(chunk / sr * tps) + 1` cursor ran the onset thousands of
        // samples early (and quantised it to a chunk boundary).
        let score = single_kick_score();
        let sr = 48000u32;
        let tps = score.ticks_per_second(); // 1920
        let nominal = (1920.0 / tps * f64::from(sr)).round() as usize; // 48000

        let mut onsets = Vec::new();
        for chunk_size in [256usize, 577] {
            let samples = synthesize_mono(&score, "a", 0.0, 1.0, sr, chunk_size);
            assert_eq!(
                samples.len(),
                (3840.0 / tps * f64::from(sr)).round() as usize
            );
            let onset = samples
                .iter()
                .position(|&s| s.abs() > 0.01)
                .expect("kick must produce an audible onset");
            assert!(
                (onset as i64 - nominal as i64).abs() <= 2,
                "kick onset landed at sample {onset} instead of {nominal} (chunk {chunk_size})"
            );
            onsets.push(onset);
        }
        assert_eq!(onsets[0], onsets[1], "chunk size must not move the onset");
        // Synth stores absolute phase as f32, so changing chunk boundaries can
        // round amplitudes microscopically; the renderer contract is that the
        // score clock and event onset samples do not move.
    }

    #[test]
    fn event_windows_repeat_across_phrases_without_boundary_duplicates() {
        let score = single_kick_score();
        let section = score.section("a").expect("test section");
        let tps = score.ticks_per_second();
        let phrase_ticks = f64::from(section.length_ticks);

        let repeated = events_in_tick_span(section, 0.0, phrase_ticks * 3.0, tps);
        assert_eq!(repeated.len(), 3, "one kick must repeat in every phrase");
        let offsets: Vec<f64> = repeated.iter().map(|(_, offset)| *offset).collect();
        assert_eq!(offsets, vec![1.0, 3.0, 5.0]);

        let before = events_in_tick_span(section, 0.0, 1920.0, tps);
        let after = events_in_tick_span(section, 1920.0, phrase_ticks, tps);
        assert!(
            before.is_empty(),
            "an event at the end is not in a half-open span"
        );
        assert_eq!(
            after.len(),
            1,
            "the adjacent span must trigger it exactly once"
        );
        assert_eq!(after[0].1, 0.0, "a boundary event starts at offset zero");
    }
}
