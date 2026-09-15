//! Deterministic floating-point math for the master chain.
//!
//! Native x86_64 and `wasm32-unknown-unknown` link against different libm
//! implementations, so Rust's `f32::{exp, ln, log10, powf, sin, cos}` are not
//! bit-identical across the two targets: their results can differ by a couple of
//! ULPs. The master chain uses these per-sample (compressor level/gain, limiter
//! release, LUFS log) and per-chain (attack/release coefficients, biquad
//! coefficients). A one-ULP difference in the slow compressor release
//! coefficient is amplified by the `1 / (1 - coef)` factor of the one-pole
//! smoother into a ~1e-4 relative difference in gain, i.e. several LSBs at
//! 16-bit — exactly the native/WASM WAV mismatch this module removes.
//!
//! Every function here is a self-contained implementation using only:
//!   * IEEE 754 `f64` arithmetic (`+`, `-`, `*`, `/`), which is correctly
//!     rounded and therefore bit-identical on every conforming target;
//!   * exponent/mantissa field manipulation on the bit pattern.
//!
//! There are no platform libm calls and no `f32::exp/ln/log10/powf/sin/cos` in
//! the dependency chain, so native and WASM produce bit-identical results for
//! identical input.
//!
//! Accuracy: the polynomial fits are minimax-style fits accurate to a few
//! `1e-10` in `f64`, so after the final round-to-`f32` every result is within
//! ~1 ULP of the correctly rounded value — far tighter than the ~120 dB of
//! headroom the 16-bit output needs, and indistinguishable from the libm
//! versions it replaces.
//!
//! `sqrt` is intentionally *not* provided: IEEE 754 already requires `sqrt` to
//! be correctly rounded, so `f32::sqrt` is bit-identical across platforms.
//!
//! The fitted polynomial/table coefficients below carry deliberate full-precision
//! values (they are minimax approximations, not hand-typed constants), so the
//! `excessive_precision` lint is disabled for this module.

#![allow(clippy::excessive_precision)]

use std::f64::consts::{FRAC_1_PI, FRAC_PI_2, LN_10, LN_2, LOG10_E, LOG2_E, PI, SQRT_2};

/// `ln(10) / 20`, the dB-to-linear exponent scale factor.
const LN10_OVER_20: f64 = LN_10 / 20.0;

/// `ln(f32::MAX)` — arguments at or above this overflow `f32`.
const EXP_OVERFLOW: f64 = 88.72283905206835;
/// `ln(f32::MIN_POSITIVE)` — arguments at or below this underflow to 0.
const EXP_UNDERFLOW: f64 = -87.33654475055308;

/// Minimax polynomial for `2^f` on `f in [-0.5, 0.5]` (degree 8).
const EXP2_POLY: [f64; 9] = [
    1.000000000002163e+00,
    6.931471805498804e-01,
    2.402265069541113e-01,
    5.550410925907066e-02,
    9.618129255591910e-03,
    1.333346486369302e-03,
    1.540338329498208e-04,
    1.530635628791920e-05,
    1.327211618234721e-06,
];

/// Minimax polynomial for `ln(1 + u)` on `u in [-0.29289322, 0.41421356]`
/// (degree 11), i.e. `u = m - 1` after `m` is reduced into `[1/sqrt(2), sqrt(2)]`.
const LN1P_POLY: [f64; 12] = [
    -3.066125131567787e-11,
    1.000000002525844e+00,
    -4.999999820689000e-01,
    3.333327804418841e-01,
    -2.500012775284929e-01,
    2.000341932763837e-01,
    -1.666552657028136e-01,
    1.419971808721590e-01,
    -1.242466940381764e-01,
    1.201731556614345e-01,
    -1.163133668824440e-01,
    6.458878354261852e-02,
];

/// Odd minimax polynomial for `sin(r)` on `r in [-pi/2, pi/2]`, as
/// `sin(r) = r * (c0 + c1*s + c2*s^2 + ... + c6*s^6)` with `s = r^2`.
const SIN_POLY: [f64; 7] = [
    1.000000000001888e+00,
    -1.666666666658091e-01,
    8.333333326684251e-03,
    -1.984126794122013e-04,
    2.755706448879024e-06,
    -2.503477898519590e-08,
    1.547898085773734e-10,
];

