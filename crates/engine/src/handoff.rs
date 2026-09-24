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

/// Linear amplitude above which a rendered buffer counts as sounding, about
/// -60 dBFS. A genuinely silent voice renders exact zeros, and mastered music
/// peaks near the -1 dBFS ceiling, so this sits far below any real signal and
/// far above digital silence.
pub const SOUNDING_THRESHOLD: f32 = 0.001;

/// Whether `buffer` has produced audible signal: any sample whose magnitude
/// exceeds [`SOUNDING_THRESHOLD`].
pub fn buffer_is_sounding(buffer: &[f32]) -> bool {
    buffer.iter().any(|sample| sample.abs() > SOUNDING_THRESHOLD)
}

/// Progress state for a crossfade between an outgoing and an incoming voice.
///
/// The player owns one of these for the lifetime of a handoff. Re-targeting a
/// score mid-fade (a second `generate()` before the fade ends) must not restart
/// the fade: the outgoing gain would snap back to `1.0` and click. `begin` and
/// `retarget` therefore differ — `begin` starts a fade at zero, while `retarget`
/// keeps the running counter and only re-fixes the total.
///
/// A handoff that begins while the incoming voice is still silent (for example
/// a score that opens on its sparse section) does not fade the outgoing down
/// into that silence. It first *holds*: the outgoing stays at full gain and the
/// incoming silent until the incoming's rendered buffer first sounds above
/// [`SOUNDING_THRESHOLD`], at which point the crossfade starts from progress
/// zero. The hold is bounded by the fade length itself (`samples_total`) so a
/// never-sounding incoming cannot hold forever; the fade then runs anyway.
#[derive(Clone, Copy, Debug, Default)]
pub struct Handoff {
    samples_done: u64,
    samples_total: u64,
    /// True while the incoming voice has not yet sounded and the outgoing is
    /// held at full gain instead of fading.
    waiting: bool,
    /// Samples spent waiting for the incoming to sound. Bounded by
    /// `samples_total`.
    wait_elapsed: u64,
}

impl Handoff {
    /// A handoff that has not started (no outgoing voice, no crossfade).
    pub const fn idle() -> Self {
        Self {
            samples_done: 0,
            samples_total: 0,
            waiting: false,
            wait_elapsed: 0,
        }
    }

    /// Whether a crossfade is currently in progress (an outgoing voice is
    /// fading or being held for the incoming's first sound).
    pub fn is_active(&self) -> bool {
        self.samples_total > 0
    }

    /// Whether the handoff is holding the outgoing at full gain because the
    /// incoming voice has not yet produced signal.
    pub fn is_waiting(&self) -> bool {
        self.waiting && self.samples_total > 0
    }

    /// Begin a handoff from a non-handoff state: the incoming voice has just
    /// replaced the playing one, so the outgoing is held at full gain until the
    /// incoming sounds (or the hold bound elapses), then fades from zero.
    pub fn begin(&mut self, samples_total: u64) {
        self.samples_done = 0;
        self.samples_total = samples_total;
        self.waiting = true;
        self.wait_elapsed = 0;
    }

    /// Replace the incoming voice while a crossfade is already running. The
    /// fade continues from where it is: the running counter and any hold clock
    /// are preserved so the outgoing gain does not snap back to `1.0` and a
    /// still-silent incoming does not restart the hold bound. The total is
    /// re-affirmed from the (unchanged) outgoing score's fade length.
    pub fn retarget(&mut self, samples_total: u64) {
        self.samples_total = samples_total;
    }

    /// Observe the incoming voice's rendered buffer for this frame. While
    /// waiting, this either keeps holding (the incoming is still silent) or
    /// starts the fade at zero because the buffer sounds above
    /// [`SOUNDING_THRESHOLD`], or because the elapsed wait reached the hold
    /// bound (`samples_total`). Once fading, calls are a no-op.
    pub fn poll(&mut self, incoming: &[f32]) {
        if !self.is_waiting() {
            return;
        }
        let frames = incoming.len() as u64;
        let bound_reached = self.wait_elapsed.saturating_add(frames) >= self.samples_total;
        if buffer_is_sounding(incoming) || bound_reached {
            self.waiting = false;
            self.wait_elapsed = 0;
            self.samples_done = 0;
        }
    }

