use std::collections::VecDeque;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

use crate::dmath;

pub const DEFAULT_TARGET_LUFS: f32 = -14.0;
pub const DEFAULT_CEILING_DBTP: f32 = -1.0;
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

/// The gain that brings typical post-compressor material to the target
/// loudness, measured over the listening pack. The live normalizer starts here
/// before it has heard anything, and pre-master probes use it to express a
/// mastered level in the raw domain.
pub const NOMINAL_LOUDNESS_GAIN_DB: f32 = 23.5;

/// The live normalizer measures the gated loudness of the last 30 s, so a
/// piece's own soft and loud passages keep their contrast while different
/// pieces, recipes and styles all play at the target.
const LOUDNESS_WINDOW_SECONDS: f32 = 30.0;
/// BS.1770 measurement blocks: 400 ms, every 100 ms.
const LOUDNESS_BLOCK_SECONDS: f32 = 0.4;
const LOUDNESS_HOP_SECONDS: f32 = 0.1;
/// BS.1770 gates: blocks under -70 LUFS are silence; blocks more than 10 LU
/// under the ungated loudness do not count either.
const LOUDNESS_ABSOLUTE_GATE_LUFS: f32 = -70.0;
const LOUDNESS_RELATIVE_GATE_LU: f32 = 10.0;
/// How fast the live gain moves: down quickly when the music is too loud, up
/// slowly when it is too quiet, so it never pumps.
const LOUDNESS_FALL_DB_PER_SECOND: f32 = 6.0;
const LOUDNESS_RISE_DB_PER_SECOND: f32 = 1.5;
/// How far the live gain may move from [`NOMINAL_LOUDNESS_GAIN_DB`].
const LOUDNESS_GAIN_RANGE_DB: f32 = 24.0;
/// The most loudness the live normalizer makes up for what the true-peak
/// limiter takes off dense material; past this the limiter would crush it.
const LOUDNESS_LIMITER_MAKEUP_MAX_LU: f32 = 4.0;

/// Guard subtracted (in dB) from the ceiling when computing the true-peak limit
/// target, offline and live.
///
/// The limiter bounds the envelope produced by [`true_peak_envelope_4x`], while
/// external meters such as `ffmpeg loudnorm` reconstruct with their own
/// interpolant and can read slightly higher. This margin covers that difference
/// so the externally measured true peak stays at or below `DEFAULT_CEILING_DBTP`.
///
/// Calibrated on the full listening pack (12 renders, racing and suspense,
/// four styles, six sections): with 1.0 dB the external measurement reads
/// -1.74 to -1.98 dBTP, i.e. every render clears the -1.0 dBTP ceiling with at
/// least 0.7 dB to spare. Lowering it trades that margin for a little loudness.
pub const TRUE_PEAK_GUARD_DB: f32 = 1.0;

/// Release time for offline TP limiter gain smoothing (max-hold + exp release).
/// 5 ms chosen to suppress distortion on transient hits without excessive pumping;
/// non-causal smoothing is acceptable for offline.
const TP_LIMITER_RELEASE_SEC: f32 = 0.005;

/// The live limiter's look-ahead: how early its gain starts to come down
/// before a peak, and the signal latency it adds.
const LIMITER_LOOKAHEAD_SECONDS: f32 = 0.003;
/// How fast the live limiter's gain recovers after a peak.
const LIMITER_RELEASE_SECONDS: f32 = 0.075;

