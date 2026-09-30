//! Strategy score: club techno and trance for a match, from the first build
//! orders to the battle and the win.
//!
//! Five sections follow a match: `build` (the groove alone), `expand` (bass
//! and stab join), `tension` (the kick drops out for a breakdown and a roll),
//! `battle` (the drop, everything pumping) and `victory` (the outro). Each
//! seed picks a minor key, a progression, and its bass, stab and arpeggio
//! figures; the style decides how much of the trance pad and lead come in.

use crate::rng::{hash_text, keyed_unit, DeterministicRandom};
use crate::score::{
    AdaptiveCondition, AdaptiveRule, MusicEvent, PortableScore, PortableSection, SongForm,
    SongFormStep, SCORE_SCHEMA_VERSION,
};
use crate::theory::NOTE_NAMES;

pub const GENERATOR_VERSION: &str = "1.0.0";

const TICKS_PER_BEAT: u32 = 960;
/// Threat at or above this plays the battle whatever the phase.
const THREAT_BATTLE: f64 = 0.75;
/// A building economy at or above this moves into the expansion.
const ECONOMY_EXPAND: f64 = 0.6;
const SIXTEENTH: u32 = TICKS_PER_BEAT / 4;
const BAR: u32 = 16 * SIXTEENTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategyStyle {
    Techno,
    Trance,
}

