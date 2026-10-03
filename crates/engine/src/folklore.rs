//! Folklore: a seeded chacarera listening prototype.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    FormOrigin, MusicEvent, PortableScore, PortableSection, SongForm, SongFormStep,
    SCORE_SCHEMA_VERSION,
};
use crate::theory::{mode_intervals, scale_pitch, NOTE_NAMES};

pub const GENERATOR_VERSION: &str = "0.2.0-prototype";
const EIGHTH: u32 = 480;
const BEAT: u32 = 960;
const BAR: u32 = 3 * BEAT;
const LOWEST_GUITAR_PITCH: i32 = 40;

#[derive(Clone, Debug)]
pub struct FolkloreInput {
    pub secret: String,
    pub seed: String,
    pub energy: f64,
    pub complexity: f64,
    pub brightness: f64,
    pub syncopation: f64,
}

#[derive(Clone, Copy)]
struct Traits {
    energy: f64,
    complexity: f64,
    brightness: f64,
    syncopation: f64,
}

impl From<&FolkloreInput> for Traits {
    fn from(input: &FolkloreInput) -> Self {
        fn normal(value: f64) -> f64 {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.5
            }
        }
        Self {
            energy: normal(input.energy),
            complexity: normal(input.complexity),
            brightness: normal(input.brightness),
            syncopation: normal(input.syncopation),
        }
    }
}

fn subseed(input: &FolkloreInput, domain: &str) -> u32 {
    hash_text(&format!(
        "folklore/{GENERATOR_VERSION}\0{}\0{}\0{domain}",
        input.secret, input.seed
    ))
}

const ROOTS: [i32; 5] = [7, 9, 4, 2, 0];

pub fn folklore_root_pitch_class(input: &FolkloreInput) -> i32 {
    ROOTS[subseed(input, "root") as usize % ROOTS.len()]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Chord {
    Tonic,
    Subdominant,
    Dominant,
    Relative,
    Sixth,
}

impl Chord {
    fn notes(self) -> &'static [i32] {
        match self {
            Self::Tonic => &[0, 3, 7],
            Self::Subdominant => &[5, 8, 12],
            Self::Dominant => &[7, 11, 14, 17],
            Self::Relative => &[3, 7, 10],
            Self::Sixth => &[8, 12, 15],
        }
    }

    fn bass(self) -> i32 {
        self.notes()[0]
    }
}

use Chord::{Dominant as V7, Relative as III, Sixth as VI, Subdominant as Iv, Tonic as I};

struct SectionPlan {
    id: &'static str,
    label: &'static str,
    feeling: &'static str,
    color: &'static str,
    harmony: &'static [Chord],
    lead: bool,
}

const PLANS: &[SectionPlan] = &[
    SectionPlan {
        id: "introduccion",
        label: "Introducción",
        feeling: "invitation",
        color: "#6B4423",
        harmony: &[I, Iv, V7, I],
        lead: false,
    },
    SectionPlan {
        id: "primera",
        label: "Primera",
        feeling: "verse",
        color: "#8B5A2B",
        harmony: &[I, Iv, V7, I, III, VI, V7, I],
        lead: true,
    },
    SectionPlan {
        id: "interludio",
        label: "Interludio",
        feeling: "reflection",
        color: "#556B2F",
        harmony: &[III, VI, V7, I],
        lead: false,
    },
    SectionPlan {
        id: "segunda",
        label: "Segunda",
        feeling: "verse",
        color: "#8B5A2B",
        harmony: &[I, Iv, V7, I, III, VI, V7, I],
        lead: true,
    },
    SectionPlan {
        id: "estribillo",
        label: "Estribillo",
        feeling: "refrain",
        color: "#CD853F",
        harmony: &[III, VI, Iv, V7, III, VI, V7, I],
        lead: true,
    },
    SectionPlan {
        id: "cierre",
        label: "Cierre",
        feeling: "resolve",
        color: "#5C4033",
        harmony: &[Iv, V7, V7, I],
        lead: true,
    },
];

struct Composer {
    tonic: i32,
    traits: Traits,
    hook: [usize; 4],
}

struct Part<'a> {
    id: &'a str,
    events: Vec<MusicEvent>,
    next_id: usize,
}