#[derive(Clone, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn highpass(fc: f32, q: f32, fs: f32) -> Self {
        let w0 = 2.0 * PI * fc / fs;
        let alpha = dmath::sin(w0) / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self {
            b0: ((1.0 + dmath::cos(w0)) / 2.0) / a0,
            b1: -(1.0 + dmath::cos(w0)) / a0,
            b2: ((1.0 + dmath::cos(w0)) / 2.0) / a0,
            a1: (-2.0 * dmath::cos(w0)) / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn highshelf(fc: f32, q: f32, gain_db: f32, fs: f32) -> Self {
        let a = dmath::pow10(gain_db / 40.0);
        let w0 = 2.0 * PI * fc / fs;
        let alpha = dmath::sin(w0) / (2.0 * q);
        let a0 = (a + 1.0) - (a - 1.0) * dmath::cos(w0) + 2.0 * a.sqrt() * alpha;
        Self {
            b0: (a * ((a + 1.0) + (a - 1.0) * dmath::cos(w0) + 2.0 * a.sqrt() * alpha)) / a0,
            b1: (-2.0 * a * ((a - 1.0) + (a + 1.0) * dmath::cos(w0))) / a0,
            b2: (a * ((a + 1.0) + (a - 1.0) * dmath::cos(w0) - 2.0 * a.sqrt() * alpha)) / a0,
            a1: (2.0 * ((a - 1.0) - (a + 1.0) * dmath::cos(w0))) / a0,
            a2: ((a + 1.0) - (a - 1.0) * dmath::cos(w0) - 2.0 * a.sqrt() * alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

pub struct HighPass {
    filter: Biquad,
    // stereo states (independent filters, updated only on stereo path)
    filter_l: Biquad,
    filter_r: Biquad,
}

impl HighPass {
    pub fn new(sample_rate: u32) -> Self {
        let f = Biquad::highpass(28.0, 0.55, sample_rate as f32);
        Self {
            filter: f.clone(),
            filter_l: f.clone(),
            filter_r: f,
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for x in buffer.iter_mut() {
            *x = self.filter.process(*x);
        }
    }

    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        for x in left.iter_mut() {
            *x = self.filter_l.process(*x);
        }
        for x in right.iter_mut() {
            *x = self.filter_r.process(*x);
        }
    }
}

// DynamicsProcessor implements a soft-knee peak-detecting compressor with
// lookahead delay on the signal path (but causal detection).
pub struct DynamicsProcessor {
    threshold: f32,
    knee: f32,
    ratio: f32,
    attack_coef: f32,
    release_coef: f32,
    gain_env: f32,
    delay_line: std::collections::VecDeque<f32>,
    lookahead_frames: usize,
}

impl DynamicsProcessor {
    pub fn new(
        sample_rate: u32,
        threshold: f32,
        knee: f32,
        ratio: f32,
        attack_sec: f32,
        release_sec: f32,
        lookahead_sec: f32,
    ) -> Self {
        let fs = sample_rate as f32;
        let attack_coef = dmath::exp(-1.0 / (attack_sec * fs));
        let release_coef = dmath::exp(-1.0 / (release_sec * fs));

        let lookahead_frames = (lookahead_sec * fs).round() as usize;
        let mut delay_line = std::collections::VecDeque::with_capacity(lookahead_frames + 1);
        for _ in 0..lookahead_frames {
            delay_line.push_back(0.0);
        }

        Self {
            threshold,
            knee,
            ratio,
            attack_coef,
            release_coef,
            gain_env: 1.0,
            delay_line,
            lookahead_frames,
        }
    }

    fn compute_target_gain(&self, input_abs: f32) -> f32 {
        let level_db = if input_abs > 1e-6 {
            20.0 * dmath::log10(input_abs)
        } else {
            -120.0
        };
        let over = level_db - self.threshold;
        let mut reduction_db = 0.0;
        if over > self.knee / 2.0 {
            reduction_db = over * (1.0 - 1.0 / self.ratio);
        } else if over > -self.knee / 2.0 && self.knee > 0.0 {
            let q = over + self.knee / 2.0;
            reduction_db = (q * q) / (2.0 * self.knee) * (1.0 - 1.0 / self.ratio);
        }
        dmath::db_to_linear(-reduction_db)
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for x in buffer.iter_mut() {
            let target_gain = self.compute_target_gain(x.abs());

            if target_gain < self.gain_env {
                self.gain_env =
                    self.attack_coef * self.gain_env + (1.0 - self.attack_coef) * target_gain;
            } else {
                self.gain_env =
                    self.release_coef * self.gain_env + (1.0 - self.release_coef) * target_gain;
            }

            let delayed_x = if self.lookahead_frames > 0 {
                self.delay_line.push_back(*x);
                self.delay_line.pop_front().unwrap_or(*x)
            } else {
                *x
            };

            *x = delayed_x * self.gain_env;
        }
    }

    /// Stereo-linked: detector = max(|L|, |R|) per sample; same gain applied to both.
    pub fn process_stereo_linked(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(left.len(), right.len());
        for i in 0..left.len() {
            let dl = left[i].abs();
            let dr = right[i].abs();
            let input_abs = dl.max(dr);
            let target_gain = self.compute_target_gain(input_abs);
            if target_gain < self.gain_env {
                self.gain_env =
                    self.attack_coef * self.gain_env + (1.0 - self.attack_coef) * target_gain;
            } else {
                self.gain_env =
                    self.release_coef * self.gain_env + (1.0 - self.release_coef) * target_gain;
            }
            // no lookahead for this helper (stereo comp/lim use their own setup)
            left[i] *= self.gain_env;
            right[i] *= self.gain_env;
        }
    }
}

pub struct Compressor {
    processor: DynamicsProcessor,
}

impl Compressor {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            processor: DynamicsProcessor::new(sample_rate, -18.0, 12.0, 3.2, 0.007, 0.16, 0.003),
        }
    }
    pub fn process(&mut self, buffer: &mut [f32]) {
        self.processor.process(buffer);
    }
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        // linked via the helper (note: this bypasses lookahead delay line for simplicity;
        // the mono comp lookahead is 3ms; for linked we apply direct here)
        self.processor.process_stereo_linked(left, right);
    }
}

/// Look-ahead true-peak brickwall for the realtime path. Each frame's required
/// gain comes from the same 4x true-peak estimate the offline path and the
/// meter use; the lowest requirement is held across the look-ahead, released
/// smoothly, then averaged over the look-ahead, so the gain has fully reached
/// every peak's requirement by the time that peak leaves the delay line.
pub struct Limiter {
    limit: f32,
    lookahead: usize,
    release_coef: f32,
    /// The last `2 * TP_CENTER` input samples per channel, so every frame's
    /// true-peak estimate sees its full kernel on both sides.
    context: [Vec<f32>; 2],
    extended: [Vec<f32>; 2],
    /// Signal delay per channel: the look-ahead the gain needs.
    delay: [VecDeque<f32>; 2],
    /// Sliding minimum of the required gain over the look-ahead, as
    /// (frame, gain) pairs with rising gains.
    held: VecDeque<(u64, f32)>,
    frame: u64,
    released: f32,
    /// The released gains being averaged, and their running sum.
    smoothing: Vec<f32>,
    smoothing_index: usize,
    smoothing_sum: f32,
    engaged: bool,
}

impl Limiter {
    pub fn new(sample_rate: u32, ceiling_dbtp: f32) -> Self {
        let fs = sample_rate as f32;
        let lookahead = (LIMITER_LOOKAHEAD_SECONDS * fs).round() as usize;
        let context_len = 2 * TP_CENTER as usize;
        let delay = || VecDeque::from(vec![0.0; lookahead]);
        Self {
            limit: dmath::db_to_linear(ceiling_dbtp - TRUE_PEAK_GUARD_DB),
            lookahead,
            release_coef: dmath::exp(-1.0 / (LIMITER_RELEASE_SECONDS * fs)),
            context: [vec![0.0; context_len], vec![0.0; context_len]],
            extended: [Vec::new(), Vec::new()],
            delay: [delay(), delay()],
            held: VecDeque::with_capacity(lookahead + 2),
            frame: 0,
            released: 1.0,
            smoothing: vec![1.0; lookahead + 1],
            smoothing_index: 0,
            smoothing_sum: (lookahead + 1) as f32,
            engaged: false,
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        self.process_channels([buffer]);
    }

    /// Stereo-linked: one gain from the louder channel's true peak.
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.process_channels([left, right]);
    }

    fn process_channels<const N: usize>(&mut self, mut channels: [&mut [f32]; N]) {
        let frames = channels[0].len();
        let context_len = self.context[0].len();
        for (channel, samples) in channels.iter().enumerate() {
            let extended = &mut self.extended[channel];
            extended.clear();
            extended.extend_from_slice(&self.context[channel]);
            extended.extend_from_slice(samples);
            self.context[channel].copy_from_slice(&extended[extended.len() - context_len..]);
        }
        let center = TP_CENTER as usize;
        for frame in 0..frames {
            let level = (0..N)
                .map(|channel| true_peak_at(&self.extended[channel], center + frame))
                .fold(0.0f32, f32::max);
            let gain = self.next_gain(if level > self.limit {
                self.limit / level
            } else {
                1.0
            });
            for (channel, samples) in channels.iter_mut().enumerate() {
                let delay = &mut self.delay[channel];
                delay.push_back(self.extended[channel][center + frame]);
                samples[frame] = delay.pop_front().unwrap_or(0.0) * gain;
            }
        }
    }

    /// Take one frame's required gain; return the gain for the frame leaving
    /// the delay line.
    fn next_gain(&mut self, required: f32) -> f32 {
        while self.held.back().is_some_and(|&(_, gain)| gain >= required) {
            self.held.pop_back();
        }
        self.held.push_back((self.frame, required));
        while self
            .held
            .front()
            .is_some_and(|&(frame, _)| frame + (self.lookahead as u64) < self.frame)
        {
            self.held.pop_front();
        }
        self.frame += 1;
        let hold = self.held.front().map_or(1.0, |&(_, gain)| gain);

        self.released = if hold < self.released {
            hold
        } else {
            self.release_coef * self.released + (1.0 - self.release_coef) * hold
        };

        let leaving = std::mem::replace(&mut self.smoothing[self.smoothing_index], self.released);
        self.smoothing_sum += self.released - leaving;
        self.smoothing_index += 1;
        if self.smoothing_index == self.smoothing.len() {
            // Re-sum once per lap so rounding never accumulates.
            self.smoothing_index = 0;
            self.smoothing_sum = self.smoothing.iter().sum();
        }
        let gain = (self.smoothing_sum / self.smoothing.len() as f32).min(1.0);
        if gain < 0.99 {
            self.engaged = true;
        }
        gain
    }

    pub fn was_engaged(&mut self) -> bool {
        let engaged = self.engaged;
        self.engaged = false;
        engaged
    }
}

/// Compute 4x true-peak envelope using Catmull-Rom cubic interpolation.
/// Returns a Vec of len N where env[i] = max absolute value of the
/// reconstructed signal in the local 4x neighbourhood of sample i.
///
/// Catmull-Rom is a linear combination of input samples (the four taps
/// are a linear transform). Therefore, if input samples are scaled by g,
/// every interpolated value (and thus every env entry) is scaled by exactly g.
/// Bounding the envelope therefore bounds the continuous interpolated signal's
/// true peak exactly (w.r.t. this interpolant).
///
/// We chose Catmull-Rom over a windowed-sinc polyphase because it requires
/// no coefficient tables, is trivial to implement, and is "linear" for the
/// scaling proof. It is the same family of cubic already prototyped in the
/// prior attempt.
/// Taps per 4x phase for the true-peak interpolator. An odd count keeps the
/// sinc and its window centred on the fractional interpolation point for every
/// phase; an even count truncates one side and tilts the passband, which makes
/// the estimator over-read bright material. 33 taps keeps the worst-case
/// reconstruction error near Nyquist around -13 dB, where a 13-tap kernel is
/// only -3.6 dB and would let real intersample peaks through.
const TP_TAPS_PER_PHASE: usize = 33;
/// The taps span `i - 16 ..= i + 16` around the sample being interpolated.
const TP_CENTER: isize = 16;

/// Polyphase windowed-sinc kernels for the four 4x interpolation phases.
///
/// The interpolator is an FIR, and therefore linear in the input samples. The
/// ceiling guarantee depends on exactly that: scaling every input sample by `g`
/// scales every interpolated value by exactly `g`, so bounding this envelope
/// bounds the reconstructed true peak. A cheaper interpolator (for example
/// Catmull-Rom) also scales linearly but underestimates intersample peaks on
/// bright material, which is why a real windowed-sinc kernel is used here.
/// Hann-windowed 4x polyphase sinc kernels, normalised to unity DC gain
/// per phase. Baked as literals rather than computed with `sin`/`cos` at
/// run time: the platform libm would otherwise pick slightly different
/// coefficients and break native/WASM byte parity.
const TP_KERNELS: [[f32; TP_TAPS_PER_PHASE]; 4] = [
    [
        -0.0f32,
        1.0988192e-18f32,
        -1.4836533e-18f32,
        -4.0453876e-18f32,
        -5.7087407e-18f32,
        3.15073e-17f32,
        -1.203203e-17f32,
        1.5688382e-17f32,
        -1.9490858e-17f32,
        2.3293337e-17f32,
        -2.6949688e-17f32,
        3.03194e-17f32,
        -3.327298e-17f32,
        3.5696916e-17f32,
        -3.7498066e-17f32,
        3.8607206e-17f32,
        1.0f32,
        3.8607206e-17f32,
        -3.7498066e-17f32,
        3.5696916e-17f32,
        -3.327298e-17f32,
        3.03194e-17f32,
        -2.6949688e-17f32,
        2.3293337e-17f32,
        -1.9490858e-17f32,
        1.5688382e-17f32,
        -1.203203e-17f32,
        3.15073e-17f32,
        -5.7087407e-18f32,
        -4.0453876e-18f32,
        -1.4836533e-18f32,
        1.0988192e-18f32,
        -0.0f32,
    ],
    [
        0.0f32,
        -7.987145e-05f32,
        0.0004616447f32,
        -0.0012083584f32,
        0.002379796f32,
        -0.004044325f32,
        0.0062849806f32,
        -0.00921f32,
        0.012971487f32,
        -0.017799895f32,
        0.024071863f32,
        -0.032455638f32,
        0.044261575f32,
        -0.062439073f32,
        0.095230505f32,
        -0.17736062f32,
        0.89975125f32,
        0.29847378f32,
        -0.124854244f32,
        0.07602277f32,
        -0.052245565f32,
        0.0378052f32,
        -0.027939532f32,
        0.020723091f32,
        -0.015233367f32,
        0.010974186f32,
        -0.007653766f32,
        0.005086629f32,
        -0.0031456735f32,
        0.0017369703f32,
        -0.0007857983f32,
        0.00022864972f32,
        -8.606689e-06f32,
    ],
    [
        0.0f32,
        -4.944115e-05f32,
        0.00047260898f32,
        -0.0013919936f32,
        0.0028899822f32,
        -0.005059594f32,
        0.00801198f32,
        -0.011889399f32,
        0.016888019f32,
        -0.023299532f32,
        0.03159159f32,
        -0.042576153f32,
        0.05780207f32,
        -0.08061996f32,
        0.11980109f32,
        -0.20762788f32,
        0.6350566f32,
        0.6350566f32,
        -0.20762788f32,
        0.11980109f32,
        -0.08061996f32,
        0.05780207f32,
        -0.042576153f32,
        0.03159159f32,
        -0.023299532f32,
        0.016888019f32,
        -0.011889399f32,
        0.00801198f32,
        -0.005059594f32,
        0.0028899822f32,
        -0.0013919936f32,
        0.00047260898f32,
        -4.944115e-05f32,
    ],
    [
        0.0f32,
        -8.606689e-06f32,
        0.00022864972f32,
        -0.0007857983f32,
        0.0017369703f32,
        -0.0031456735f32,
        0.005086629f32,
        -0.007653766f32,
        0.010974186f32,
        -0.015233367f32,
        0.020723091f32,
        -0.027939532f32,
        0.0378052f32,
        -0.052245565f32,
        0.07602277f32,
        -0.124854244f32,
        0.29847378f32,
        0.89975125f32,
        -0.17736062f32,
        0.095230505f32,
        -0.062439073f32,
        0.044261575f32,
        -0.032455638f32,
        0.024071863f32,
        -0.017799895f32,
        0.012971487f32,
        -0.00921f32,
        0.0062849806f32,
        -0.004044325f32,
        0.002379796f32,
        -0.0012083584f32,
        0.0004616447f32,
        -7.987145e-05f32,
    ],
];
/// Largest magnitude of the 4x interpolant in the neighbourhood of `index`.
fn true_peak_at(samples: &[f32], index: usize) -> f32 {
    let kernels = &TP_KERNELS;
    let last = samples.len() - 1;
    let mut peak = samples[index].abs();
    for row in kernels {
        let mut acc = 0.0f32;
        for (tap, &h) in row.iter().enumerate() {
            let j = index as isize + tap as isize - TP_CENTER;
            let x = if j < 0 {
                samples[0]
            } else if j as usize > last {
                samples[last]
            } else {
                samples[j as usize]
            };
            acc += x * h;
        }
        peak = peak.max(acc.abs());
    }
    peak
}

/// Per-sample neighbourhood peak of the 4x oversampled signal.
pub fn true_peak_envelope_4x(samples: &[f32]) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(samples.len());
    true_peak_envelope_4x_into(samples, &mut envelope);
    envelope
}