impl StrategyStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "techno" => Ok(Self::Techno),
            "trance" => Ok(Self::Trance),
            other => Err(format!("Unknown strategy style: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Techno => "techno",
            Self::Trance => "trance",
        }
    }

    fn display(self) -> &'static str {
        match self {
            Self::Techno => "Techno",
            Self::Trance => "Trance",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StrategyArrangement {
    /// The sections with no form: the game picks them.
    #[default]
    Original,
    /// Every section once, in match order, looping from the build.
    AllPhases,
}

impl StrategyArrangement {
    /// `seeded` plays the match in order too until the pool composer lands.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" => Ok(Self::Original),
            "all-phases" | "seeded" => Ok(Self::AllPhases),
            other => Err(format!("Unknown strategy arrangement: {other}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrategyInput {
    pub secret: String,
    pub seed: String,
    pub style: StrategyStyle,
    /// Energy: tempo and drum weight.
    pub energy: f64,
    /// Brightness: how open the lead's filter and the pad sit.
    pub brightness: f64,
    /// Complexity: how many ghost sixteenths the hats play.
    pub complexity: f64,
    /// Syncopation: extra off-beat stabs and claps.
    pub syncopation: f64,
}

/// The match phases a game sends, and the label of the section each plays.
pub const STRATEGY_PHASES: [(&str, &str); 10] = [
    ("build", "Build Order"),
    ("scout", "Recon"),
    ("expand", "Expansion"),
    ("research", "Tech Up"),
    ("raid", "Raid"),
    ("tension", "Standoff"),
    ("siege", "Siege"),
    ("battle", "Battle"),
    ("victory", "Victory"),
    ("defeat", "Defeat"),
];

/// Minor-key progressions as semitones above the tonic: i–VI–III–VII,
/// i–VII–VI–VII and i–VI–iv–VII.
const PROGRESSIONS: [[i32; 4]; 3] = [[0, 8, 3, 10], [0, 10, 8, 10], [0, 8, 5, 10]];
/// The defeat's darker loop: i–iv–i–VII.
const DARK_PROGRESSION: [i32; 4] = [0, 5, 0, 10];
/// Chord qualities follow the natural minor: i, iv minor; III, VI, VII major.
fn triad(root: i32) -> [i32; 3] {
    let minor = matches!(root.rem_euclid(12), 0 | 5);
    [root, root + if minor { 3 } else { 4 }, root + 7]
}

/// Stab rhythms (sixteenths in the bar) and arpeggio shapes (chord-tone
/// indices, octave up for 3).
const STABS: [&[u32]; 3] = [&[3, 6, 10], &[2, 7, 10, 14], &[3, 8, 11]];
const ARPS: [[usize; 4]; 3] = [[0, 1, 2, 1], [0, 2, 3, 2], [2, 1, 0, 1]];

struct Plan {
    tonic: i32,
    progression: [i32; 4],
    stab: &'static [u32],
    arp: [usize; 4],
    /// The acid line: per sixteenth, a rest or an interval above the root
    /// with an accent that opens the filter.
    acid: [Option<(i32, bool)>; 16],
}

struct Writer {
    section: &'static str,
    events: Vec<MusicEvent>,
    serial: usize,
}

impl Writer {
    fn new(section: &'static str) -> Self {
        Self {
            section,
            events: Vec::new(),
            serial: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn note(
        &mut self,
        lane: &str,
        voice: &str,
        bar: u32,
        at: u32,
        length: u32,
        pitch: i32,
        velocity: f64,
    ) {
        self.serial += 1;
        self.events.push(MusicEvent::Note {
            id: format!("{}:{lane}:{}", self.section, self.serial),
            section: self.section.into(),
            lane: lane.into(),
            start_tick: bar * BAR + at * SIXTEENTH,
            duration_ticks: length * SIXTEENTH,
            velocity: velocity.clamp(0.04, 0.95),
            pitch: u8::try_from(pitch).expect("strategy pitch stays in MIDI range"),
            voice: voice.into(),
            role: (lane == "lead").then(|| "melody".into()),
        });
    }

    fn hit(&mut self, voice: &str, bar: u32, at: u32, velocity: f64) {
        self.serial += 1;
        self.events.push(MusicEvent::Percussion {
            id: format!("{}:percussion:{}", self.section, self.serial),
            section: self.section.into(),
            lane: "percussion".into(),
            start_tick: bar * BAR + at * SIXTEENTH,
            duration_ticks: SIXTEENTH,
            velocity: velocity.clamp(0.04, 0.95),
            voice: voice.into(),
        });
    }

    fn finish(mut self, label: &str, feeling: &str, color: &str, bars: u32) -> PortableSection {
        self.events.sort_by_key(MusicEvent::start_tick);
        PortableSection {
            id: self.section.into(),
            label: label.into(),
            feeling: feeling.into(),
            color: color.into(),
            length_ticks: bars * BAR,
            events: self.events,
        }
    }
}

/// Layers and endings of one 8-bar block, as bit flags.
type Block = u16;
const KICK: Block = 1;
const HATS: Block = 1 << 1;
const CLAP: Block = 1 << 2;
const BASS: Block = 1 << 3;
const ACID: Block = 1 << 4;
const STAB: Block = 1 << 5;
const PAD: Block = 1 << 6;
const ARP: Block = 1 << 7;
/// Claps roll through the block's last beat into the next.
const FILL: Block = 1 << 8;
/// The kick and bass drop out for the block's last bar.
const GAP: Block = 1 << 9;
/// A snare roll and a riser across the block's last bar.
const ROLL: Block = 1 << 10;
const BLOCK_BARS: u32 = 8;
const GROOVE: Block = KICK | HATS | CLAP | BASS;

/// A section: its 8-bar blocks, whether the bass follows the progression,
/// and how it reads in the Lab.
struct SectionPlan {
    id: &'static str,
    blocks: &'static [Block],
    moving: bool,
    feeling: &'static str,
    color: &'static str,
}

const SECTIONS: [SectionPlan; 10] = [
    SectionPlan {
        id: "build",
        blocks: &[
            KICK | HATS,
            KICK | HATS | CLAP,
            GROOVE,
            GROOVE | STAB | FILL,
        ],
        moving: false,
        feeling: "first orders / the machine starts",
        color: "#4d6b8a",
    },
    SectionPlan {
        id: "scout",
        blocks: &[KICK | HATS | ACID, KICK | HATS | CLAP | ACID | GAP],
        moving: false,
        feeling: "eyes on the map / quiet feelers",
        color: "#4f7f8f",
    },
    SectionPlan {
        id: "expand",
        blocks: &[
            GROOVE,
            GROOVE | STAB,
            GROOVE | STAB | ARP,
            GROOVE | STAB | FILL,
        ],
        moving: true,
        feeling: "new ground / the economy hums",
        color: "#3f8f7a",
    },
    SectionPlan {
        id: "research",
        blocks: &[HATS | ARP | PAD, KICK | HATS | ARP | PAD | ROLL],
        moving: true,
        feeling: "labs humming / something new",
        color: "#5a7fb0",
    },
    SectionPlan {
        id: "raid",
        blocks: &[GROOVE | ACID, GROOVE | ACID | STAB | FILL],
        moving: false,
        feeling: "hit and run / out before they know",
        color: "#b0663a",
    },
    SectionPlan {
        id: "tension",
        blocks: &[PAD | ARP, PAD | ARP | ROLL],
        moving: true,
        feeling: "armies face off / nobody moves",
        color: "#7a5aa6",
    },
    SectionPlan {
        id: "siege",
        blocks: &[
            KICK | CLAP | BASS,
            GROOVE | ACID,
            GROOVE | ACID | PAD,
            GROOVE | ACID | PAD | FILL,
        ],
        moving: true,
        feeling: "walls shaking / no way out",
        color: "#8f3f4a",
    },
    SectionPlan {
        id: "battle",
        blocks: &[
            GROOVE | STAB | ARP,
            GROOVE | STAB | ARP | ACID,
            GROOVE | STAB | ARP | GAP,
            GROOVE | STAB | ARP | ACID | FILL,
        ],
        moving: true,
        feeling: "everything committed",
        color: "#c2453a",
    },
    SectionPlan {
        id: "victory",
        blocks: &[HATS | PAD | ARP, KICK | HATS | PAD | ARP],
        moving: true,
        feeling: "the map is yours",
        color: "#d9b34a",
    },
    SectionPlan {
        id: "defeat",
        blocks: &[PAD, PAD | HATS],
        moving: true,
        feeling: "the lights go out",
        color: "#3a3a48",
    },
];

/// Trance lays pads under its stabs; techno trades pads under the kick for
/// the acid line.
fn styled(block: Block, style: StrategyStyle) -> Block {
    match style {
        StrategyStyle::Trance if block & STAB != 0 => block | PAD,
        StrategyStyle::Techno if block & KICK != 0 && block & PAD != 0 => (block & !PAD) | ACID,
        _ => block,
    }
}

fn write_section(
    plan_of: &SectionPlan,
    plan: &Plan,
    input: &StrategyInput,
    seed: u32,
) -> PortableSection {
    let id = plan_of.id;
    let bars = plan_of.blocks.len() as u32 * BLOCK_BARS;
    let progression = if id == "defeat" {
        DARK_PROGRESSION
    } else {
        plan.progression
    };
    let mut w = Writer::new(id);
    let drive = 0.8 + input.energy * 0.15;
    let bass_pitch = |offset: i32| 33 + (plan.tonic + offset).rem_euclid(12);
    for bar in 0..bars {
        let block = styled(plan_of.blocks[(bar / BLOCK_BARS) as usize], input.style);
        let has = |layer: Block| block & layer != 0;
        let last_of_block = bar % BLOCK_BARS == BLOCK_BARS - 1;
        let gap = has(GAP) && last_of_block;
        let chord_root = progression[(bar % 4) as usize];
        let chord = triad(chord_root);
        let root = if plan_of.moving { chord_root } else { 0 };
        if has(KICK) && !gap {
            for beat in 0..4 {
                w.hit("techno-kick", bar, beat * 4, drive);
            }
        }
        if has(HATS) {
            for beat in 0..4 {
                w.hit("open-hat", bar, beat * 4 + 2, 0.65);
            }
            for at in (0..16).filter(|at| at % 4 != 2) {
                // Ghost sixteenths come and go per bar with the seed.
                if at % 2 == 1 && keyed_unit(seed, "hat", bar, at) >= input.complexity {
                    continue;
                }
                w.hit("hat", bar, at, if at % 2 == 0 { 0.34 } else { 0.2 });
            }
        }
        let fill = has(FILL) && last_of_block;
        if has(CLAP) {
            w.hit("clap", bar, 4, 0.8);
            if !fill {
                w.hit("clap", bar, 12, 0.8);
            }
            if !fill && keyed_unit(seed, "clap", bar, 0) < input.syncopation * 0.4 {
                w.hit("clap", bar, 15, 0.45);
            }
        }
        if fill {
            for at in 12..16 {
                w.hit("clap", bar, at, 0.5 + f64::from(at - 12) * 0.12);
            }
        }
        if has(BASS) && !gap {
            let pitch = bass_pitch(root);
            for at in (0..16).filter(|at| at % 4 != 0) {
                let up = at % 4 == 2;
                w.note(
                    "bass",
                    "saw-bass",
                    bar,
                    at,
                    1,
                    pitch + if up { 12 } else { 0 },
                    if up { 0.9 } else { 0.55 },
                );
            }
        }
        if has(ACID) {
            let pitch = bass_pitch(root) + 12;
            for (at, step) in plan.acid.iter().enumerate() {
                if let Some((interval, accent)) = step {
                    w.note(
                        "acid",
                        "saw-bass",
                        bar,
                        at as u32,
                        1,
                        pitch + interval,
                        if *accent { 0.95 } else { 0.45 },
                    );
                }
            }
        }
        if has(STAB) {
            let voicing: Vec<i32> = triad(root)
                .iter()
                .chain(&[root + 10])
                .map(|interval| 57 + (plan.tonic + interval - 9).rem_euclid(12))
                .collect();
            let push = keyed_unit(seed, "push", bar, 0) < input.syncopation * 0.6;
            for &at in plan.stab.iter().chain(push.then_some(&15)) {
                for pitch in &voicing {
                    w.note("stab", "stab", bar, at, (16 - at).min(2), *pitch, 0.72);
                }
            }
        }
        if has(PAD) {
            for interval in chord {
                w.note(
                    "pad",
                    "trance-pad",
                    bar,
                    0,
                    16,
                    55 + (plan.tonic + interval - 7).rem_euclid(12),
                    0.55 + input.brightness * 0.2,
                );
            }
        }
        if has(ARP) {
            let tones: Vec<i32> = chord
                .iter()
                .map(|interval| 69 + (plan.tonic + interval - 9).rem_euclid(12))
                .collect();
            let level = if has(ROLL) {
                0.35 + f64::from(bar % BLOCK_BARS) / f64::from(BLOCK_BARS) * 0.55
            } else {
                0.9
            };
            for at in 0..16 {
                let index = plan.arp[(at % 4) as usize];
                let pitch = if index == 3 {
                    tones[0] + 12
                } else {
                    tones[index]
                };
                w.note(
                    "lead",
                    "trance-lead",
                    bar,
                    at,
                    1,
                    pitch,
                    level * (0.7 + input.brightness * 0.3),
                );
            }
        }
        if has(ROLL) && last_of_block {
            for at in 0..16 {
                w.hit("snare", bar, at, 0.3 + f64::from(at) * 0.04);
            }
            w.hit("reverse-cymbal", bar, 0, 0.8);
        }
    }
    let (_, label) = STRATEGY_PHASES
        .iter()
        .find(|(phase, _)| *phase == id)
        .unwrap();
    w.finish(label, plan_of.feeling, plan_of.color, bars)
}

/// Serialized selection rules. The game's `matchPhase` picks its section; a
/// `won` match plays the victory, a high `threat` forces the battle, and a
/// strong `economy` moves the build into the expansion.
pub fn default_rules() -> Vec<AdaptiveRule> {
    let rule = |target: &str,
                priority: i32,
                numeric: serde_json::Value,
                categorical: serde_json::Value| {
        AdaptiveRule {
            target: target.into(),
            priority,
            when: AdaptiveCondition {
                numeric,
                categorical,
            },
            hold: None,
        }
    };
    let mut rules = vec![
        rule(
            "victory",
            30,
            serde_json::json!({ "won": { "min": 1.0 } }),
            serde_json::json!({}),
        ),
        rule(
            "battle",
            20,
            serde_json::json!({ "threat": { "min": THREAT_BATTLE } }),
            serde_json::json!({}),
        ),
        rule(
            "expand",
            15,
            serde_json::json!({ "economy": { "min": ECONOMY_EXPAND } }),
            serde_json::json!({ "matchPhase": "build" }),
        ),
    ];
    rules.extend(STRATEGY_PHASES.iter().map(|(phase, _)| {
        rule(
            phase,
            10,
            serde_json::json!({}),
            serde_json::json!({ "matchPhase": phase }),
        )
    }));
    rules
}

pub fn generate_strategy(
    input: &StrategyInput,
    arrangement: StrategyArrangement,
) -> Result<PortableScore, String> {
    let seed = hash_text(&format!(
        "{}\0{}\0strategy-{GENERATOR_VERSION}",
        input.secret, input.seed
    ));
    let mut rng = DeterministicRandom::new(seed);
    let plan = Plan {
        tonic: *rng.pick(&[9, 7, 2, 4, 0, 5]),
        progression: *rng.pick(&PROGRESSIONS),
        stab: STABS[rng.integer(STABS.len() as u32) as usize],
        arp: *rng.pick(&ARPS),
        acid: std::array::from_fn(|step| {
            let rest = step % 4 != 0 && rng.next() < 0.3;
            (!rest).then(|| (*rng.pick(&[0, 0, 12, 3, 7, 10]), rng.next() < 0.3))
        }),
    };
    let energy = input.energy.clamp(0.0, 1.0);
    let bpm = match input.style {
        StrategyStyle::Techno => 126.0 + (energy * 6.0).round(),
        StrategyStyle::Trance => 132.0 + (energy * 6.0).round(),
    };
    let clean = StrategyInput {
        energy,
        brightness: input.brightness.clamp(0.0, 1.0),
        complexity: input.complexity.clamp(0.0, 1.0),
        syncopation: input.syncopation.clamp(0.0, 1.0),
        ..input.clone()
    };
    let sections: Vec<PortableSection> = SECTIONS
        .iter()
        .map(|section| write_section(section, &plan, &clean, seed ^ hash_text(section.id)))
        .collect();
    let form = (arrangement == StrategyArrangement::AllPhases).then(|| SongForm {
        steps: STRATEGY_PHASES
            .iter()
            .map(|(id, _)| SongFormStep {
                section: (*id).into(),
                repeats: 1,
            })
            .collect(),
        loop_from: Some(0),
        origin: None,
    });
    let score = PortableScore {
        schema_version: SCORE_SCHEMA_VERSION,
        id: format!(
            "strategy-generated-v{}-{seed:08x}",
            GENERATOR_VERSION.replace('.', "-")
        ),
        title: format!(
            "{} {} Minor",
            input.style.display(),
            NOTE_NAMES[plan.tonic as usize].to_uppercase()
        ),
        bpm,
        beats_per_bar: 4,
        ticks_per_beat: TICKS_PER_BEAT,
        crossfade_bars: 2.0,
        default_section: "build".into(),
        sections,
        rules: default_rules(),
        form,
    };
    score.validate()?;
    Ok(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(style: StrategyStyle, seed: &str) -> StrategyInput {
        StrategyInput {
            secret: String::new(),
            seed: seed.into(),
            style,
            energy: 0.5,
            brightness: 0.5,
            complexity: 0.5,
            syncopation: 0.5,
        }
    }

    #[test]
    fn generates_every_phase_as_a_long_valid_section_for_both_styles() {
        for style in [StrategyStyle::Techno, StrategyStyle::Trance] {
            let score =
                generate_strategy(&input(style, "m1"), StrategyArrangement::Original).unwrap();
            let ids: Vec<&str> = score.sections.iter().map(|s| s.id.as_str()).collect();
            let phases: Vec<&str> = STRATEGY_PHASES.iter().map(|(id, _)| *id).collect();
            assert_eq!(ids, phases);
            for section in &score.sections {
                assert!(section.length_ticks >= 16 * BAR, "{} is short", section.id);
            }
            assert!(score.sections.iter().all(|s| !s.events.is_empty()));
            assert!(score.form.is_none());
        }
    }

    #[test]
    fn deterministic_and_seeded() {
        let a = generate_strategy(
            &input(StrategyStyle::Techno, "m1"),
            StrategyArrangement::Original,
        )
        .unwrap();
        let b = generate_strategy(
            &input(StrategyStyle::Techno, "m1"),
            StrategyArrangement::Original,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        let distinct: std::collections::HashSet<String> = (0..20)
            .map(|i| {
                let s = generate_strategy(
                    &input(StrategyStyle::Trance, &format!("m{i}")),
                    StrategyArrangement::Original,
                )
                .unwrap();
                serde_json::to_string(&s.sections).unwrap()
            })
            .collect();
        assert!(distinct.len() > 10);
    }

    #[test]
    fn the_breakdown_drops_the_kick_and_the_battle_brings_it_back() {
        let score = generate_strategy(
            &input(StrategyStyle::Trance, "m1"),
            StrategyArrangement::AllPhases,
        )
        .unwrap();
        let kicks = |id: &str| {
            score
                .section(id)
                .unwrap()
                .events
                .iter()
                .filter(|e| e.voice() == "techno-kick")
                .count()
        };
        assert_eq!(kicks("tension"), 0);
        assert!(kicks("battle") > 0);
        assert_eq!(score.form.unwrap().steps.len(), STRATEGY_PHASES.len());
    }
}