impl Part<'_> {
    fn note(&mut self, lane: &str, tick: u32, duration: u32, velocity: f64, pitch: i32) {
        self.push(MusicEvent::Note {
            id: String::new(),
            section: self.id.into(),
            lane: lane.into(),
            start_tick: tick,
            duration_ticks: duration,
            velocity: velocity.clamp(0.0, 1.0),
            pitch: pitch as u8,
            voice: "nylon-guitar".into(),
            role: (lane == "melody").then(|| "melody".into()),
        });
    }

    fn drum(&mut self, tick: u32, voice: &str, velocity: f64) {
        self.push(MusicEvent::Percussion {
            id: String::new(),
            section: self.id.into(),
            lane: "bombo".into(),
            start_tick: tick,
            duration_ticks: 240,
            velocity: velocity.clamp(0.0, 1.0),
            voice: voice.into(),
        });
    }

    fn push(&mut self, mut event: MusicEvent) {
        let id = format!("{}-{}", self.id, self.next_id);
        match &mut event {
            MusicEvent::Note { id: event_id, .. } | MusicEvent::Percussion { id: event_id, .. } => {
                *event_id = id;
            }
        }
        self.next_id += 1;
        self.events.push(event);
    }
}

impl Composer {
    fn section(&self, plan: &SectionPlan, input: &FolkloreInput) -> PortableSection {
        let mut part = Part {
            id: plan.id,
            events: Vec::new(),
            next_id: 0,
        };
        let mut rng = DeterministicRandom::new(subseed(input, plan.id));
        let count = plan.harmony.len();
        for (bar, &chord) in plan.harmony.iter().enumerate() {
            let start = bar as u32 * BAR;
            let closing = bar + 1 == count;
            let gain = (0.78 + 0.21 * self.traits.energy) * if closing { 0.86 } else { 1.0 };
            if !closing || plan.id != "cierre" {
                // Bombo skin in 3/4 against dry rim and guitar accents in 6/8.
                part.drum(start, "bombo-rim", 0.35 * gain);
                part.drum(start + BEAT, "bombo", 0.71 * gain);
                part.drum(start + 3 * EIGHTH, "bombo-rim", 0.48 * gain);
                part.drum(start + 2 * BEAT, "bombo", 0.82 * gain);
                if self.traits.complexity > 0.68 && bar % 4 == 2 {
                    part.drum(start + 5 * EIGHTH, "bombo-rim", 0.27 * gain);
                }
            } else {
                part.drum(start, "bombo", 0.73 * gain);
            }

            // Three walking bass beats; sparse rasgueado at the two compound accents.
            for beat in 0..3 {
                if closing && plan.id == "cierre" && beat != 0 {
                    break;
                }
                let bass = self.tonic - 12
                    + if beat == 1 {
                        chord.notes()[2] - 12
                    } else {
                        chord.bass()
                    };
                part.note(
                    "bass",
                    start + beat * BEAT,
                    580,
                    (0.47 + 0.06 * rng.next()) * gain,
                    LOWEST_GUITAR_PITCH + (bass - LOWEST_GUITAR_PITCH).rem_euclid(12),
                );
            }
            let strums: &[u32] = if closing && plan.id == "cierre" {
                &[0]
            } else {
                &[0, 3 * EIGHTH]
            };
            for (index, &offset) in strums.iter().enumerate() {
                for (finger, &interval) in chord.notes().iter().enumerate() {
                    let spread = (finger as u32) * (12 + (self.traits.syncopation * 16.0) as u32);
                    part.note(
                        "rasgueado",
                        start + offset + spread,
                        370,
                        (0.27 + 0.06 * self.traits.brightness + 0.03 * rng.next())
                            * gain
                            * if index == 0 { 0.86 } else { 1.0 },
                        self.tonic + interval,
                    );
                }
            }

            if plan.lead || (bar >= 2 && plan.id != "interludio") {
                self.melody(&mut part, &mut rng, bar, chord, plan);
            } else if plan.id == "interludio" && bar % 2 == 0 {
                part.note(
                    "melody",
                    start + 3 * EIGHTH,
                    900,
                    0.56 * gain,
                    self.tonic + 12 + chord.notes()[1],
                );
            }
        }
        part.events.sort_by_key(MusicEvent::start_tick);
        PortableSection {
            id: plan.id.into(),
            label: plan.label.into(),
            feeling: plan.feeling.into(),
            color: plan.color.into(),
            length_ticks: count as u32 * BAR,
            events: part.events,
        }
    }