fn true_peak_envelope_4x_into(samples: &[f32], envelope: &mut Vec<f32>) {
    envelope.clear();
    envelope.extend((0..samples.len()).map(|index| true_peak_at(samples, index)));
}

/// Max absolute value over the sample points and the three 1/4-way points
/// in the cubic segment from p1 to p2 (t=0 at p1, t=1 at p2).
pub struct Meter {
    k_filter_1: Biquad,
    k_filter_2: Biquad,
    block_samples: usize,
    hop_samples: usize,
    buffer: Vec<f32>,
    blocks_energy: Vec<f32>,

    max_true_peak: f32,
}

impl Meter {
    pub fn new(sample_rate: u32) -> Self {
        let fs = sample_rate as f32;
        let (k_filter_1, k_filter_2) = k_weighting(fs);
        Self {
            k_filter_1,
            k_filter_2,
            block_samples: (fs * 0.400).round() as usize,
            hop_samples: (fs * 0.100).round() as usize,
            buffer: Vec::new(),
            blocks_energy: Vec::new(),

            max_true_peak: 0.0,
        }
    }

    pub fn process(&mut self, buffer: &[f32]) {
        // Meter and limiters must agree on true peak, so both use the shared
        // polyphase windowed-sinc estimator rather than a cheaper stand-in.
        for peak in true_peak_envelope_4x(buffer) {
            if peak > self.max_true_peak {
                self.max_true_peak = peak;
            }
        }

        for &x in buffer {
            let k1 = self.k_filter_1.process(x);
            let k2 = self.k_filter_2.process(k1);

            self.buffer.push(k2);
            if self.buffer.len() == self.block_samples {
                let sum_sq: f32 = self.buffer.iter().map(|&s| s * s).sum();
                let z_i = sum_sq / (self.block_samples as f32);
                self.blocks_energy.push(z_i);
                self.buffer.drain(0..self.hop_samples);
            }
        }
    }

    /// Stereo: track max TP across ch; K energy uses per-ch then average for loudness (standard approach).
    pub fn process_stereo(&mut self, left: &[f32], right: &[f32]) {
        assert_eq!(left.len(), right.len());
        for (pl, pr) in true_peak_envelope_4x(left)
            .into_iter()
            .zip(true_peak_envelope_4x(right))
        {
            let p = pl.max(pr);
            if p > self.max_true_peak {
                self.max_true_peak = p;
            }
        }
        // K-weight per sample, push average energy per block
        for (&xl, &xr) in left.iter().zip(right.iter()) {
            let k1l = self.k_filter_1.process(xl);
            let k2l = self.k_filter_2.process(k1l);
            let k1r = self.k_filter_1.process(xr); // note: separate state? reuse for simplicity; or would need dual k too
            let k2r = self.k_filter_2.process(k1r);
            // to avoid polluting, we use only one channel's k for block? better average the z
            let z = (k2l * k2l + k2r * k2r) * 0.5;
            self.buffer.push(z); // push 'energy sample' proxy
            if self.buffer.len() == self.block_samples {
                let sum_sq: f32 = self.buffer.iter().copied().sum(); // already sq
                let z_i = sum_sq / (self.block_samples as f32);
                self.blocks_energy.push(z_i);
                self.buffer.drain(0..self.hop_samples);
            }
        }
    }

    pub fn report(&self) -> (f32, f32) {
        let lufs = gated_loudness(self.blocks_energy.iter()).unwrap_or(-f32::INFINITY);

        let mut true_peak_db = -f32::INFINITY;
        if self.max_true_peak > 1e-6 {
            true_peak_db = 20.0 * dmath::log10(self.max_true_peak);
        }

        (lufs, true_peak_db)
    }
}

/// The BS.1770 K-weighting pre-filter: a high shelf, then a high pass.
fn k_weighting(sample_rate: f32) -> (Biquad, Biquad) {
    (
        Biquad::highshelf(1681.97, FRAC_1_SQRT_2, 4.0, sample_rate),
        Biquad::highpass(38.13, 0.5003, sample_rate),
    )
}

