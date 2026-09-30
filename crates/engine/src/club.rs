//! Club voicing for Suspense: techno and trance material written over the
//! song form Suspense already composed.
//!
//! The score keeps every Suspense section, length, label, form and rule; only
//! the events are rewritten. Each section is played in 8-bar blocks whose
//! layers (kick, hats, clap, bass, acid line, stab, pad, arpeggio) and ending
//! (fill, kick gap, snare roll) follow the phase's role and energy, so a break
//! becomes a breakdown and a peak becomes the drop. One seed fixes the key, the
//! progression, and the stab, arpeggio and acid figures for the whole piece.

use crate::match_phases::{
    block_at, match_phase, Block, ACID, ARP, BASS, BLOCK_BARS, CLAP, FILL, GAP, GROOVE, HATS, KICK,
    PAD, ROLL, STAB,
};
use crate::rng::{keyed_unit, DeterministicRandom};
use crate::score::{MusicEvent, PortableScore, PortableSection};
use crate::suspense_pool::{phase_spec, PhaseRole};
use crate::theory::NOTE_NAMES;

const SIXTEENTHS_PER_BAR: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClubStyle {
    Techno,
    Trance,
}

impl ClubStyle {
    fn name(self) -> &'static str {
        match self {
            Self::Techno => "Techno",
            Self::Trance => "Trance",
        }
    }

    /// Techno sits in the mid 120s, trance a little faster; energy pushes both.
    fn bpm(self, energy: f64) -> f64 {
        let base = match self {
            Self::Techno => 126.0,
            Self::Trance => 132.0,
        };
        base + (energy * 6.0).round()
    }
}

/// The four generation traits as the club writer reads them, each 0..1.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ClubTraits {
    /// Tempo and kick weight.
    pub(crate) energy: f64,
    /// How many ghost sixteenths the hats play.
    pub(crate) complexity: f64,
    /// How open the pad and lead sit.
    pub(crate) brightness: f64,
    /// Extra off-beat stabs and claps.
    pub(crate) syncopation: f64,
}

/// Minor-key progressions as semitones above the tonic: i–VI–III–VII,
/// i–VII–VI–VII and i–VI–iv–VII.
const PROGRESSIONS: [[i32; 4]; 3] = [[0, 8, 3, 10], [0, 10, 8, 10], [0, 8, 5, 10]];

/// Chord qualities follow the natural minor: i, iv minor; III, VI, VII major.
fn triad(root: i32) -> [i32; 3] {
    let minor = matches!(root.rem_euclid(12), 0 | 5);
    [root, root + if minor { 3 } else { 4 }, root + 7]
}

/// Stab rhythms (sixteenths in the bar) and arpeggio shapes (chord-tone
/// indices, 3 is the root an octave up).
const STABS: [&[u32]; 3] = [&[3, 6, 10], &[2, 7, 10, 14], &[3, 8, 11]];
const ARPS: [[usize; 4]; 3] = [[0, 1, 2, 1], [0, 2, 3, 2], [2, 1, 0, 1]];

/// What one seed fixes for the whole piece.
struct Plan {
    tonic: i32,
    progression: [i32; 4],
    stab: &'static [u32],
    arp: [usize; 4],
    /// The acid line: per sixteenth, a rest or an interval above the root
    /// with an accent that opens the filter.
    acid: [Option<(i32, bool)>; 16],
}

impl Plan {
    fn new(seed: u32) -> Self {
        let mut rng = DeterministicRandom::new(seed);
        Self {
            tonic: *rng.pick(&[9, 7, 2, 4, 0, 5]),
            progression: *rng.pick(&PROGRESSIONS),
            stab: STABS[rng.integer(STABS.len() as u32) as usize],
            arp: *rng.pick(&ARPS),
            acid: std::array::from_fn(|step| {
                let rest = step % 4 != 0 && rng.next() < 0.3;
                (!rest).then(|| (*rng.pick(&[0, 0, 12, 3, 7, 10]), rng.next() < 0.3))
            }),
        }
    }
}

/// The blocks a phase cycles through, by its role and energy.
fn blocks_for(role: PhaseRole, energy: u32) -> &'static [Block] {
    match role {
        PhaseRole::Intro => &[KICK | HATS, KICK | HATS | CLAP | FILL],
        PhaseRole::Groove if energy < 55 => &[GROOVE, GROOVE | STAB | FILL],
        PhaseRole::Groove => &[GROOVE | STAB, GROOVE | STAB | ARP | FILL],
        PhaseRole::Break => &[PAD | ARP, PAD | ARP | ROLL],
        PhaseRole::Bridge => &[KICK | HATS | ACID, GROOVE | ACID | GAP],
        PhaseRole::Build => &[HATS | ARP | PAD, KICK | HATS | ARP | PAD | ROLL],
        PhaseRole::Peak => &[
            GROOVE | STAB | ARP,
            GROOVE | STAB | ARP | ACID | GAP,
            GROOVE | STAB | ARP | ACID | FILL,
        ],
        PhaseRole::Loop => &[GROOVE | ACID, GROOVE | ACID | STAB | FILL],
        PhaseRole::Outro => &[HATS | PAD | ARP, PAD],
    }
}