    fn melody(
        &self,
        part: &mut Part<'_>,
        rng: &mut DeterministicRandom,
        bar: usize,
        chord: Chord,
        plan: &SectionPlan,
    ) {
        let start = bar as u32 * BAR;
        let final_bar = bar + 1 == plan.harmony.len();
        let answer = bar % 4 >= 2;
        let shift = if plan.id == "estribillo" { 1 } else { 0 };
        let tones = chord.notes();
        let first = tones[(self.hook[bar % 4] + shift) % 3];
        let next = tones[(self.hook[(bar + 1) % 4] + shift) % 3];
        let strength = (0.72 + 0.15 * self.traits.energy) * (0.96 + 0.06 * rng.next());
        let octave = self.tonic + 12;
        // A recurring call, an answering contour, and a small silence before each resolution.
        part.note(
            "melody",
            start,
            if final_bar { 1250 } else { 710 },
            strength,
            octave + first,
        );
        if final_bar {
            part.note(
                "melody",
                start + 3 * EIGHTH,
                3 * EIGHTH,
                strength * 0.88,
                octave,
            );
            return;
        }
        if self.traits.complexity > 0.25 && bar % 4 != 3 {
            let intervals = mode_intervals("aeolian");
            let degree = ((self.hook[bar % 4] + bar) % 7) as i32;
            let passing = scale_pitch(octave, degree, &intervals);
            let offset = if self.traits.syncopation > 0.55 {
                EIGHTH
            } else {
                BEAT
            };
            part.note("melody", start + offset, 340, strength * 0.7, passing);
        }
        part.note(
            "melody",
            start + 3 * EIGHTH,
            if bar % 4 == 3 { 700 } else { 570 },
            strength * if answer { 0.94 } else { 0.84 },
            octave + next,
        );
        if chord == V7 && (bar + 1) % 4 == 3 {
            // Harmonic minor's raised seventh anticipates the tonic in the next bar.
            part.note(
                "melody",
                start + 5 * EIGHTH,
                380,
                strength * 0.78,
                octave + 11,
            );
        } else if self.traits.complexity > 0.7 && bar % 4 == 1 {
            part.note(
                "melody",
                start + 5 * EIGHTH,
                330,
                strength * 0.62,
                octave + tones[1],
            );
        }
    }
}