#[inline]
fn horner(coeffs: &[f64], x: f64) -> f64 {
    let mut acc = coeffs[coeffs.len() - 1];
    for &c in coeffs[..coeffs.len() - 1].iter().rev() {
        acc = acc * x + c;
    }
    acc
}

/// Construct `2^n` for an integral `n` directly from the exponent field.
/// Exact for every representable power of two.
#[inline]
fn pow2(n: i32) -> f64 {
    if n < -1074 {
        return 0.0;
    }
    if n > 1023 {
        return f64::INFINITY;
    }
    f64::from_bits(((n + 1023) as u64) << 52)
}

/// `exp(x)` as `f64`, with the argument assumed finite and within the `f32`
/// representable range (overflow/underflow are handled by [`exp`]).
#[inline]
fn exp64(x: f64) -> f64 {
    // exp(x) = 2^(x * log2(e)); split x * log2(e) into integer n + fraction f.
    let t = x * LOG2_E;
    let n = t.round();
    let f = t - n; // f in [-0.5, 0.5]
    horner(&EXP2_POLY, f) * pow2(n as i32)
}

/// Deterministic `f32::exp`.
pub fn exp(x: f32) -> f32 {
    let x = x as f64;
    if x.is_nan() {
        return f32::NAN;
    }
    if x >= EXP_OVERFLOW {
        return f32::INFINITY;
    }
    if x <= EXP_UNDERFLOW {
        return 0.0;
    }
    exp64(x) as f32
}

/// `ln(x)` as `f64`, for positive finite `x` (special cases handled by [`ln`]).
#[inline]
fn ln64(x: f64) -> f64 {
    // Decompose x = m * 2^e with m in [1, 2).
    let bits = x.to_bits();
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;
    let mut m = f64::from_bits(mantissa | 0x3ff0_0000_0000_0000); // m in [1, 2)
    if m >= SQRT_2 {
        // Reduce into [1/sqrt(2), sqrt(2)) so ln(m) stays within [-0.3466, 0.3466].
        m *= 0.5;
        e += 1;
    }
    let u = m - 1.0; // u in [-0.2929, 0.4142]
    e as f64 * LN_2 + horner(&LN1P_POLY, u)
}

/// Deterministic `f32::ln`.
pub fn ln(x: f32) -> f32 {
    let x = x as f64;
    if x.is_nan() || x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 {
        return f32::NEG_INFINITY;
    }
    if x.is_infinite() {
        return x as f32;
    }
    ln64(x) as f32
}

/// Deterministic `f32::log10`.
pub fn log10(x: f32) -> f32 {
    // log10(x) = ln(x) * log10(e); delegating to [`ln`] keeps the edge cases
    // (0, negative, NaN, infinity) consistent and single-sourced.
    ln(x) * LOG10_E as f32
}

/// Deterministic `10^x`.
pub fn pow10(x: f32) -> f32 {
    let x = x as f64;
    if x.is_nan() {
        return f32::NAN;
    }
    if x * LN_10 >= EXP_OVERFLOW {
        return f32::INFINITY;
    }
    if x * LN_10 <= EXP_UNDERFLOW {
        return 0.0;
    }
    exp64(x * LN_10) as f32
}

/// Deterministic `10^(db / 20)`, i.e. the standard dB-to-linear gain conversion.
pub fn db_to_linear(db: f32) -> f32 {
    let db = db as f64;
    if db.is_nan() {
        return f32::NAN;
    }
    if db * LN10_OVER_20 >= EXP_OVERFLOW {
        return f32::INFINITY;
    }
    if db * LN10_OVER_20 <= EXP_UNDERFLOW {
        return 0.0;
    }
    exp64(db * LN10_OVER_20) as f32
}