/// Whether the bass follows the progression or holds the tonic.
fn moving(role: PhaseRole) -> bool {
    !matches!(role, PhaseRole::Intro | PhaseRole::Bridge | PhaseRole::Loop)
}

/// Trance lays pads under its stabs; techno trades pads under the kick for
/// the acid line.
fn styled(block: Block, style: ClubStyle) -> Block {
    match style {
        ClubStyle::Trance if block & STAB != 0 => block | PAD,
        ClubStyle::Techno if block & KICK != 0 && block & PAD != 0 => (block & !PAD) | ACID,
        _ => block,
    }
}

struct Writer<'a> {
    section: &'a str,
    sixteenth: u32,
    events: Vec<MusicEvent>,
    serial: usize,
}

impl Writer<'_> {
    fn start(&self, bar: u32, at: u32) -> u32 {
        (bar * SIXTEENTHS_PER_BAR + at) * self.sixteenth
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
            id: format!("{}:club-{lane}:{}", self.section, self.serial),
            section: self.section.into(),
            lane: lane.into(),
            start_tick: self.start(bar, at),
            duration_ticks: length * self.sixteenth,
            velocity: velocity.clamp(0.04, 0.95),
            pitch: u8::try_from(pitch).expect("club pitch stays in MIDI range"),
            voice: voice.into(),
            role: (lane == "lead").then(|| "melody".into()),
        });
    }

    fn hit(&mut self, voice: &str, bar: u32, at: u32, velocity: f64) {
        self.serial += 1;
        self.events.push(MusicEvent::Percussion {
            id: format!("{}:club-percussion:{}", self.section, self.serial),
            section: self.section.into(),
            lane: "percussion".into(),
            start_tick: self.start(bar, at),
            duration_ticks: self.sixteenth,
            velocity: velocity.clamp(0.04, 0.95),
            voice: voice.into(),
        });
    }
}

/// Rewrite every section of `score` as club music in `style`, and set the
/// tempo and title to match. Sections, labels, lengths, form and rules stay.
pub(crate) fn revoice(score: &mut PortableScore, style: ClubStyle, traits: ClubTraits, seed: u32) {
    let plan = Plan::new(seed);
    score.bpm = style.bpm(traits.energy);
    score.title = format!(
        "{} {} minor",
        style.name(),
        NOTE_NAMES[plan.tonic as usize].to_uppercase()
    );
    let sixteenth = score.ticks_per_beat / 4;
    for (index, section) in score.sections.iter_mut().enumerate() {
        section.events = write_section(
            section,
            &plan,
            style,
            traits,
            seed ^ (index as u32).wrapping_mul(0x9e37_79b9),
            sixteenth,
        );
    }
}