    /// Advance by `frames` mixed samples. While holding for the incoming's
    /// first sound this advances the hold clock; while fading it advances the
    /// fade counter.
    pub fn advance(&mut self, frames: u64) {
        if self.waiting {
            self.wait_elapsed = self.wait_elapsed.saturating_add(frames);
        } else {
            self.samples_done = self.samples_done.saturating_add(frames);
        }
    }

    /// Linear crossfade progress in `[0, 1]` at sample offset `offset` within
    /// the buffer being mixed. Pinned at `0.0` while holding for the incoming's
    /// first sound, so the gains stay at `(1, 0)`.
    pub fn progress(&self, offset: u64) -> f32 {
        if self.samples_total == 0 || self.waiting {
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
        self.waiting = false;
        self.wait_elapsed = 0;
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

    /// A one-sample buffer whose magnitude is above [`SOUNDING_THRESHOLD`]: the
    /// incoming voice has produced signal.
    fn sounding() -> [f32; 1] {
        [SOUNDING_THRESHOLD * 2.0]
    }

    #[test]
    fn retargeting_a_mid_fade_handoff_does_not_restart_the_fade() {
        let mut handoff = Handoff::idle();
        handoff.begin(4800);
        handoff.poll(&sounding()); // the incoming sounds, so the fade starts
        handoff.advance(2400);

        // Halfway through the fade.
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
        handoff.poll(&sounding());
        handoff.advance(80);
        assert!((crossfade_gains(handoff.progress(0)).0 - 0.2).abs() < 1e-6);

        // The previous fade completed, then a brand-new handoff starts.
        handoff.reset();
        handoff.begin(200);
        handoff.poll(&sounding());

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
        handoff.poll(&sounding());

        handoff.advance(120); // overshoot past the total
        assert_eq!(handoff.progress(0), 1.0);
        assert_eq!(handoff.progress(50), 1.0);
        assert!(handoff.finished());
    }

    #[test]
    fn while_waiting_for_the_incoming_to_sound_the_gains_hold_at_one_and_zero() {
        let mut handoff = Handoff::idle();
        handoff.begin(1000);
        assert!(handoff.is_waiting());

        // The incoming renders silence for several buffers: the hold continues
        // and progress stays pinned at zero, so the gains stay (1, 0).
        for _ in 0..3 {
            handoff.poll(&[0.0f32; 64]);
            handoff.advance(64);
            assert!(handoff.is_waiting());
            for offset in [0u64, 32, 64] {
                assert_eq!(crossfade_gains(handoff.progress(offset)), (1.0, 0.0));
            }
        }
        // The fade counter has not advanced while holding.
        assert!(!handoff.finished());
    }

    #[test]
    fn a_sounding_incoming_starts_the_fade_at_zero_and_it_completes() {
        let mut handoff = Handoff::idle();
        handoff.begin(100);

        // The incoming's first frame is silent...
        handoff.poll(&[0.0f32; 8]);
        handoff.advance(8);
        assert!(handoff.is_waiting());

        // ...then a later frame contains signal above the threshold.
        let mut incoming = [0.0f32; 8];
        incoming[5] = SOUNDING_THRESHOLD * 2.0;
        handoff.poll(&incoming);
        assert!(!handoff.is_waiting());
        // The fade starts at zero progress.
        assert_eq!(handoff.progress(0), 0.0);

        // Advance to completion: the outgoing fades all the way out.
        handoff.advance(100);
        assert!(handoff.finished());
        assert_eq!(crossfade_gains(handoff.progress(0)), (0.0, 1.0));
    }

    #[test]
    fn a_never_sounding_incoming_starts_the_fade_once_the_hold_bound_expires() {
        let mut handoff = Handoff::idle();
        let total = 100;
        handoff.begin(total);

        // Keep rendering silence: the hold must end on its own, at most a buffer
        // past the bound, rather than holding forever.
        let mut waited = 0u64;
        while handoff.is_waiting() {
            handoff.poll(&[0.0f32; 16]);
            handoff.advance(16);
            waited += 16;
            assert!(waited <= total + 16, "the hold outran its bound");
        }
        assert!(!handoff.is_waiting());
        assert!(waited <= total + 16);

        // The fade then runs to completion like any other handoff.
        while !handoff.finished() {
            handoff.advance(16);
        }
        assert_eq!(crossfade_gains(handoff.progress(0)), (0.0, 1.0));
    }
}
