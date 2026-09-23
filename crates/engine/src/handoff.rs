use crate::score::PortableScore;

pub fn crossfade_gains(progress: f32) -> (f32, f32) {
    let p = progress.clamp(0.0, 1.0);
    (1.0 - p, p)
}

pub fn crossfade_sample_count(score: &PortableScore, sample_rate: u32) -> u64 {
    let bar_ticks = score.bar_ticks();
    let ticks = (score.crossfade_bars * f64::from(bar_ticks)).round() as u32;
    let ticks = ticks.max(bar_ticks);
    let samples =
        (f64::from(ticks) / score.ticks_per_second() * f64::from(sample_rate)).round() as u64;
    samples.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::racing::{generate_racing, GenerateInput, InstrumentPalette, Style};

    #[test]
    fn test_crossfade_gains() {
        assert_eq!(crossfade_gains(0.0), (1.0, 0.0));
        assert_eq!(crossfade_gains(0.5), (0.5, 0.5));
        assert_eq!(crossfade_gains(1.0), (0.0, 1.0));

        assert_eq!(crossfade_gains(-1.0), (1.0, 0.0));
        assert_eq!(crossfade_gains(2.0), (0.0, 1.0));
    }

    #[test]
    fn test_crossfade_sample_count() {
        let score = generate_racing(&GenerateInput {
            secret: "handoff-secret".into(),
            seed: "handoff-seed".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.5,
            complexity: 0.5,
            brightness: 0.5,
            syncopation: 0.5,
        })
        .unwrap();

        let sample_rate = 44100;
        let samples = crossfade_sample_count(&score, sample_rate);

        let bar_ticks = score.bar_ticks();
        let expected_ticks = bar_ticks * 2;
        let expected_samples = (f64::from(expected_ticks) / score.ticks_per_second()
            * f64::from(sample_rate))
        .round() as u64;

        assert_eq!(samples, expected_samples);
        assert!(samples >= 1);
    }
}