pub fn generate_folklore(
    input: &FolkloreInput,
    arrangement: &str,
) -> Result<PortableScore, String> {
    let arrangement = match arrangement {
        "" | "seeded" => "seeded",
        "all-phases" => "all-phases",
        _ => return Err(format!("unsupported folklore arrangement {arrangement}")),
    };
    let traits = Traits::from(input);
    let root = folklore_root_pitch_class(input);
    let mut motif_rng = DeterministicRandom::new(subseed(input, "hook"));
    let composer = Composer {
        tonic: 48 + root,
        traits,
        hook: std::array::from_fn(|_| motif_rng.integer(3) as usize),
    };
    let sections = PLANS
        .iter()
        .map(|plan| composer.section(plan, input))
        .collect();
    let order: &[&str] = if arrangement == "seeded" {
        &[
            "introduccion",
            "primera",
            "interludio",
            "segunda",
            "interludio",
            "estribillo",
            "primera",
            "estribillo",
            "cierre",
        ]
    } else {
        &[
            "introduccion",
            "primera",
            "interludio",
            "segunda",
            "estribillo",
            "cierre",
        ]
    };
    let form = SongForm {
        steps: order
            .iter()
            .map(|section| SongFormStep {
                section: (*section).into(),
                repeats: 1,
            })
            .collect(),
        loop_from: Some(if arrangement == "all-phases" { 0 } else { 1 }),
        origin: Some(FormOrigin::TransitionStart),
    };
    let id_hash = hash_text(&format!(
        "{GENERATOR_VERSION}\0{}\0{}\0{}\0{:016x}:{:016x}:{:016x}:{:016x}",
        input.secret,
        input.seed,
        arrangement,
        traits.energy.to_bits(),
        traits.complexity.to_bits(),
        traits.brightness.to_bits(),
        traits.syncopation.to_bits()
    ));
    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: format!("folklore-{id_hash:08x}"),
        title: format!(
            "Folklore {} minor",
            NOTE_NAMES[root as usize].to_uppercase()
        ),
        bpm: 140.0 + 14.0 * traits.energy,
        beats_per_bar: 3,
        ticks_per_beat: BEAT,
        crossfade_bars: 1.0,
        default_section: "introduccion".into(),
        sections,
        rules: vec![],
        form: Some(form),
    };
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn input(seed: &str) -> FolkloreInput {
        FolkloreInput {
            secret: "guitar".into(),
            seed: seed.into(),
            energy: 0.58,
            complexity: 0.47,
            brightness: 0.61,
            syncopation: 0.64,
        }
    }

    fn serialized<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string(value).unwrap()
    }

    #[test]
    fn deterministic_and_varied() {
        let a = input("call");
        let baseline = generate_folklore(&a, "seeded").unwrap();
        assert_eq!(
            serialized(&baseline),
            serialized(&generate_folklore(&a, "").unwrap())
        );
        let other = generate_folklore(&input("answer"), "seeded").unwrap();
        assert_ne!(baseline.id, other.id);
        assert_ne!(
            serialized(&baseline.sections[1]),
            serialized(&other.sections[1])
        );
        let mut hotter = a.clone();
        hotter.energy = 0.9;
        assert_ne!(
            baseline.id,
            generate_folklore(&hotter, "seeded").unwrap().id
        );
        assert_ne!(baseline.id, generate_folklore(&a, "all-phases").unwrap().id);
        assert!(generate_folklore(&a, "other").is_err());
    }

    #[test]
    fn form_and_bar_lengths() {
        let a = input("form");
        let score = generate_folklore(&a, "seeded").unwrap();
        assert_eq!(score.validate(), Ok(()));
        assert_eq!(
            score
                .sections
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            PLANS.iter().map(|p| p.id).collect::<Vec<_>>()
        );
        assert_eq!(
            score
                .sections
                .iter()
                .map(|s| s.length_ticks / BAR)
                .collect::<Vec<_>>(),
            vec![4, 8, 4, 8, 8, 4]
        );
        assert!(score.sections.iter().all(|s| s.length_ticks % BAR == 0
            && s.events
                .iter()
                .all(|e| e.start_tick() + e.duration_ticks() <= s.length_ticks)));
        assert_eq!(score.form.unwrap().steps.len(), 9);
        let all = generate_folklore(&a, "all-phases").unwrap();
        assert_eq!(all.form.as_ref().unwrap().loop_from, Some(0));
        assert_eq!(all.form.unwrap().steps.len(), 6);
        assert_eq!(
            score.title,
            format!(
                "Folklore {} minor",
                NOTE_NAMES[folklore_root_pitch_class(&a) as usize].to_uppercase()
            )
        );
    }

    #[test]
    fn cross_rhythm_harmony_and_final_cadence() {
        let a = input("groove");
        let score = generate_folklore(&a, "seeded").unwrap();
        let verse = score.section("primera").unwrap();
        for bar in 0..8 {
            let at = bar * BAR;
            for (offset, voice) in [
                (0, "bombo-rim"),
                (BEAT, "bombo"),
                (3 * EIGHTH, "bombo-rim"),
                (2 * BEAT, "bombo"),
            ] {
                assert!(verse
                    .events
                    .iter()
                    .any(|e| e.start_tick() == at + offset && e.voice() == voice));
            }
            assert!(verse.events.iter().any(|e| matches!(e,
                MusicEvent::Note { lane, start_tick, .. } if lane == "rasgueado" && *start_tick == at + 3 * EIGHTH)));
        }
        let tonic = 48 + folklore_root_pitch_class(&a);
        let bass_at = |bar| {
            verse
                .events
                .iter()
                .find_map(|e| match e {
                    MusicEvent::Note {
                        lane,
                        start_tick,
                        pitch,
                        ..
                    } if lane == "bass" && *start_tick == bar as u32 * BAR => {
                        Some(i32::from(*pitch))
                    }
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(
            [0, 5, 7, 0, 3, 8, 7, 0].map(|interval| LOWEST_GUITAR_PITCH
                + (tonic - 12 + interval - LOWEST_GUITAR_PITCH).rem_euclid(12)),
            std::array::from_fn::<_, 8, _>(bass_at)
        );
        assert!(verse.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, pitch, .. }
            if lane == "rasgueado" && i32::from(*pitch) == tonic + 17)));
        assert!(verse.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, start_tick, pitch, .. }
            if lane == "melody" && *start_tick == 6 * BAR + 5 * EIGHTH && i32::from(*pitch) == tonic + 23)));
        let close = score.section("cierre").unwrap();
        assert!(close.events.iter().any(|e| matches!(e,
            MusicEvent::Note { lane, start_tick, pitch, .. }
            if lane == "melody" && *start_tick == 3 * BAR + 3 * EIGHTH && i32::from(*pitch) == tonic + 12)));
        let ids: HashSet<_> = score
            .sections
            .iter()
            .flat_map(|s| &s.events)
            .map(|e| match e {
                MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
            })
            .collect();
        assert_eq!(
            ids.len(),
            score.sections.iter().map(|s| s.events.len()).sum::<usize>()
        );
    }

    #[test]
    fn extreme_traits_stay_valid_and_change_texture() {
        let mut a = input("bounds");
        a.energy = f64::NAN;
        a.complexity = f64::INFINITY;
        a.brightness = -1.0;
        a.syncopation = 2.0;
        let score = generate_folklore(&a, "seeded").unwrap();
        assert!(score.bpm.is_finite());
        assert_eq!(score.validate(), Ok(()));
        let mut normal = a;
        normal.energy = 0.5;
        normal.complexity = 0.5;
        normal.brightness = 0.0;
        normal.syncopation = 1.0;
        assert_eq!(
            serialized(&score),
            serialized(&generate_folklore(&normal, "seeded").unwrap())
        );
        normal.complexity = 0.1;
        let sparse = generate_folklore(&normal, "seeded").unwrap();
        assert!(
            sparse.section("primera").unwrap().events.len()
                < score.section("primera").unwrap().events.len()
        );
    }
}