fn write_section(
    section: &PortableSection,
    plan: &Plan,
    style: ClubStyle,
    traits: ClubTraits,
    seed: u32,
    sixteenth: u32,
) -> Vec<MusicEvent> {
    let (role, energy) =
        phase_spec(&section.id).map_or((PhaseRole::Groove, 50), |spec| (spec.role, spec.energy));
    // A match phase plays its own block plan; every other phase follows its role.
    let (blocks, follow) = match_phase(&section.id).map_or_else(
        || (blocks_for(role, energy), moving(role)),
        |phase| (phase.blocks, phase.moving),
    );
    let bars = section.length_ticks / (SIXTEENTHS_PER_BAR * sixteenth);
    let drive = 0.8 + traits.energy * 0.15;
    let bass_pitch = |offset: i32| 33 + (plan.tonic + offset).rem_euclid(12);
    let mut w = Writer {
        section: &section.id,
        sixteenth,
        events: Vec::new(),
        serial: 0,
    };
    for bar in 0..bars {
        let (block, last_of_block) = block_at(blocks, bar, bars);
        let block = styled(block, style);
        let has = |layer: Block| block & layer != 0;
        let gap = has(GAP) && last_of_block;
        let fill = has(FILL) && last_of_block;
        let chord_root = plan.progression[(bar % 4) as usize];
        let chord = triad(chord_root);
        let root = if follow { chord_root } else { 0 };
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
                if at % 2 == 1 && keyed_unit(seed, "hat", bar, at) >= traits.complexity {
                    continue;
                }
                w.hit("hat", bar, at, if at % 2 == 0 { 0.34 } else { 0.2 });
            }
        }
        if has(CLAP) {
            w.hit("clap", bar, 4, 0.8);
            if !fill {
                w.hit("clap", bar, 12, 0.8);
                if keyed_unit(seed, "clap", bar, 0) < traits.syncopation * 0.4 {
                    w.hit("clap", bar, 15, 0.45);
                }
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
            let push = keyed_unit(seed, "push", bar, 0) < traits.syncopation * 0.6;
            for &at in plan.stab.iter().chain(push.then_some(&15)) {
                for pitch in &voicing {
                    w.note("stab", "stab", bar, at, (16 - at).min(2), *pitch, 0.72);
                }
            }
        }
        if has(PAD) {
            for interval in chord {
                let pitch = 55 + (plan.tonic + interval - 7).rem_euclid(12);
                w.note(
                    "pad",
                    "trance-pad",
                    bar,
                    0,
                    16,
                    pitch,
                    0.55 + traits.brightness * 0.2,
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
                    level * (0.7 + traits.brightness * 0.3),
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
    w.events.sort_by_key(MusicEvent::start_tick);
    w.events
}

#[cfg(test)]
mod tests {
    use crate::score::PortableScore;
    use crate::suspense::{generate_suspense, SuspenseInput, SuspenseStyle};
    use crate::suspense_arrangement::{generate_suspense_arrangement, SuspenseArrangement};
    use crate::suspense_pool::{phase_spec, PhaseRole};

    const CLUB_VOICES: [&str; 10] = [
        "techno-kick",
        "open-hat",
        "hat",
        "clap",
        "snare",
        "reverse-cymbal",
        "saw-bass",
        "stab",
        "trance-lead",
        "trance-pad",
    ];

    fn input(style: SuspenseStyle) -> SuspenseInput {
        SuspenseInput {
            secret: String::new(),
            seed: "club-set".into(),
            style,
            tension: 0.6,
            heat: 0.5,
            mystery: 0.6,
            pulse: 0.5,
        }
    }

    fn shape(score: &PortableScore) -> Vec<(String, u32)> {
        score
            .sections
            .iter()
            .map(|s| (s.id.clone(), s.length_ticks))
            .collect()
    }

    #[test]
    fn club_styles_keep_the_suspense_form_and_play_only_club_voices() {
        for arrangement in [SuspenseArrangement::AllPhases, SuspenseArrangement::Seeded] {
            let terminal =
                generate_suspense_arrangement(&input(SuspenseStyle::Terminal), arrangement)
                    .unwrap();
            for style in [SuspenseStyle::Techno, SuspenseStyle::Trance] {
                let club = generate_suspense_arrangement(&input(style), arrangement).unwrap();
                assert_eq!(
                    shape(&club),
                    shape(&terminal),
                    "{style:?} keeps the sections"
                );
                assert_eq!(
                    serde_json::to_string(&club.form).unwrap(),
                    serde_json::to_string(&terminal.form).unwrap()
                );
                for event in club.sections.iter().flat_map(|s| s.events.iter()) {
                    assert!(
                        CLUB_VOICES.contains(&event.voice()),
                        "{style:?} plays {}",
                        event.voice()
                    );
                }
                assert!(club.bpm >= 126.0, "{style:?} runs at club tempo");
            }
        }
    }

    #[test]
    fn every_style_plays_every_match_phase_at_the_same_length() {
        let lengths = |style: SuspenseStyle| {
            let score =
                generate_suspense_arrangement(&input(style), SuspenseArrangement::AllPhases)
                    .unwrap();
            crate::match_phases::MATCH_PHASES
                .iter()
                .map(|phase| {
                    let section = score
                        .section(phase.id)
                        .unwrap_or_else(|| panic!("{style:?} lacks {}", phase.id));
                    assert!(
                        !section.events.is_empty(),
                        "{style:?} {} is silent",
                        phase.id
                    );
                    section.length_ticks
                })
                .collect::<Vec<_>>()
        };
        let terminal = lengths(SuspenseStyle::Terminal);
        for style in [
            SuspenseStyle::Cipher,
            SuspenseStyle::Noir,
            SuspenseStyle::Techno,
            SuspenseStyle::Trance,
        ] {
            assert_eq!(lengths(style), terminal, "{style:?}");
        }
    }

    #[test]
    fn breaks_drop_the_kick_and_peaks_keep_it() {
        let score = generate_suspense_arrangement(
            &input(SuspenseStyle::Techno),
            SuspenseArrangement::AllPhases,
        )
        .unwrap();
        let kicks = |role: PhaseRole| {
            score
                .sections
                .iter()
                .filter(|s| phase_spec(&s.id).is_some_and(|spec| spec.role == role))
                .flat_map(|s| s.events.iter())
                .filter(|e| e.voice() == "techno-kick")
                .count()
        };
        assert_eq!(kicks(PhaseRole::Break), 0);
        assert!(kicks(PhaseRole::Peak) > 0);
    }

    #[test]
    fn deterministic_and_the_plain_generator_revoices_too() {
        let a = generate_suspense(&input(SuspenseStyle::Trance)).unwrap();
        let b = generate_suspense(&input(SuspenseStyle::Trance)).unwrap();
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert!(a.title.starts_with("Trance"));
        assert!(a
            .sections
            .iter()
            .flat_map(|s| s.events.iter())
            .any(|e| e.voice() == "trance-pad"));
    }
}