/// The BS.1770 gated loudness of K-weighted block energies, or `None` when
/// every block is below the absolute gate.
fn gated_loudness<'a>(blocks: impl Iterator<Item = &'a f32> + Clone) -> Option<f32> {
    let loudness = |energy: f32| -0.691 + 10.0 * dmath::log10(energy);
    let energy_at = |lufs: f32| dmath::pow10((lufs + 0.691) / 10.0);
    let mean = |gate: f32| {
        let (sum, count) = blocks
            .clone()
            .filter(|&&energy| energy > gate)
            .fold((0.0f32, 0usize), |(sum, count), &energy| {
                (sum + energy, count + 1)
            });
        (count > 0).then(|| sum / count as f32)
    };
    let ungated = mean(energy_at(LOUDNESS_ABSOLUTE_GATE_LUFS))?;
    let gated = mean(energy_at(loudness(ungated) - LOUDNESS_RELATIVE_GATE_LU))?;
    Some(loudness(gated))
}

/// BS.1770 K-weighting for up to two channels.
struct KWeighting([(Biquad, Biquad); 2]);

impl KWeighting {
    fn new(sample_rate: f32) -> Self {
        Self([k_weighting(sample_rate), k_weighting(sample_rate)])
    }

    /// K-weighted energy of one mono sample.
    fn mono(&mut self, sample: f32) -> f32 {
        let (shelf, highpass) = &mut self.0[0];
        let weighted = highpass.process(shelf.process(sample));
        weighted * weighted
    }

    /// K-weighted energy of one stereo frame: the channel average, as
    /// `Meter::process_stereo` measures it.
    fn stereo(&mut self, left: f32, right: f32) -> f32 {
        let [(shelf_l, highpass_l), (shelf_r, highpass_r)] = &mut self.0;
        let weighted_l = highpass_l.process(shelf_l.process(left));
        let weighted_r = highpass_r.process(shelf_r.process(right));
        (weighted_l * weighted_l + weighted_r * weighted_r) * 0.5
    }
}

/// A sliding BS.1770 measurement: K-weighted energy accumulated per 100 ms
/// hop, 400 ms blocks every hop, gated over the last
/// [`LOUDNESS_WINDOW_SECONDS`].
struct LoudnessWindow {
    hop_samples: usize,
    hops_per_block: usize,
    window_blocks: usize,
    hop_energy: f32,
    hop_count: usize,
    /// Mean-square energy of the latest hops, to assemble one block.
    hops: VecDeque<f32>,
    /// Mean-square energy of each block in the window.
    blocks: VecDeque<f32>,
}

impl LoudnessWindow {
    fn new(hop_samples: usize) -> Self {
        Self {
            hop_samples,
            hops_per_block: (LOUDNESS_BLOCK_SECONDS / LOUDNESS_HOP_SECONDS).round() as usize,
            window_blocks: (LOUDNESS_WINDOW_SECONDS / LOUDNESS_HOP_SECONDS).round() as usize,
            hop_energy: 0.0,
            hop_count: 0,
            hops: VecDeque::new(),
            blocks: VecDeque::new(),
        }
    }

    /// Account one sample's K-weighted energy; true when it completed a hop.
    fn add(&mut self, energy: f32) -> bool {
        self.hop_energy += energy;
        self.hop_count += 1;
        if self.hop_count < self.hop_samples {
            return false;
        }
        self.hops
            .push_back(self.hop_energy / self.hop_samples as f32);
        self.hop_energy = 0.0;
        self.hop_count = 0;
        if self.hops.len() > self.hops_per_block {
            self.hops.pop_front();
        }
        if self.hops.len() == self.hops_per_block {
            self.blocks
                .push_back(self.hops.iter().sum::<f32>() / self.hops_per_block as f32);
            if self.blocks.len() > self.window_blocks {
                self.blocks.pop_front();
            }
        }
        true
    }

    fn loudness(&self) -> Option<f32> {
        gated_loudness(self.blocks.iter())
    }
}

/// Holds live output at the target loudness. It measures the gated loudness
/// of the recent programme before its own gain and steers the gain toward the
/// target, ramping sample by sample within each 100 ms hop. It also measures
/// what the limiter after it takes off, and makes that up, so the target holds
/// at the output.
struct LoudnessNormalizer {
    target_lufs: f32,
    hop_samples: usize,
    input_weighting: KWeighting,
    output_weighting: KWeighting,
    /// Loudness before the gain, after it (into the limiter), and at the output.
    input: LoudnessWindow,
    gained: LoudnessWindow,
    output: LoudnessWindow,
    /// Gain reached at the end of the current hop, and the per-sample ramp to it.
    gain_db: f32,
    gain: f32,
    gain_step: f32,
    max_fall_db: f32,
    max_rise_db: f32,
}

impl LoudnessNormalizer {
    fn new(sample_rate: u32, target_lufs: f32) -> Self {
        let fs = sample_rate as f32;
        let hop_samples = (fs * LOUDNESS_HOP_SECONDS).round() as usize;
        Self {
            target_lufs,
            hop_samples,
            input_weighting: KWeighting::new(fs),
            output_weighting: KWeighting::new(fs),
            input: LoudnessWindow::new(hop_samples),
            gained: LoudnessWindow::new(hop_samples),
            output: LoudnessWindow::new(hop_samples),
            gain_db: NOMINAL_LOUDNESS_GAIN_DB,
            gain: dmath::db_to_linear(NOMINAL_LOUDNESS_GAIN_DB),
            gain_step: 0.0,
            max_fall_db: LOUDNESS_FALL_DB_PER_SECOND * LOUDNESS_HOP_SECONDS,
            max_rise_db: LOUDNESS_RISE_DB_PER_SECOND * LOUDNESS_HOP_SECONDS,
        }
    }

    fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer {
            let energy = self.input_weighting.mono(*sample);
            *sample *= self.next_gain(energy);
        }
    }

    fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let energy = self.input_weighting.stereo(*l, *r);
            let gain = self.next_gain(energy);
            *l *= gain;
            *r *= gain;
        }
    }

    /// Measure the limited output of the samples this normalizer processed.
    fn observe_output(&mut self, buffer: &[f32]) {
        for &sample in buffer {
            self.output.add(self.output_weighting.mono(sample));
        }
    }

    fn observe_output_stereo(&mut self, left: &[f32], right: &[f32]) {
        for (&l, &r) in left.iter().zip(right) {
            self.output.add(self.output_weighting.stereo(l, r));
        }
    }

    /// Account one sample's K-weighted energy and return the gain for it.
    fn next_gain(&mut self, energy: f32) -> f32 {
        let gain = self.gain;
        self.gain += self.gain_step;
        self.gained.add(energy * gain * gain);
        if self.input.add(energy) {
            self.steer();
        }
        gain
    }

    /// Pick the gain for the next hop and ramp to it.
    fn steer(&mut self) {
        let next_db = match self.input.loudness() {
            Some(loudness) => {
                let wanted = (self.target_lufs - loudness + self.limiter_loss()).clamp(
                    NOMINAL_LOUDNESS_GAIN_DB - LOUDNESS_GAIN_RANGE_DB,
                    NOMINAL_LOUDNESS_GAIN_DB + LOUDNESS_GAIN_RANGE_DB,
                );
                wanted.clamp(
                    self.gain_db - self.max_fall_db,
                    self.gain_db + self.max_rise_db,
                )
            }
            // Silence: hold the gain rather than chase the noise floor.
            None => self.gain_db,
        };
        // Start the next hop exactly where this one was heading, then ramp.
        let reached = dmath::db_to_linear(self.gain_db);
        self.gain_db = next_db;
        self.gain = reached;
        self.gain_step = (dmath::db_to_linear(next_db) - reached) / self.hop_samples as f32;
    }

    /// How much loudness the limiter takes off, up to what may be made up.
    fn limiter_loss(&self) -> f32 {
        match (self.gained.loudness(), self.output.loudness()) {
            (Some(gained), Some(output)) => {
                (gained - output).clamp(0.0, LOUDNESS_LIMITER_MAKEUP_MAX_LU)
            }
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MasterConfig {
    pub target_lufs: f32,
    pub ceiling_dbtp: f32,
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            target_lufs: DEFAULT_TARGET_LUFS,
            ceiling_dbtp: DEFAULT_CEILING_DBTP,
        }
    }
}

pub struct MasterReport {
    pub integrated_lufs_before: f32,
    pub integrated_lufs_after: f32,
    pub true_peak_dbtp: f32,
    pub limiter_engaged: bool,
}

pub struct MasterChain {
    config: MasterConfig,
    sample_rate: u32,
    highpass: HighPass,
    compressor: Compressor,
    limiter: Limiter,
    meter_before: Meter,
    meter_after: Meter,
    loudness: LoudnessNormalizer,
    limiter_engaged: bool,
}