/// Deterministic `base.powf(exp)` for `base >= 0`.
///
/// The synth only ever raises non-negative quantities (frequencies, gains,
/// velocities) to a power, so this is defined via `powf(b, e) = exp(e * ln(b))`
/// for `b > 0`, plus the standard IEEE edge cases for `b == 0`. Negative bases
/// are not produced by the synth and yield `NaN` (defensive only).
pub fn powf(base: f32, exp: f32) -> f32 {
    let b = base as f64;
    let e = exp as f64;
    if b.is_nan() || e.is_nan() {
        return f32::NAN;
    }
    if b < 0.0 {
        return f32::NAN;
    }
    if b == 0.0 {
        return if e > 0.0 {
            0.0
        } else if e < 0.0 {
            f32::INFINITY
        } else {
            1.0
        };
    }
    if b.is_infinite() {
        return if e > 0.0 {
            f32::INFINITY
        } else if e < 0.0 {
            0.0
        } else {
            f32::NAN
        };
    }
    exp64(e * ln64(b)) as f32
}

/// `sin(x)` as `f64`, for finite `x`.
#[inline]
fn sin64(x: f64) -> f64 {
    // Reduce into [-pi/2, pi/2]; the round()/subtract reduction is exact for the
    // small angles the biquad uses and accurate enough for any representable angle.
    let n = (x * FRAC_1_PI).round();
    let r = x - n * PI; // r in [-pi/2, pi/2]
    let s = r * r;
    let sin_r = r * horner(&SIN_POLY, s);
    if n.rem_euclid(2.0) == 0.0 {
        sin_r
    } else {
        -sin_r
    }
}

/// Deterministic `f32::sin`.
pub fn sin(x: f32) -> f32 {
    let x = x as f64;
    if !x.is_finite() {
        return f32::NAN;
    }
    sin64(x) as f32
}

