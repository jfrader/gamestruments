use crate::score::MusicEvent;

use std::f32::consts::TAU;

const NOTE_TAIL_SECONDS: f32 = 0.16;
const MIN_GAIN: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VoiceType {
    Warm,
    Glass,
    Pulse,
    Pluck,
    Bass,
    Epiano,
    Organ,
    Supersaw,
    Triangle,
    Chip,
    Kick,
    Snare,
    Hat,
    Tom,
}

struct Biquad {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, x: f32, fc: f32, q: f32, sr: f32) -> f32 {
        let fc = fc.max(20.0).min(sr * 0.49);
        let q = q.max(0.1);
        let omega = std::f32::consts::TAU * (fc / sr);
        let sin_om = omega.sin();
        let cos_om = omega.cos();
        let alpha = sin_om / (2.0 * q);
        // lowpass
        let b0 = (1.0 - cos_om) * 0.5;
        let b1 = 1.0 - cos_om;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_om;
        let a2 = 1.0 - alpha;
        let y = (b0 * x + b1 * self.x1 + b2 * self.x2 - a1 * self.y1 - a2 * self.y2) / a0;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

struct Voice {
    voice_type: VoiceType,
    base_freq: f32,
    velocity: f32,
    is_melody: bool,
    start_phase: f32,
    duration: f32,
    life: f32,
    // osc phases (radians)
    phase1: f32,
    phase2: f32,
    phase3: f32,
    vib_phase: f32,
    trem_phase: f32,
    filt: Biquad,
}

pub struct Synth {
    sample_rate: f32,
    phase: f32,
    voices: Vec<Voice>,
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
        let voice = event.voice();
        let is_melody = event.is_melody();
        if let Some(pitch) = event.pitch() {
            let base_freq = midi_to_freq(pitch);
            let vtype = match voice {
                "warm" => VoiceType::Warm,
                "glass" => VoiceType::Glass,
                "pulse" => VoiceType::Pulse,
                "pluck" => VoiceType::Pluck,
                "chip" => VoiceType::Chip,
                "organ" => VoiceType::Organ,
                "supersaw" => VoiceType::Supersaw,
                "triangle" => VoiceType::Triangle,
                "bass" => VoiceType::Bass,
                "epiano" => VoiceType::Epiano,
                _ => VoiceType::Warm,
            };
            let life = duration as f32 + NOTE_TAIL_SECONDS + 0.05;
            self.voices.push(Voice {
                voice_type: vtype,
                base_freq,
                velocity: velocity.clamp(0.0, 1.0),
                is_melody,
                start_phase: self.phase,
                duration: duration as f32,
                life,
                phase1: 0.0,
                phase2: 0.0,
                phase3: 0.0,
                vib_phase: 0.0,
                trem_phase: 0.0,
                filt: Biquad::new(),
            });
            return;
        }
        // percussion
        let vtype = match voice {
            "kick" => VoiceType::Kick,
            "snare" => VoiceType::Snare,
            "hat" => VoiceType::Hat,
            "tom" => VoiceType::Tom,
            _ => VoiceType::Kick,
        };
        let mut base_freq = 80.0f32;
        let life = match vtype {
            VoiceType::Kick => 0.225,
            VoiceType::Snare => 0.125,
            VoiceType::Hat => 0.085,
            VoiceType::Tom => 0.20,
            _ => 0.12,
        };
        if voice == "tom" {
            if let MusicEvent::Percussion { id, .. } = event {
                let u = deterministic_unit(id);
                base_freq = 155.0 + u * 58.0;
            }
        }
        self.voices.push(Voice {
            voice_type: vtype,
            base_freq,
            velocity: velocity.clamp(0.0, 1.0),
            is_melody: false,
            start_phase: self.phase,
            duration: life,
            life: life + 0.01,
            phase1: 0.0,
            phase2: 0.0,
            phase3: 0.0,
            vib_phase: 0.0,
            trem_phase: 0.0,
            filt: Biquad::new(),
        });
    }