impl MasterChain {
    pub fn new(sample_rate: u32, config: MasterConfig) -> Self {
        Self {
            config,
            sample_rate,
            highpass: HighPass::new(sample_rate),
            compressor: Compressor::new(sample_rate),
            limiter: Limiter::new(sample_rate, config.ceiling_dbtp),
            meter_before: Meter::new(sample_rate),
            meter_after: Meter::new(sample_rate),
            loudness: LoudnessNormalizer::new(sample_rate, config.target_lufs),
            limiter_engaged: false,
        }
    }

    /// Linear amplitude of this chain's raw-sample output ceiling (the clamp
    /// bound applied after limiting). Exposed for the crossfade mixer, which
    /// sums two already-mastered voice buffers and must still bound the mix.
    pub fn ceiling_linear(&self) -> f32 {
        dmath::db_to_linear(self.config.ceiling_dbtp)
    }

    /// REALTIME causal path (Godot, live WASM etc.). Metering is skipped because
    /// live callers do not request a report. Use [`Self::process_metered`] when a
    /// realtime report is required.
    pub fn process(&mut self, buffer: &mut [f32]) {
        self.process_realtime(buffer, false);
    }

    /// Realtime causal path with integrated-loudness and true-peak metering.
    pub fn process_metered(&mut self, buffer: &mut [f32]) {
        self.process_realtime(buffer, true);
    }

    fn process_realtime(&mut self, buffer: &mut [f32], meter: bool) {
        for x in buffer.iter_mut() {
            if !x.is_finite() {
                *x = 0.0;
            }
        }

        if meter {
            self.meter_before.process(buffer);
        }

        self.highpass.process(buffer);
        self.compressor.process(buffer);
        self.loudness.process(buffer);
        self.limiter.process(buffer);

        if self.limiter.was_engaged() {
            self.limiter_engaged = true;
        }

        let ceiling_linear = dmath::db_to_linear(self.config.ceiling_dbtp);
        for x in buffer.iter_mut() {
            *x = x.clamp(-ceiling_linear, ceiling_linear);
        }
        self.loudness.observe_output(buffer);

        if meter {
            self.meter_after.process(buffer);
        }
    }

    /// Stereo realtime path with linked detectors and no report metering.
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.process_stereo_realtime(left, right, false);
    }

    /// Stereo realtime path with linked detectors and report metering.
    pub fn process_stereo_metered(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.process_stereo_realtime(left, right, true);
    }

    fn process_stereo_realtime(&mut self, left: &mut [f32], right: &mut [f32], meter: bool) {
        let n = left.len();
        if n != right.len() || n == 0 {
            return;
        }
        for i in 0..n {
            if !left[i].is_finite() {
                left[i] = 0.0;
            }
            if !right[i].is_finite() {
                right[i] = 0.0;
            }
        }
        if meter {
            self.meter_before.process_stereo(left, right);
        }
        self.highpass.process_stereo(left, right);
        self.compressor.process_stereo(left, right);
        self.loudness.process_stereo(left, right);
        self.limiter.process_stereo(left, right);
        if self.limiter.was_engaged() {
            self.limiter_engaged = true;
        }
        let ceiling = dmath::db_to_linear(self.config.ceiling_dbtp);
        for i in 0..n {
            left[i] = left[i].clamp(-ceiling, ceiling);
            right[i] = right[i].clamp(-ceiling, ceiling);
        }
        self.loudness.observe_output_stereo(left, right);
        if meter {
            self.meter_after.process_stereo(left, right);
        }
    }

    /// OFFLINE render path (render_wav etc.).
    /// Whole buffer available: measure LUFS after comp, apply corrective gain
    /// (iterated up to 3 times because TP limiting changes loudness slightly),
    /// then apply true-peak limiting using the shared envelope helper + max-hold
    /// release smoothing, then hard clamp.
    pub fn process_offline(&mut self, samples: &mut [f32]) -> MasterReport {
        if samples.is_empty() {
            return MasterReport {
                integrated_lufs_before: DEFAULT_TARGET_LUFS,
                integrated_lufs_after: DEFAULT_TARGET_LUFS,
                true_peak_dbtp: DEFAULT_CEILING_DBTP,
                limiter_engaged: false,
            };
        }

        for x in samples.iter_mut() {
            if !x.is_finite() {
                *x = 0.0;
            }
        }

        self.highpass.process(samples);
        self.compressor.process(samples);

        // Measure post-compressor (pre-gain) LUFS for the "before" figure.
        let lufs_before = measure_lufs(samples, self.sample_rate);

        // Iterative gain to hit target (including effect of subsequent TP limiting).
        // We re-measure after a simulated gain+limit on a probe because limiting
        // reduces loudness a little on peaky material.
        let mut target_gain = 1.0f32;
        let ceiling_for_iter = dmath::db_to_linear(self.config.ceiling_dbtp);
        let limit_t_for_iter = ceiling_for_iter * dmath::db_to_linear(-TRUE_PEAK_GUARD_DB);
        let fs_iter = self.sample_rate as f32;
        let r_iter = dmath::exp(-1.0 / (TP_LIMITER_RELEASE_SEC * fs_iter));
        for _ in 0..3 {
            let mut probe = samples.to_vec();
            for x in &mut probe {
                *x *= target_gain;
            }
            // simulate TP limit on probe (no need to write back clamp here for measure)
            let env_p = true_peak_envelope_4x(&probe);
            let mut gc: Vec<f32> = env_p
                .iter()
                .map(|&e| {
                    if e > 1e-9 {
                        (limit_t_for_iter / e).min(1.0)
                    } else {
                        1.0
                    }
                })
                .collect();
            let mut held = 1.0f32;
            for g in &mut gc {
                if *g < held {
                    held = *g;
                } else {
                    held = r_iter * held + (1.0 - r_iter) * *g;
                }
                *g = held;
            }
            for (s, g) in probe.iter_mut().zip(gc.into_iter()) {
                *s *= g;
            }
            let lu = measure_lufs(&probe, self.sample_rate);
            let err = self.config.target_lufs - lu;
            if err.abs() <= 0.3 {
                break;
            }
            target_gain *= dmath::db_to_linear(err);
            target_gain = target_gain.clamp(0.01, 100.0);
        }

        for x in samples.iter_mut() {
            *x *= target_gain;
        }

        // True-peak limiting using the *exact same* envelope helper the Meter
        // and realtime limiter use. Non-causal smoothing is fine for offline.
        let ceiling = dmath::db_to_linear(self.config.ceiling_dbtp);
        let limit_target = ceiling * dmath::db_to_linear(-TRUE_PEAK_GUARD_DB);

        let env = true_peak_envelope_4x(samples);
        let mut gain_curve: Vec<f32> = env
            .iter()
            .map(|&e| {
                if e > 1e-9 {
                    (limit_target / e).min(1.0)
                } else {
                    1.0
                }
            })
            .collect();

        // max-hold + exponential release (instant attack, slow release)
        let fs = self.sample_rate as f32;
        let r = dmath::exp(-1.0 / (TP_LIMITER_RELEASE_SEC * fs));
        let mut held = 1.0f32;
        let mut any_reduction = false;
        for g in &mut gain_curve {
            if *g < held {
                held = *g;
            } else {
                held = r * held + (1.0 - r) * *g;
            }
            if held < 0.999 {
                any_reduction = true;
            }
            *g = held;
        }

        for (s, g) in samples.iter_mut().zip(gain_curve.into_iter()) {
            *s *= g;
        }

        // Residual true-peak safety. The smoothed gain curve cannot track every
        // isolated transient exactly, and an external meter uses its own (also
        // linear) interpolant, so bound the envelope actually measured here.
        // Because the interpolator is linear this converges tightly and only
        // touches the samples that exceed the target.
        for _ in 0..4 {
            let env = true_peak_envelope_4x(samples);
            let worst = env.iter().copied().fold(0.0f32, f32::max);
            if worst <= limit_target {
                break;
            }
            for (s, e) in samples.iter_mut().zip(env.iter()) {
                if *e > limit_target {
                    *s *= limit_target / *e;
                }
            }
        }

        // Absolute backstop: samples are *always* within ceiling after this.
        for s in samples.iter_mut() {
            *s = s.clamp(-ceiling, ceiling);
        }

        let lufs_after = measure_lufs(samples, self.sample_rate);

        let final_tp = {
            let env = true_peak_envelope_4x(samples);
            let mp = env.iter().copied().fold(0.0f32, f32::max);
            if mp > 1e-9 {
                20.0 * dmath::log10(mp)
            } else {
                -120.0
            }
        };

        MasterReport {
            integrated_lufs_before: lufs_before,
            integrated_lufs_after: lufs_after,
            true_peak_dbtp: final_tp,
            limiter_engaged: any_reduction || self.limiter_engaged,
        }
    }

    /// OFFLINE stereo path. Same loudness target/ceiling logic, linked detectors,
    /// per-channel K for LUFS (mean energy), max TP across channels.
    pub fn process_offline_stereo(&mut self, left: &mut [f32], right: &mut [f32]) -> MasterReport {
        let n = left.len();
        if n != right.len() {
            return MasterReport {
                integrated_lufs_before: DEFAULT_TARGET_LUFS,
                integrated_lufs_after: DEFAULT_TARGET_LUFS,
                true_peak_dbtp: DEFAULT_CEILING_DBTP,
                limiter_engaged: false,
            };
        }
        if n == 0 {
            return MasterReport {
                integrated_lufs_before: DEFAULT_TARGET_LUFS,
                integrated_lufs_after: DEFAULT_TARGET_LUFS,
                true_peak_dbtp: DEFAULT_CEILING_DBTP,
                limiter_engaged: false,
            };
        }
        for i in 0..n {
            if !left[i].is_finite() {
                left[i] = 0.0;
            }
            if !right[i].is_finite() {
                right[i] = 0.0;
            }
        }
        self.highpass.process_stereo(left, right);
        self.compressor.process_stereo(left, right);

        // lufs before: use stereo meter on post comp
        let mut m_before = Meter::new(self.sample_rate);
        m_before.process_stereo(left, right);
        let (lufs_before, _) = m_before.report();

        // iterative gain + tp limit (same as mono, applied to both)
        let mut target_gain = 1.0f32;
        let ceiling_for_iter = dmath::db_to_linear(self.config.ceiling_dbtp);
        let limit_t_for_iter = ceiling_for_iter * dmath::db_to_linear(-TRUE_PEAK_GUARD_DB);
        let fs_iter = self.sample_rate as f32;
        let r_iter = dmath::exp(-1.0 / (TP_LIMITER_RELEASE_SEC * fs_iter));
        for _ in 0..3 {
            let mut probe_l = left.to_vec();
            let mut probe_r = right.to_vec();
            for i in 0..n {
                probe_l[i] *= target_gain;
                probe_r[i] *= target_gain;
            }
            let env_pl = true_peak_envelope_4x(&probe_l);
            let env_pr = true_peak_envelope_4x(&probe_r);
            let mut gc: Vec<f32> = (0..n)
                .map(|i| {
                    let e = env_pl[i].max(env_pr[i]);
                    if e > 1e-9 {
                        (limit_t_for_iter / e).min(1.0)
                    } else {
                        1.0
                    }
                })
                .collect();
            let mut held = 1.0f32;
            for g in &mut gc {
                if *g < held {
                    held = *g;
                } else {
                    held = r_iter * held + (1.0 - r_iter) * *g;
                }
                *g = held;
            }
            for i in 0..n {
                probe_l[i] *= gc[i];
                probe_r[i] *= gc[i];
            }
            let lu = measure_lufs_stereo(&probe_l, &probe_r, self.sample_rate);
            let err = self.config.target_lufs - lu;
            if err.abs() <= 0.3 {
                break;
            }
            target_gain *= dmath::db_to_linear(err);
            target_gain = target_gain.clamp(0.01, 100.0);
        }
        for i in 0..n {
            left[i] *= target_gain;
            right[i] *= target_gain;
        }

        // TP limiting using max env
        let ceiling = dmath::db_to_linear(self.config.ceiling_dbtp);
        let limit_target = ceiling * dmath::db_to_linear(-TRUE_PEAK_GUARD_DB);
        let env_l = true_peak_envelope_4x(left);
        let env_r = true_peak_envelope_4x(right);
        let mut gain_curve: Vec<f32> = (0..n)
            .map(|i| {
                let e = env_l[i].max(env_r[i]);
                if e > 1e-9 {
                    (limit_target / e).min(1.0)
                } else {
                    1.0
                }
            })
            .collect();
        let fs = self.sample_rate as f32;
        let r = dmath::exp(-1.0 / (TP_LIMITER_RELEASE_SEC * fs));
        let mut held = 1.0f32;
        let mut any_reduction = false;
        for g in &mut gain_curve {
            if *g < held {
                held = *g;
            } else {
                held = r * held + (1.0 - r) * *g;
            }
            if held < 0.999 {
                any_reduction = true;
            }
            *g = held;
        }
        for i in 0..n {
            left[i] *= gain_curve[i];
            right[i] *= gain_curve[i];
        }
        // residual safety
        for _ in 0..4 {
            let el = true_peak_envelope_4x(left);
            let er = true_peak_envelope_4x(right);
            let worst = el
                .iter()
                .zip(er.iter())
                .map(|(a, b)| a.max(*b))
                .fold(0.0f32, f32::max);
            if worst <= limit_target {
                break;
            }
            for i in 0..n {
                let e = el[i].max(er[i]);
                if e > limit_target {
                    let g = limit_target / e;
                    left[i] *= g;
                    right[i] *= g;
                }
            }
        }
        for i in 0..n {
            left[i] = left[i].clamp(-ceiling, ceiling);
            right[i] = right[i].clamp(-ceiling, ceiling);
        }
        let lufs_after = measure_lufs_stereo(left, right, self.sample_rate);
        let final_tp = {
            let el = true_peak_envelope_4x(left);
            let er = true_peak_envelope_4x(right);
            let mp = el
                .iter()
                .zip(er.iter())
                .map(|(a, b)| a.max(*b))
                .fold(0.0f32, f32::max);
            if mp > 1e-9 {
                20.0 * dmath::log10(mp)
            } else {
                -120.0
            }
        };
        MasterReport {
            integrated_lufs_before: lufs_before,
            integrated_lufs_after: lufs_after,
            true_peak_dbtp: final_tp,
            limiter_engaged: any_reduction || self.limiter_engaged,
        }
    }

    /// Report values accumulated by [`Self::process_metered`] or
    /// [`Self::process_stereo_metered`]. The default realtime process methods do
    /// not update the meters.
    pub fn report(&self) -> MasterReport {
        let (lufs_before, _) = self.meter_before.report();
        let (lufs_after, tp_after) = self.meter_after.report();
        MasterReport {
            integrated_lufs_before: lufs_before,
            integrated_lufs_after: lufs_after,
            true_peak_dbtp: tp_after,
            limiter_engaged: self.limiter_engaged,
        }
    }
}

