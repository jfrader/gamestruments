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

/// Progress state for a crossfade between an outgoing and an incoming voice.
///
/// The player owns one of these for the lifetime of a handoff. Re-targeting a
/// score mid-fade (a second `generate()` before the fade ends) must not restart
/// the fade: the outgoing gain would snap back to `1.0` and click. `begin` and
/// `retarget` therefore differ — `begin` starts a fade at zero, while `retarget`
/// keeps the running counter and only re-fixes the total.
#[derive(Clone, Copy, Debug, Default)]
pub struct Handoff {
    samples_done: u64,
    samples_total: u64,
}

impl Handoff {
    /// A handoff that has not started (no outgoing voice, no crossfade).
    pub const fn idle() -> Self {
        Self {
            samples_done: 0,
            samples_total: 0,
        }
    }

    /// Whether a crossfade is currently in progress (an outgoing voice is fading).
    pub fn is_active(&self) -> bool {
        self.samples_total > 0
    }

    /// Begin a crossfade from a non-handoff state: the fade starts at zero.
    pub fn begin(&mut self, samples_total: u64) {
        self.samples_done = 0;
        self.samples_total = samples_total;
    }

    /// Replace the incoming voice while a crossfade is already running. The
    /// fade continues from where it is: the running counter is preserved so the
    /// outgoing gain does not snap back to `1.0`. The total is re-affirmed from
    /// the (unchanged) outgoing score's fade length.
    pub fn retarget(&mut self, samples_total: u64) {
        self.samples_total = samples_total;
    }

    /// Advance the fade by `frames` mixed samples.
    pub fn advance(&mut self, frames: u64) {
        self.samples_done = self.samples_done.saturating_add(frames);
    }

    /// Linear crossfade progress in `[0, 1]` at sample offset `offset` within
    /// the buffer being mixed.
    pub fn progress(&self, offset: u64) -> f32 {
        if self.samples_total == 0 {
            0.0
        } else {
            ((self.samples_done.saturating_add(offset)) as f32 / self.samples_total as f32)
                .clamp(0.0, 1.0)
        }
    }

    /// True when the fade has reached its end and the outgoing voice can drop.
    pub fn finished(&self) -> bool {
        self.samples_total > 0 && self.samples_done >= self.samples_total
    }

    /// Reset to idle after the outgoing voice is dropped.
    pub fn reset(&mut self) {
        self.samples_done = 0;
        self.samples_total = 0;
    }
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

    #[test]
    fn retargeting_a_mid_fade_handoff_does_not_restart_the_fade() {
        let mut handoff = Handoff::idle();
        handoff.begin(4800);

        // Halfway through the fade.
        handoff.advance(2400);
        let (g_out_mid, _) = crossfade_gains(handoff.progress(0));
        assert!((g_out_mid - 0.5).abs() < 1e-6);

        // A second generate() arrives mid-fade: the incoming voice is replaced,
        // but the outgoing fade must continue from where it is.
        handoff.retarget(4800);

        let (g_out_after, _) = crossfade_gains(handoff.progress(0));
        assert!(
            (g_out_after - g_out_mid).abs() < 1e-6,
            "outgoing gain jumped from {g_out_mid} to {g_out_after} on retarget"
        );
        assert!(
            g_out_after < 1.0,
            "outgoing gain snapped back to 1.0: the fade restarted"
        );
    }

    #[test]
    fn beginning_a_fresh_handoff_starts_the_fade_at_zero() {
        let mut handoff = Handoff::idle();
        handoff.begin(100);
        handoff.advance(80);
        assert!((crossfade_gains(handoff.progress(0)).0 - 0.2).abs() < 1e-6);

        // The previous fade completed, then a brand-new handoff starts.
        handoff.reset();
        handoff.begin(200);

        let (g_out, _) = crossfade_gains(handoff.progress(0));
        assert_eq!(g_out, 1.0, "a fresh handoff must begin at full outgoing gain");
    }

    #[test]
    fn progress_is_clamped_and_finished_detects_the_end() {
        let mut handoff = Handoff::idle();
        assert!(!handoff.is_active());
        assert_eq!(handoff.progress(0), 0.0);

        handoff.begin(100);
        assert!(handoff.is_active());

        handoff.advance(120); // overshoot past the total
        assert_eq!(handoff.progress(0), 1.0);
        assert_eq!(handoff.progress(50), 1.0);
        assert!(handoff.finished());
    }
}