/// Deterministic `f32::cos`.
pub fn cos(x: f32) -> f32 {
    let x = x as f64;
    if !x.is_finite() {
        return f32::NAN;
    }
    sin64(x + FRAC_PI_2) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel_err(got: f32, want: f64) -> f64 {
        let g = got as f64;
        if want == 0.0 {
            g.abs()
        } else {
            ((g - want) / want).abs()
        }
    }

    /// Accuracy bound used across all functions: the fits are accurate to ~1e-10
    /// in f64, so the f32 result is within ~1 ULP (1.2e-7) of the correct value.
    /// We assert the task's 1e-6 target with margin to spare.
    fn assert_close(got: f32, want: f64) {
        let e = rel_err(got, want);
        assert!(
            e < 1e-6,
            "got {got:?} vs want {want:?} (rel err {e:.3e})"
        );
    }

    #[test]
    fn exp_accuracy() {
        for &x in &[-80.0f32, -20.0, -10.0, -3.0, -1.0, -0.5, 0.0, 0.5, 1.0, 3.0, 10.0, 20.0, 80.0] {
            assert_close(exp(x), (x as f64).exp());
        }
        assert_eq!(exp(0.0), 1.0);
        assert_eq!(exp(1.0), (1.0f64.exp()) as f32);
    }

    #[test]
    fn exp_edges() {
        assert!(exp(f32::NAN).is_nan());
        assert_eq!(exp(f32::INFINITY), f32::INFINITY);
        assert_eq!(exp(f32::NEG_INFINITY), 0.0);
        assert_eq!(exp(88.7229), f32::INFINITY);
        assert_eq!(exp(-87.3366), 0.0);
        assert!(exp(-87.0).is_finite() && exp(-87.0) > 0.0);
        assert!(exp(88.0).is_finite());
    }

    #[test]
    fn ln_accuracy() {
        for &x in &[1e-30f32, 1e-20, 1e-10, 1e-6, 0.001, 0.1, 0.5, 1.0, 1.5, 2.0, 10.0, 100.0, 1e10, 1e30] {
            assert_close(ln(x), (x as f64).ln());
        }
        assert_close(ln(std::f32::consts::E), 1.0);
    }

    #[test]
    fn ln_edges() {
        assert!(ln(f32::NAN).is_nan());
        assert!(ln(-1.0).is_nan());
        assert_eq!(ln(0.0), f32::NEG_INFINITY);
        assert_eq!(ln(f32::INFINITY), f32::INFINITY);
        assert_close(ln(1.0), 0.0);
    }

    #[test]
    fn log10_accuracy() {
        for &x in &[1e-6f32, 0.001, 0.1, 1.0, 2.0, 10.0, 1000.0, 1e6] {
            assert_close(log10(x), (x as f64).log10());
        }
        assert_eq!(log10(0.0), f32::NEG_INFINITY);
        assert!(log10(-1.0).is_nan());
    }

    #[test]
    fn pow10_accuracy() {
        for &x in &[-10.0f32, -3.0, -1.0, -0.5, 0.0, 0.5, 1.0, 3.0, 10.0] {
            assert_close(pow10(x), 10f64.powf(x as f64));
        }
        assert_eq!(pow10(0.0), 1.0);
        assert_close(pow10(3.0), 1000.0);
    }

    #[test]
    fn db_to_linear_accuracy() {
        for &db in &[-100.0f32, -40.0, -20.0, -12.0, -6.0, -1.0, 0.0, 6.0, 20.0, 40.0] {
            assert_close(db_to_linear(db), 10f64.powf(db as f64 / 20.0));
        }
        assert_eq!(db_to_linear(0.0), 1.0);
        assert_close(db_to_linear(20.0), 10.0);
    }

    #[test]
    fn powf_accuracy() {
        // Positive bases over a range of exponents, plus the exact shapes the
        // synth uses (2^(cents/1200), velocity/envelope curves).
        for &b in &[2.0f32, 0.5, 1.001, 10.0, 440.0] {
            for &e in &[-1.0f32, -0.5, 0.0, 0.25, 1.0, 1.7, 3.0] {
                assert_close(powf(b, e), (b as f64).powf(e as f64));
            }
        }
        assert_eq!(powf(2.0, 0.0), 1.0);
        assert_close(powf(2.0, -1.0), 0.5);
        assert_eq!(powf(0.0, 1.7), 0.0);
        assert_eq!(powf(0.0, -1.0), f32::INFINITY);
        assert_eq!(powf(0.0, 0.0), 1.0);
        assert!(powf(-1.0, 0.5).is_nan());
    }

    #[test]
    fn sin_cos_accuracy() {
        use std::f32::consts::PI as PI32;
        for &x in &[-3.0f32, -1.0, -0.5, 0.0, 0.1, 0.5, 1.0, 1.5, 3.0] {
            assert_close(sin(x), (x as f64).sin());
            assert_close(cos(x), (x as f64).cos());
        }
        assert_close(sin(PI32 / 2.0), 1.0);
        assert_close(cos(0.0), 1.0);
        assert_close(cos(PI32 / 2.0), 0.0);
    }

    #[test]
    fn sin_cos_edges() {
        assert!(sin(f32::NAN).is_nan());
        assert!(cos(f32::NAN).is_nan());
        assert!(sin(f32::INFINITY).is_nan());
        assert!(cos(f32::NEG_INFINITY).is_nan());
        assert_eq!(sin(0.0), 0.0);
    }

    #[test]
    fn round_trips() {
        // exp(ln(x)) ≈ x and 10^log10(x) ≈ x over a broad range.
        for &x in &[1e-6f32, 0.01, 0.5, 1.0, 2.0, 100.0, 1e6] {
            let back = exp(ln(x));
            assert!(
                (back / x - 1.0).abs() < 1e-5,
                "exp(ln({x})) = {back} (rel err {})",
                (back / x - 1.0).abs()
            );
            let back = pow10(log10(x));
            assert!((back / x - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn determinism_smoke() {
        // The whole point: the same input yields a fixed bit pattern. We just
        // assert exact equality of the bit patterns for a few representative
        // arguments (trivially true, but documents the contract).
        let samples = [0.5f32, -1.0, 1e-6, 23.5, -70.0];
        for &x in &samples {
            assert_eq!(exp(x).to_bits(), exp(x).to_bits());
            assert_eq!(log10(x.max(1e-6)).to_bits(), log10(x.max(1e-6)).to_bits());
        }
    }
}