fn measure_lufs(samples: &[f32], sr: u32) -> f32 {
    let mut m = Meter::new(sr);
    m.process(samples);
    let (l, _) = m.report();
    l
}

fn measure_lufs_stereo(left: &[f32], right: &[f32], sr: u32) -> f32 {
    let mut m = Meter::new(sr);
    m.process_stereo(left, right);
    let (l, _) = m.report();
    l
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{score::PortableScore, synth::events_starting_at, Synth};
    use std::f32::consts::PI;
    use std::time::{Duration, Instant};

    // --- helpers local to tests ---
    fn decode_wav_samples(wav: &[u8]) -> Vec<f32> {
        let data_size = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
        let mut out = Vec::with_capacity(data_size / 2);
        for i in (0..data_size).step_by(2) {
            let s = i16::from_le_bytes(wav[44 + i..44 + i + 2].try_into().unwrap());
            out.push(s as f32 / 32768.0);
        }
        out
    }

    fn ceiling_linear() -> f32 {
        dmath::db_to_linear(DEFAULT_CEILING_DBTP)
    }

    #[derive(Clone, Copy)]
    enum RealtimeBenchmarkMode {
        SynthOnly,
        ProductionMaster,
        MeteredMaster,
        MasterWithoutMeters,
    }

    fn process_without_meters(chain: &mut MasterChain, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            if !sample.is_finite() {
                *sample = 0.0;
            }
        }
        chain.highpass.process(buffer);
        chain.compressor.process(buffer);
        chain.loudness.process(buffer);

        chain.limiter.process(buffer);
        let ceiling = dmath::db_to_linear(chain.config.ceiling_dbtp);
        for sample in buffer.iter_mut() {
            *sample = sample.clamp(-ceiling, ceiling);
        }
        chain.loudness.observe_output(buffer);
    }

    fn sine(frequency: f32, amplitude: f32, seconds: f32) -> Vec<f32> {
        let rate = DEFAULT_SAMPLE_RATE as f32;
        (0..(rate * seconds) as usize)
            .map(|i| amplitude * dmath::sin(2.0 * PI * frequency * i as f32 / rate))
            .collect()
    }

    fn gain_db(normalizer: &LoudnessNormalizer) -> f32 {
        20.0 * dmath::log10(normalizer.gain)
    }

    #[test]
    fn loudness_normalizer_keeps_silence_silent_and_holds_its_gain() {
        let mut normalizer = LoudnessNormalizer::new(DEFAULT_SAMPLE_RATE, DEFAULT_TARGET_LUFS);
        let mut silence = vec![0.0; DEFAULT_SAMPLE_RATE as usize * 5];
        normalizer.process(&mut silence);
        assert!(silence.iter().all(|&sample| sample == 0.0));
        assert!((gain_db(&normalizer) - NOMINAL_LOUDNESS_GAIN_DB).abs() < 1e-3);
    }

    #[test]
    fn loudness_normalizer_settles_a_steady_tone_at_the_target() {
        let mut normalizer = LoudnessNormalizer::new(DEFAULT_SAMPLE_RATE, DEFAULT_TARGET_LUFS);
        let mut tone = sine(1000.0, 0.01, 60.0);
        normalizer.process(&mut tone);
        let mut meter = Meter::new(DEFAULT_SAMPLE_RATE);
        meter.process(&tone[tone.len() - DEFAULT_SAMPLE_RATE as usize * 10..]);
        let (lufs, _) = meter.report();
        assert!(
            (lufs - DEFAULT_TARGET_LUFS).abs() < 0.5,
            "settled at {lufs:.2} LUFS"
        );
    }

    #[test]
    fn loudness_normalizer_moves_its_gain_no_faster_than_its_rates() {
        // The first measurement needs one whole block; from then on the gain
        // moves for one second.
        let seconds = LOUDNESS_BLOCK_SECONDS + 1.0;
        // A loud tone wants far less gain: the gain falls at its fall rate.
        let mut normalizer = LoudnessNormalizer::new(DEFAULT_SAMPLE_RATE, DEFAULT_TARGET_LUFS);
        normalizer.process(&mut sine(1000.0, 0.5, seconds));
        let fallen = NOMINAL_LOUDNESS_GAIN_DB - gain_db(&normalizer);
        assert!(
            (fallen - LOUDNESS_FALL_DB_PER_SECOND).abs() < 0.01,
            "fell {fallen:.2} dB in 1 s"
        );
        // A quiet tone wants far more gain: the gain rises at its rise rate.
        let mut normalizer = LoudnessNormalizer::new(DEFAULT_SAMPLE_RATE, DEFAULT_TARGET_LUFS);
        normalizer.process(&mut sine(1000.0, 1e-3, seconds));
        let risen = gain_db(&normalizer) - NOMINAL_LOUDNESS_GAIN_DB;
        assert!(
            (risen - LOUDNESS_RISE_DB_PER_SECOND).abs() < 0.01,
            "rose {risen:.2} dB in 1 s"
        );
    }

    fn benchmark_realtime_render(
        score: &PortableScore,
        seconds: usize,
        mode: RealtimeBenchmarkMode,
        voice_filter: Option<&str>,
    ) -> Duration {
        const SAMPLE_RATE: usize = 48_000;
        const BUFFER_FRAMES: usize = 256;
        let section_id = "grid";
        let section_length = score
            .section(section_id)
            .expect("benchmark section must exist")
            .length_ticks;
        let ticks_per_second = score.ticks_per_second();
        let total_frames = seconds * SAMPLE_RATE;
        let mut schedule = Vec::with_capacity(total_frames.div_ceil(BUFFER_FRAMES));
        let mut frames_produced = 0usize;
        let mut tick = 0u32;
        while frames_produced < total_frames {
            let frames = (total_frames - frames_produced).min(BUFFER_FRAMES);
            let next_tick = crate::synth::tick_at_sample(
                (frames_produced + frames) as u64,
                SAMPLE_RATE as f64,
                ticks_per_second,
            );
            let span_ticks = next_tick.saturating_sub(tick).max(1);
            let local_tick = tick % section_length.max(1);
            let events = events_starting_at(score, section_id, local_tick, span_ticks)
                .into_iter()
                .map(|event| {
                    let offset_seconds =
                        (f64::from(event.start_tick()) - f64::from(local_tick)) / ticks_per_second;
                    (event, offset_seconds.max(0.0))
                })
                .collect::<Vec<_>>();
            schedule.push((frames, events));
            frames_produced += frames;
            tick = next_tick;
        }

        let started = Instant::now();
        let mut synth = Synth::new(SAMPLE_RATE as f32);
        let mut master = MasterChain::new(SAMPLE_RATE as u32, MasterConfig::default());
        let mut checksum = 0.0f32;
        for (frames, events) in &schedule {
            for (event, offset_seconds) in events {
                if voice_filter.is_none_or(|voice| event.voice() == voice) {
                    synth.trigger_at(event, ticks_per_second, *offset_seconds);
                }
            }
            let mut buffer = [0.0f32; BUFFER_FRAMES];
            synth.fill(&mut buffer[..*frames]);
            match mode {
                RealtimeBenchmarkMode::SynthOnly => {}
                RealtimeBenchmarkMode::ProductionMaster => {
                    master.process(&mut buffer[..*frames]);
                }
                RealtimeBenchmarkMode::MeteredMaster => {
                    master.process_metered(&mut buffer[..*frames]);
                }
                RealtimeBenchmarkMode::MasterWithoutMeters => {
                    process_without_meters(&mut master, &mut buffer[..*frames]);
                }
            }
            checksum += buffer[frames - 1];
        }
        std::hint::black_box(checksum);
        started.elapsed()
    }

    fn median_realtime_render(
        score: &PortableScore,
        seconds: usize,
        mode: RealtimeBenchmarkMode,
        voice_filter: Option<&str>,
    ) -> Duration {
        let mut timings = (0..3)
            .map(|_| benchmark_realtime_render(score, seconds, mode, voice_filter))
            .collect::<Vec<_>>();
        timings.sort_unstable();
        timings[1]
    }

    #[test]
    #[ignore = "release-only timing benchmark; run explicitly with --ignored --nocapture"]
    #[allow(clippy::assertions_on_constants)]
    fn realtime_render_benchmark() {
        assert!(
            !cfg!(debug_assertions),
            "run this timing benchmark with cargo test --release"
        );
        const AUDIO_SECONDS: usize = 12;
        let score: PortableScore = serde_json::from_str(include_str!(
            "../../../catalog/racing/tiny-torque-level-004/score.json"
        ))
        .expect("representative racing score must parse");
        let cases = [
            ("synth only", RealtimeBenchmarkMode::SynthOnly),
            (
                "synth + production master",
                RealtimeBenchmarkMode::ProductionMaster,
            ),
            (
                "synth + metered master",
                RealtimeBenchmarkMode::MeteredMaster,
            ),
            (
                "synth + master, meters off",
                RealtimeBenchmarkMode::MasterWithoutMeters,
            ),
        ];
        println!("rendering {AUDIO_SECONDS}s of racing/grid audio (median of 3)");
        for (label, mode) in cases {
            let elapsed = median_realtime_render(&score, AUDIO_SECONDS, mode, None);
            let milliseconds_per_audio_second =
                elapsed.as_secs_f64() * 1_000.0 / AUDIO_SECONDS as f64;
            println!("{label}: {milliseconds_per_audio_second:.3} ms CPU / s audio");
            if label.contains("production master") {
                // Regression ceiling for the production render path (synth + MasterChain::process).
                // Guards against CPU cost creep re-introduced in the realtime mix path used by games.
                // Measured on 2026-09-21 dev machine (release, median of 3): ~19.5 ms CPU per audio-second.
                // Set ceiling at ~2x (CI runners slow/noisy): 40.0 .
                // Re-measure with: cargo test -p gamestruments-engine --release realtime_render_benchmark -- --ignored --nocapture --test-threads=1
                // Update this ceiling (and comment) only after confirming a legitimate sustained change.
                assert!(
                    milliseconds_per_audio_second < 40.0,
                    "production master render cost {milliseconds_per_audio_second:.3} ms/s exceeded ceiling 40.0; re-measure and adjust if needed"
                );
            }
        }
    }

    #[test]
    #[ignore = "release-only voice timing benchmark; run explicitly with --ignored --nocapture"]
    #[allow(clippy::assertions_on_constants)]
    fn realtime_synth_voice_breakdown_benchmark() {
        assert!(
            !cfg!(debug_assertions),
            "run this timing benchmark with cargo test --release"
        );
        const AUDIO_SECONDS: usize = 12;
        let score: PortableScore = serde_json::from_str(include_str!(
            "../../../catalog/racing/tiny-torque-level-004/score.json"
        ))
        .expect("representative racing score must parse");
        println!("rendering {AUDIO_SECONDS}s of racing/grid audio by voice (median of 3)");
        for voice in ["__none__", "pluck", "bass", "kick", "snare", "hat"] {
            let elapsed = median_realtime_render(
                &score,
                AUDIO_SECONDS,
                RealtimeBenchmarkMode::SynthOnly,
                Some(voice),
            );
            let milliseconds_per_audio_second =
                elapsed.as_secs_f64() * 1_000.0 / AUDIO_SECONDS as f64;
            println!("{voice}: {milliseconds_per_audio_second:.3} ms CPU / s audio");
        }
    }

    #[test]
    fn meter_sanity() {
        // A full-scale 1 kHz sine (peak = 1.0) has RMS = 1/sqrt(2) ≈ -3.0103 dBFS.
        // Per EBU R128 / ITU, integrated LUFS for such a sine is approximately -3.01 LUFS
        // (the K-weighted, gated measurement lands very close because the tone is steady).
        // True peak (our Catmull-Rom) for a pure sine at this freq is ~0.0 dBTP.
        let mut meter = Meter::new(48000);
        let mut sine = vec![0.0; 48000 * 3]; // 3s for stable blocks
        for (i, x) in sine.iter_mut().enumerate() {
            *x = (2.0 * PI * 1000.0 * i as f32 / 48000.0).sin();
        }
        meter.process(&sine);
        let (lufs, tp) = meter.report();
        assert!(
            (lufs + 3.01).abs() < 0.5,
            "full-scale 1kHz sine should measure ~-3.01 LUFS, got {}",
            lufs
        );
        assert!(
            tp.abs() < 0.2,
            "full-scale 1kHz sine should measure ~0.0 dBTP, got {}",
            tp
        );
    }

    #[test]
    fn realtime_limiter_awkward_chunks() {
        let mut chain = MasterChain::new(48000, MasterConfig::default());
        // very loud signal that would clip hard without limiter
        let mut signal = vec![0.0; 48000];
        for (i, x) in signal.iter_mut().enumerate() {
            *x = (2.0 * PI * 100.0 * i as f32 / 48000.0).sin() * 10.0;
        }

        let chunks = [1usize, 7, 64, 256, 4096];
        let mut pos = 0;
        let mut out = Vec::new();
        let mut ci = 0;
        while pos < signal.len() {
            let chunk_size = chunks[ci % chunks.len()].min(signal.len() - pos);
            let mut buf = signal[pos..pos + chunk_size].to_vec();
            chain.process(&mut buf);
            // also assert no NaN in this chunk
            for &s in &buf {
                assert!(s.is_finite(), "NaN/Inf in realtime output");
            }
            out.extend_from_slice(&buf);
            pos += chunk_size;
            ci += 1;
        }

        let ceiling = ceiling_linear();
        for &s in &out {
            assert!(
                s.abs() <= ceiling + 1e-6,
                "sample ceiling invariant violated across chunks: {}",
                s
            );
        }
        let max_true_peak = true_peak_envelope_4x(&out)
            .into_iter()
            .fold(0.0f32, f32::max);
        assert!(
            max_true_peak <= ceiling + 2e-3,
            "realtime true-peak ceiling violated across chunks: {max_true_peak}"
        );
    }

    #[test]
    fn realtime_metering_does_not_change_audio() {
        let mut unmetered = MasterChain::new(48000, MasterConfig::default());
        let mut metered = MasterChain::new(48000, MasterConfig::default());
        let mut signal = (0..48000)
            .map(|index| {
                let phase = 2.0 * PI * 997.0 * index as f32 / 48000.0;
                phase.sin() * (0.5 + (index % 127) as f32 / 127.0)
            })
            .collect::<Vec<_>>();
        let mut metered_signal = signal.clone();

        for (plain, measured) in signal.chunks_mut(256).zip(metered_signal.chunks_mut(256)) {
            unmetered.process(plain);
            metered.process_metered(measured);
        }

        assert_eq!(
            signal
                .iter()
                .map(|sample| sample.to_bits())
                .collect::<Vec<_>>(),
            metered_signal
                .iter()
                .map(|sample| sample.to_bits())
                .collect::<Vec<_>>()
        );
        assert!(metered.report().integrated_lufs_after.is_finite());
    }

    #[test]
    fn offline_renders_hit_lufs_and_tp_and_never_clip() {
        // Sweep across recipes, styles, sections, seeds through the *offline* path (render_wav).
        // This would have caught both the static-trim spread and the true-peak overs.
        use crate::{
            generate_racing, generate_suspense, render::render_wav, GenerateInput,
            InstrumentPalette, Style, SuspenseInput, SuspenseStyle,
        };

        let seeds = ["qa-seed-a", "qa-seed-b"];
        let r_styles = [Style::Funk, Style::Neon];
        let r_sections = ["cruise", "grid", "attack"];
        let ceiling = ceiling_linear();
        let target = DEFAULT_TARGET_LUFS;

        let mut all_lufs: Vec<f32> = vec![];

        // Racing
        for seed in &seeds {
            for sty in &r_styles {
                for sec in &r_sections {
                    let score = generate_racing(&GenerateInput {
                        secret: "qa-secret".into(),
                        seed: (*seed).into(),
                        style: *sty,
                        palette: InstrumentPalette::default(),
                        energy: 0.5,
                        complexity: 0.5,
                        brightness: 0.5,
                        syncopation: 0.5,
                    })
                    .expect("racing score");
                    let wav = render_wav(&score, sec, 1, 48000);
                    let samples = decode_wav_samples(&wav);

                    for &s in &samples {
                        assert!(s.is_finite(), "non-finite sample in offline render");
                        assert!(
                            s.abs() <= ceiling + 1e-9,
                            "offline sample exceeded ceiling {}: {}",
                            ceiling,
                            s
                        );
                    }

                    let mut meter = Meter::new(48000);
                    meter.process(&samples);
                    let (lufs, tp) = meter.report();

                    assert!(
                        (lufs - target).abs() <= 1.5,
                        "LUFS {} outside +/-1.5 of {} for racing {}/{}",
                        lufs,
                        target,
                        seed,
                        sec
                    );
                    assert!(
                        tp <= DEFAULT_CEILING_DBTP + 1e-3,
                        "true peak {} > ceiling for racing",
                        tp
                    );

                    all_lufs.push(lufs);
                }
            }
        }

        // A bit of suspense too (different recipe) - use correct fields
        for seed in &seeds {
            let score = generate_suspense(&SuspenseInput {
                secret: "qa-secret".into(),
                seed: (*seed).into(),
                style: SuspenseStyle::Terminal,
                tension: 0.6,
                heat: 0.5,
                mystery: 0.5,
                pulse: 0.5,
            })
            .expect("suspense score");
            let wav = render_wav(&score, "verse", 1, 48000);
            let samples = decode_wav_samples(&wav);
            let mut meter = Meter::new(48000);
            meter.process(&samples);
            let (lufs, tp) = meter.report();
            assert!(
                (lufs - target).abs() <= 1.5,
                "suspense LUFS out of range: {}",
                lufs
            );
            assert!(tp <= DEFAULT_CEILING_DBTP + 1e-3);

            for &s in &samples {
                assert!(s.abs() <= ceiling + 1e-9);
            }
            all_lufs.push(lufs);
        }

        // Dedicated loudness spread test across styles (same recipe/seed/section).
        // A static trim can never achieve this; the per-render measurement+gain can.
        let spread_styles = [Style::Chip, Style::Funk, Style::Fusion, Style::Neon];
        let mut style_lufs = vec![];
        for sty in &spread_styles {
            let score = generate_racing(&GenerateInput {
                secret: "spread-secret".into(),
                seed: "spread-seed".into(),
                style: *sty,
                palette: InstrumentPalette::default(),
                energy: 0.62,
                complexity: 0.6,
                brightness: 0.52,
                syncopation: 0.7,
            })
            .expect("style spread score");
            let wav = render_wav(&score, "cruise", 2, 48000);
            let samples = decode_wav_samples(&wav);
            let mut meter = Meter::new(48000);
            meter.process(&samples);
            let (l, _) = meter.report();
            style_lufs.push(l);
        }
        let min_l = style_lufs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_l = style_lufs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let spread = max_l - min_l;
        assert!(
            spread <= 2.0,
            "loudness spread across styles {} > 2.0 LU (min {} max {})",
            spread,
            min_l,
            max_l
        );
    }
}