    pub fn fill(&mut self, buffer: &mut [f32]) {
        let sr = self.sample_rate;
        let dt = 1.0 / sr;
        let mut i = 0;
        while i < buffer.len() {
            let t = self.phase;
            let mut mix = 0.0f32;
            let mut j = 0;
            while j < self.voices.len() {
                let age = (t - self.voices[j].start_phase).max(0.0);
                if age >= self.voices[j].life {
                    self.voices.swap_remove(j);
                    continue;
                }
                let contrib = Synth::generate_voice_sample(
                    &mut self.noise_state,
                    &mut self.voices[j],
                    age,
                    sr,
                    dt,
                );
                mix += contrib;
                j += 1;
            }
            buffer[i] = mix.clamp(-0.95, 0.95);
            self.phase += dt;
            i += 1;
        }
    }

    fn generate_voice_sample(
        noise_state: &mut u32,
        v: &mut Voice,
        age: f32,
        sr: f32,
        dt: f32,
    ) -> f32 {
        let vel = v.velocity;
        let is_mel = v.is_melody;
        let base = v.base_freq;
        match v.voice_type {
            VoiceType::Warm | VoiceType::Glass | VoiceType::Pulse | VoiceType::Pluck => {
                let (
                    primary,
                    secondary,
                    sec_r,
                    sec_g,
                    det_c,
                    g,
                    att,
                    dec,
                    sus,
                    rel,
                    cs,
                    ce,
                    res,
                    pd,
                ) = match v.voice_type {
                    VoiceType::Warm => (
                        Wave::Saw,
                        Wave::Triangle,
                        1.002,
                        0.62,
                        10.0,
                        0.068,
                        0.048,
                        0.22,
                        0.74,
                        0.2,
                        1600.0,
                        620.0,
                        0.35,
                        0.0,
                    ),
                    VoiceType::Glass => (
                        Wave::Sine,
                        Wave::Sine,
                        2.003,
                        0.22,
                        6.0,
                        0.07,
                        0.01,
                        0.18,
                        0.5,
                        0.16,
                        3800.0,
                        1400.0,
                        0.45,
                        0.0,
                    ),
                    VoiceType::Pulse => (
                        Wave::Saw,
                        Wave::Triangle,
                        0.5,
                        0.38,
                        8.0,
                        0.05,
                        0.014,
                        0.14,
                        0.66,
                        0.14,
                        1900.0,
                        780.0,
                        0.4,
                        0.0,
                    ),
                    VoiceType::Pluck => (
                        Wave::Saw,
                        Wave::Triangle,
                        2.0,
                        0.16,
                        5.0,
                        0.05,
                        0.004,
                        0.09,
                        0.22,
                        0.08,
                        3400.0,
                        720.0,
                        0.9,
                        0.004,
                    ),
                    _ => unreachable!(),
                };
                let f1 = compute_freq(base, age, pd) * 2f32.powf(-det_c / 1200.0);
                let f2 = compute_freq(base * sec_r, age, pd * 0.5) * 2f32.powf(det_c / 1200.0);
                let s1 = generate_osc(v.phase1, primary);
                v.phase1 += TAU * f1 * dt;
                let s2 = generate_osc(v.phase2, secondary);
                v.phase2 += TAU * f2 * dt;
                let mut sig = s1 + s2 * sec_g;
                // filter sweep
                let bright = 0.72 + vel * 0.48;
                let mut fc = (cs * bright).min(sr * 0.44);
                let end_fc = (ce * (0.82 + vel * 0.28)).max(120.0);
                let ramp_d = v.duration.min(dec);
                if ramp_d > 0.0 && age < ramp_d {
                    let frac = age / ramp_d;
                    fc *= (end_fc / fc).powf(frac);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = res + if is_mel { 0.25 } else { 0.0 };
                sig = v.filt.process(sig, fc, q, sr);
                let peak = g * velocity_curve(vel, 0.82) * if is_mel { 1.18 } else { 1.0 };
                let env = compute_envelope(age, v.duration, peak, sus, att, dec, rel);
                sig * env
            }
            VoiceType::Bass => {
                let f_body = compute_freq(base, age, 0.004);
                let s_body = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * f_body * dt;
                let f_sub = base * 0.5;
                let s_sub = generate_osc(v.phase2, Wave::Sine);
                v.phase2 += TAU * f_sub * dt;
                let bg = 0.42 + vel * 0.1;
                let sg = 0.95;
                let mut sig = s_body * bg + s_sub * sg;
                let mut fc = 520.0 + vel * 680.0;
                let end_fc = (base * 2.4).clamp(180.0, 420.0);
                let ramp_d = 0.16f32;
                if age < ramp_d {
                    let frac = age / ramp_d;
                    fc *= (end_fc / fc).powf(frac);
                } else {
                    fc = end_fc;
                }
                let q = 0.7 + vel * 0.35;
                sig = v.filt.process(sig, fc, q, sr);
                let peak = 0.12 * velocity_curve(vel, 0.78);
                let env = compute_envelope(age, v.duration, peak, 0.62, 0.008, 0.12, 0.11);
                sig * env
            }
            VoiceType::Epiano => {
                let det = 7.0 / 1200.0;
                let f_l = base * 2f32.powf(-det);
                let f_r = base * 2f32.powf(det);
                let s_l = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * f_l * dt;
                let s_r = generate_osc(v.phase2, Wave::Sine);
                v.phase2 += TAU * f_r * dt;
                let tine_f = base * (2.001 + vel * 0.003) * 2f32.powf(3.0 / 1200.0);
                let s_t = generate_osc(v.phase3, Wave::Sine);
                v.phase3 += TAU * tine_f * dt;
                let bg = 0.62;
                let mut sig = (s_l + s_r) * bg + s_t * 0.0; // tine g separate
                                                            // tine pre gain ramp
                let tine_peak = 0.11 + vel.powf(1.7) * 0.38;
                let tine_dec = 0.09 + (1.0 - vel) * 0.08;
                let tine_d_t = v.duration.min(tine_dec);
                let mut tg = 0.012;
                if age < 0.004 {
                    let fr = age / 0.004;
                    let tgt = tine_peak * 0.7;
                    tg = MIN_GAIN * (tgt / MIN_GAIN).powf(fr);
                } else if age < tine_d_t {
                    let fr = (age - 0.004) / (tine_d_t - 0.004).max(1e-6);
                    let tgt = tine_peak * 0.7;
                    tg = tgt * (0.012 / tgt).powf(fr);
                }
                sig += s_t * tg;
                // filter
                let mut fc = 1100.0 + vel.powf(1.4) * 2200.0;
                let end_fc = 780.0 + vel * 420.0;
                let ramp_d = v.duration.min(0.28);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= (end_fc / fc).powf(fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.45 + vel * 0.35;
                sig = v.filt.process(sig, fc, q, sr);
                let peak = 0.12 * velocity_curve(vel, 0.78);
                let sus = 0.48 + (1.0 - vel) * 0.12;
                let att = 0.012;
                let dec = 0.18 + (1.0 - vel) * 0.08;
                let rel = 0.16;
                let env = compute_envelope(age, v.duration, peak, sus, att, dec, rel);
                // trem
                let tr = 0.975 + v.trem_phase.sin() * (0.018 + vel * 0.008);
                v.trem_phase += TAU
                    * (4.65
                        + ((/* pitch approx from f */(base.log2() * 12.0 + 69.0) as i32 % 5)
                            as f32)
                            * 0.07)
                    * dt;
                sig * env * tr
            }
            VoiceType::Organ => {
                let gs = [0.42f32, 0.28, 0.16];
                let rs = [1.0f32, 2.0, 3.0];
                let mut mix = 0.0;
                let ps = [&mut v.phase1, &mut v.phase2, &mut v.phase3];
                for (i, (&r, &g)) in rs.iter().zip(gs.iter()).enumerate() {
                    let f = base * r;
                    mix += ps[i].sin() * g;
                    *ps[i] += TAU * f * dt;
                }
                let tr = 0.92 + v.trem_phase.sin() * 0.08;
                v.trem_phase += TAU * 5.4 * dt;
                let peak = (if is_mel { 0.09 } else { 0.034 }) * velocity_curve(vel, 0.8);
                let env = compute_envelope(age, v.duration, peak, 0.7, 0.03, 0.18, 0.2);
                mix * env * tr
            }
            VoiceType::Supersaw => {
                let dets = [-11.0f32, 0.0, 13.0];
                let mut mix = 0.0;
                let ps = [&mut v.phase1, &mut v.phase2, &mut v.phase3];
                for (i, &c) in dets.iter().enumerate() {
                    let f = base * 2f32.powf(c / 1200.0);
                    mix += saw_phase(*ps[i]);
                    *ps[i] += TAU * f * dt;
                }
                let mut fc = 2400.0 + vel * 900.0;
                let end_fc = 1100.0;
                let ramp_d = v.duration.min(0.22);
                if ramp_d > 0.0 && age < ramp_d {
                    let fr = age / ramp_d;
                    fc *= (end_fc / fc).powf(fr);
                } else if age >= ramp_d {
                    fc = end_fc;
                }
                let q = 0.4;
                let sig = v.filt.process(mix, fc, q, sr);
                let peak = (if is_mel { 0.034 } else { 0.016 }) * velocity_curve(vel, 0.8);
                let env = compute_envelope(age, v.duration, peak, 0.62, 0.02, 0.14, 0.16);
                sig * env
            }
            VoiceType::Triangle => {
                let s = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * base * dt;
                let peak = 0.1 * velocity_curve(vel, 0.75);
                let env = compute_envelope(age, v.duration, peak, 0.7, 0.004, 0.05, 0.04);
                s * env
            }
            VoiceType::Chip => {
                let is_lead = is_mel;
                let oct_g = if is_lead { 0.12 } else { 0.04 };
                let steps = if is_lead { 28 } else { 48 };
                let vib_f = if is_lead { 5.7 } else { 0.8 };
                let vib_d = if is_lead { 16.0 } else { 4.0 };
                let vib = v.vib_phase.sin() * vib_d;
                v.vib_phase += TAU * vib_f * dt;
                let f1 = base * 2f32.powf(vib / 1200.0);
                let s1 = generate_osc(v.phase1, Wave::Square);
                v.phase1 += TAU * f1 * dt;
                let f2 = base * 2.0;
                let s2 = generate_osc(v.phase2, Wave::Square);
                v.phase2 += TAU * f2 * dt;
                let mut sig = s1 + s2 * oct_g;
                sig = bitcrush(sig, steps);
                let peak = (if is_lead { 0.032 } else { 0.011 }) * velocity_curve(vel, 0.82);
                let env = compute_envelope(age, v.duration, peak, 0.55, 0.003, 0.04, 0.03);
                sig * env
            }
            VoiceType::Kick => {
                let start_f = 162.0 + vel * 18.0;
                let end_f = 49.0;
                let tf = if age < 0.12 {
                    start_f * (end_f / start_f).powf(age / 0.12)
                } else {
                    end_f
                };
                let s = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * tf * dt;
                let g0 = (0.27 * vel).max(MIN_GAIN);
                let te = if age < 0.22 {
                    g0 * (MIN_GAIN / g0).powf(age / 0.22)
                } else {
                    MIN_GAIN
                };
                let tone = s * te;
                let ng = if age < 0.018 {
                    let g0 = (0.045 * vel).max(MIN_GAIN);
                    g0 * (MIN_GAIN / g0).powf(age / 0.018)
                } else {
                    0.0
                };
                let n = noise(noise_state);
                let nf = v.filt.process(n, 4800.0, 0.65, sr);
                tone + nf * ng
            }
            VoiceType::Snare => {
                let start_f = 205.0f32;
                let end_f = 142.0f32;
                let tf = if age < 0.085 {
                    start_f * (end_f / start_f).powf(age / 0.085_f32)
                } else {
                    end_f
                };
                let s = generate_osc(v.phase1, Wave::Triangle);
                v.phase1 += TAU * tf * dt;
                let g0 = (0.105 * vel).max(MIN_GAIN);
                let te = if age < 0.12 {
                    g0 * (MIN_GAIN / g0).powf(age / 0.12)
                } else {
                    MIN_GAIN
                };
                let tone = s * te;
                let ng = if age < 0.16 {
                    let g0 = (0.205 * vel).max(MIN_GAIN);
                    g0 * (MIN_GAIN / g0).powf(age / 0.16)
                } else {
                    0.0
                };
                let n = noise(noise_state);
                let nf = v.filt.process(n, 2350.0, 0.72, sr);
                tone + nf * ng
            }
            VoiceType::Hat => {
                let ng = if age < 0.08 {
                    let g0 = (0.12 * vel).max(MIN_GAIN);
                    g0 * (MIN_GAIN / g0).powf(age / 0.08)
                } else {
                    0.0
                };
                let n = noise(noise_state);
                let nf = v.filt.process(n, 6200.0, 0.35, sr);
                nf * ng
            }
            VoiceType::Tom => {
                let bs = v.base_freq;
                let bf = if age < 0.15 {
                    bs * (bs * 0.58 / bs).powf(age / 0.15)
                } else {
                    bs * 0.58
                };
                let sb = generate_osc(v.phase1, Wave::Sine);
                v.phase1 += TAU * bf * dt;
                let os = bs * 1.63;
                let oe = bs * 0.92;
                let of = if age < 0.11 {
                    os * (oe / os).powf(age / 0.11)
                } else {
                    oe
                };
                let so = generate_osc(v.phase2, Wave::Triangle);
                v.phase2 += TAU * of * dt;
                let g0 = (0.17 * vel).max(MIN_GAIN);
                let te = if age < 0.19 {
                    g0 * (MIN_GAIN / g0).powf(age / 0.19)
                } else {
                    MIN_GAIN
                };
                let tone = (sb + so * 0.23) * te;
                let ng = if age < 0.026 {
                    let g0 = (0.032 * vel).max(MIN_GAIN);
                    g0 * (MIN_GAIN / g0).powf(age / 0.026)
                } else {
                    0.0
                };
                let n = noise(noise_state);
                let nf = v.filt.process(n, 1750.0, 0.8, sr);
                tone + nf * ng
            }
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

fn midi_to_freq(pitch: u8) -> f32 {
    440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0)
}

fn velocity_curve(v: f32, e: f32) -> f32 {
    v.max(0.02).powf(e)
}

fn clamp_f(v: f32, lo: f32, hi: f32) -> f32 {
    v.max(lo).min(hi)
}

fn compute_freq(base: f32, age: f32, pitch_drop: f32) -> f32 {
    if pitch_drop <= 0.0 {
        base
    } else {
        let start_f = base * (1.0 + pitch_drop);
        let ramp = 0.022;
        if age < ramp {
            let fr = age / ramp;
            start_f * (base / start_f).powf(fr)
        } else {
            base
        }
    }
}

fn compute_envelope(
    age: f32,
    duration: f32,
    peak: f32,
    sustain: f32,
    attack: f32,
    decay: f32,
    release: f32,
) -> f32 {
    let attack_d = clamp_f(attack, 0.001, duration * 0.24);
    let decay_d = clamp_f(decay, 0.001, duration * 0.46);
    let attack_end = attack_d;
    let decay_end = (duration * 0.7).min(attack_end + decay_d);
    let release_t = clamp_f(release, 0.02, 0.24);
    let safe_p = peak.max(MIN_GAIN);
    let sus_g = (safe_p * sustain).max(MIN_GAIN);
    let t = age;
    if t <= 0.0 {
        MIN_GAIN
    } else if t < attack_end {
        let fr = t / attack_end.max(1e-6);
        MIN_GAIN * (safe_p / MIN_GAIN).powf(fr)
    } else if t < decay_end {
        let fr = (t - attack_end) / (decay_end - attack_end).max(1e-6);
        safe_p * (sus_g / safe_p).powf(fr)
    } else if t < duration {
        sus_g
    } else {
        let rel_end = duration + release_t;
        if t < rel_end {
            let fr = (t - duration) / release_t.max(1e-6);
            sus_g * (MIN_GAIN / sus_g).powf(fr)
        } else {
            MIN_GAIN
        }
    }
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Saw,
    Triangle,
    Square,
}

fn generate_osc(phase: f32, wave: Wave) -> f32 {
    match wave {
        Wave::Sine => phase.sin(),
        Wave::Saw => saw_phase(phase),
        Wave::Triangle => {
            let x = 2.0 * (phase / TAU).fract() - 1.0;
            1.0 - 2.0 * x.abs()
        }
        Wave::Square => {
            if (phase / TAU).fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

fn saw_phase(phase: f32) -> f32 {
    2.0 * (phase / TAU).fract() - 1.0
}

fn bitcrush(x: f32, steps: u32) -> f32 {
    let q = steps.max(2) as f32;
    let xc = x.clamp(-1.0, 1.0);
    (xc * q).round() / q
}

fn noise(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn deterministic_unit(seed: &str) -> f32 {
    let mut hash: u32 = 0x811c9dc5;
    for b in seed.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    (hash as f32) / (u32::MAX as f32)
}

use crate::score::PortableScore;

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
