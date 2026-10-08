# Engine Synth Voice Audit (GURI-563)

This document is the literal side-by-side audit of the Rust engine (`crates/engine/src/synth.rs`)
against the signed-off Audio Lab implementation (`apps/demo/src/audio-engine.ts`).

**Scope**: Every parameter in `SYNTH_VOICES`, dedicated schedulers (epiano/organ/supersaw/triangle/chip/bass/percussion),
envelope math (`scheduleEnvelope` / `compute_envelope`), `NOTE_TAIL_SECONDS`, velocity curves,
filter sweeps, bitcrush/vibrato/tremolo, noise bands, pitch drop, and related constants.

**Rules**:
- TypeScript side is **never changed**.
- All mismatches fixed in Rust **except**:
  - Stereo width / panning (engine is mono by design).
  - Room/echo bus (documented intentional, not part of core voice synth).
  - Anything where the engine render is mono by design.
- For intentional rows: value = "intentional (mono render)" or note the bus.
- Golden render test metrics (catalog grid) must remain green; post-fix peak < 0.9.

All values are from direct code reading (2026-09-04 session).

## Audit Table

| Parameter | Lab value | Rust value | Status |
|-----------|-----------|------------|--------|
| NOTE_TAIL_SECONDS | 0.16 | 0.16 | ✅ |
| MIN_GAIN | 0.0001 | 0.0001 | ✅ |
| General: primary/secondary types (warm/glass/pulse/pluck) | saw/tri ; sine/sine ; saw/tri ; saw/tri | same (Wave::Saw/Triangle/Sine) | ✅ |
| warm: secondaryRatio | 1.002 | 1.002 | ✅ |
| warm: secondaryGain | 0.62 | 0.62 | ✅ |
| warm: detuneCents | 10 | 10 | ✅ |
| warm: gain | 0.068 | 0.068 | ✅ |
| warm: attack/decay/sustain/release | 0.048 / 0.22 / 0.74 / 0.2 | same | ✅ |
| warm: cutoffStart/End | 1600 / 620 | same | ✅ |
| warm: resonance | 0.35 | 0.35 | ✅ |
| warm: pitchDrop | 0 | 0 | ✅ |
| glass: secondaryRatio | 2.003 | 2.003 | ✅ |
| glass: secondaryGain | 0.22 | 0.22 | ✅ |
| glass: detuneCents | 6 | 6 | ✅ |
| glass: gain | 0.07 | 0.07 | ✅ |
| glass: attack/decay/sustain/release | 0.01 / 0.18 / 0.5 / 0.16 | same | ✅ |
| glass: cutoffStart/End | 3800 / 1400 | same | ✅ |
| glass: resonance | 0.45 | 0.45 | ✅ |
| glass: pitchDrop | 0 | 0 | ✅ |
| pulse: secondaryRatio | 0.5 | 0.5 | ✅ |
| pulse: secondaryGain | 0.38 | 0.38 | ✅ |
| pulse: detuneCents | 8 | 8 | ✅ |
| pulse: gain | 0.05 | 0.05 | ✅ |
| pulse: attack/decay/sustain/release | 0.014 / 0.14 / 0.66 / 0.14 | same | ✅ |
| pulse: cutoffStart/End | 1900 / 780 | same | ✅ |
| pulse: resonance | 0.4 | 0.4 | ✅ |
| pulse: pitchDrop | 0 | 0 | ✅ |
| pluck: secondaryRatio | 2 | 2 | ✅ |
| pluck: secondaryGain | 0.16 | 0.16 | ✅ |
| pluck: detuneCents | 5 | 5 | ✅ |
| pluck: gain | 0.05 | 0.05 | ✅ |
| pluck: attack/decay/sustain/release | 0.004 / 0.09 / 0.22 / 0.08 | same | ✅ |
| pluck: cutoffStart/End | 3400 / 720 | same | ✅ |
| pluck: resonance | 0.9 | 0.9 | ✅ |
| pluck: pitchDrop | 0.004 | 0.004 | ✅ |
| General filter: vel brightness | 0.72 + vel*0.48 | same | ✅ |
| General filter: end vel factor | cutoffEnd * (0.82 + vel*0.28) | same | ✅ |
| General Q | resonance + (melody ? 0.25 : 0) | same | ✅ |
| General peak | gain * pow(max(0.02,vel),0.82) * (mel?1.18:1) | velocity_curve + 1.18 ternary | ✅ |
| Envelope clamps (att/dec/rel) | clamp(att,0.001,dur*0.24); clamp(dec,0.001,dur*0.46); clamp(rel,0.02,0.24) | identical in compute_envelope | ✅ |
| Envelope decay target | min(dur*0.7, attE + cl(dec)) | same | ✅ |
| Envelope curves | exp ramp (pow for frac) | identical powf frac simulation | ✅ |
| Melody boost | 1.18x on peak for role=melody | same | ✅ |
| pitchDrop ramp time | 0.022s | 0.022 | ✅ |
| epiano: body detunes | -7 / +7 | same (7/1200 cents) | ✅ |
| epiano: tine ratio | 2.001 + vel*0.003 | same | ✅ |
| epiano: tine detune cents | 3 | same (via *2^(3/1200)) | ✅ |
| epiano: tremolo freq | 4.65 + (pitch % 5)*0.07 | same (now exact v.pitch) | ✅ (fixed approx) |
| epiano: tremolo base/depth | 0.975 / (0.018 + vel*0.008) | same | ✅ |
| epiano: tine peak | 0.11 + pow(vel,1.7)*0.38 | same | ✅ |
| epiano: tine decay | 0.09 + (1-vel)*0.08 | same | ✅ |
| epiano: filter start | 1100 + pow(vel,1.4)*2200 | same | ✅ |
| epiano: filter end | 780 + vel*420 | same | ✅ |
| epiano: filter ramp | min(end, start+0.28) | v.duration.min(0.28) | ✅ |
| epiano: env peak/sus/att/dec/rel | 0.12*pow(vel,0.78) / 0.48+(1-v)*0.12 / 0.012 / 0.18+(1-v)*0.08 / 0.16 | same | ✅ |
| organ: drawbar gains | 0.42/0.28/0.16 (ratios 1/2/3) | same | ✅ |
| organ: tremolo | 5.4 Hz depth 0.08 base 0.92 | same | ✅ |
| organ: env peak (mel/non) | (mel?0.09:0.034)*pow(vel,0.8) | same | ✅ |
| organ: env sus/att/dec/rel | 0.7 / 0.03 / 0.18 / 0.2 | same | ✅ |
| supersaw: detunes | -11/0/13 | same | ✅ |
| supersaw: filter start | 2400 + vel*900 | same | ✅ |
| supersaw: filter end | 1100 | same | ✅ |
| supersaw: filter ramp | min(dur, 0.22) | same | ✅ |
| supersaw: filter Q | 0.4 | 0.4 | ✅ |
| supersaw: env peak (mel/non) | (mel?0.034:0.016)*pow(vel,0.8) | same | ✅ |
| supersaw: env sus/att/dec/rel | 0.62 / 0.02 / 0.14 / 0.16 | same | ✅ |
| triangle bass: body | triangle | same | ✅ |
| triangle: env peak | 0.1 * pow(vel,0.75) | same | ✅ |
| triangle: env sus/att/dec/rel | 0.7 / 0.004 / 0.05 / 0.04 | same | ✅ |
| chip: octave gain (lead/harm) | 0.12 / 0.04 | same | ✅ |
| chip: bitcrush steps (lead/harm) | 28 / 48 | same | ✅ |
| chip: vibrato freq (lead/harm) | 5.7 / 0.8 | same | ✅ |
| chip: vibrato depth cents | 16 / 4 | same | ✅ |
| chip: vibrato only on primary | yes (detune) | yes (freq shift on s1) | ✅ |
| chip: env peak (lead/harm) | (l?0.032:0.011)*pow(vel,0.82) | same | ✅ |
| chip: env sus/att/dec/rel | 0.55 / 0.003 / 0.04 / 0.03 | same | ✅ |
| bass: body + sub | triangle + sine*0.5 | same | ✅ |
| bass: body pitchDrop | 0.004 | 0.004 | ✅ |
| bass: bodyGain | 0.42 + vel*0.1 | same | ✅ |
| bass: subGain | 0.95 | 0.95 | ✅ |
| bass: filter start | 520 + vel*680 | same | ✅ |
| bass: filter end | (f*2.4).clamp(180,420) | same | ✅ |
| bass: filter ramp | min(end, start+0.16) | v.duration.min(0.16) | ✅ (fixed) |
| bass: filter Q | 0.7 + vel*0.35 | same | ✅ |
| bass: env peak | 0.12 * pow(vel,0.78) | same | ✅ |
| bass: env sus/att/dec/rel | 0.62 / 0.008 / 0.12 / 0.11 | same | ✅ |
| kick: freq start/end/ramp | 162+vel*18 → 49 / 0.12s | same | ✅ |
| kick: tone gain/env | 0.27*vel → MIN / 0.22s | same | ✅ |
| kick: noise gain/dur/filter | 0.045*vel / 0.018s / hp4800 Q0.65 | same | ✅ (now correct HP) |
| snare: tone freq start/end/ramp | 205 → 142 / 0.085s | same | ✅ |
| snare: tone gain/env | 0.105*vel → MIN / 0.12s | same | ✅ |
| snare: noise gain/dur/filter | 0.205*vel / 0.16s / bp2350 Q0.72 | same | ✅ (now correct BP) |
| hat: noise gain/dur/filter | 0.12*vel / 0.08s / hp6200 Q0.35 | same | ✅ (now correct HP) |
| tom: body freq start | 155 + u*58 | same (det unit) | ✅ |
| tom: body ramp | *0.58 / 0.15s | same | ✅ |
| tom: overtone start/ratio/ramp/g | *1.63 → *0.92 / 0.11s / 0.23 | same | ✅ |
| tom: env gain | 0.17*vel → MIN / 0.19s | same | ✅ |
| tom: noise gain/dur/filter | 0.032*vel / 0.026s / bp1750 Q0.8 | same | ✅ (now correct BP) |
| width / panning (all voices) | width 0.16-0.34, pans ± | N/A (mono) | intentional (mono render) |
| room/echo bus + sends (supersaw) | present in lab master | absent (synth only) | intentional (documented bus) |
| master highpass/compressor/limiter | in lab | N/A (engine returns raw mix) | intentional (mono render) |
| noise buffer / per-hit offset | lab: per-seed offset into fixed lfsr buf | now: per-voice lfsr pre-advanced by exact offset | ✅ (fixed for determinism) |
| epiano pitch %5 for trem | event.pitch % 5 | v.pitch (exact) | ✅ (fixed) |
| biquad for noise (hp/bp) | lab uses correct type per band | was always LP; now correct modes | ✅ (fixed) |

**Summary**: ~65+ individual parameters/knobs audited. All core voice identity, envelopes, filters, modulations now match after fixes. 3 rows marked intentional (mono design + documented bus). No other unexplained diffs remain.

The render golden test (catalog grid 3phrases @48000) metrics continue to pass post-fix (peak <0.9, variation, duration).

## Verification commands (run in this dir)
```bash
cargo test -p gamestruments-engine
cargo clippy -p gamestruments-engine --all-targets -- -D warnings
cargo build -p gamestruments-godot
```

Listening pack is generated via the extended `render_listen` example (see below; defaults to `target/listen/`).
