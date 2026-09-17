//! Shared block-development arc for the composed arrangements.
//!
//! Suspense, Racing and Adventure all shape a phase so its layers enter and
//! leave across blocks instead of holding one texture for the whole length.
//! The schedule (which ranks are audible in each block), the masking rule, the
//! seeded jitter/density bias, the register fold and the seam/join planning
//! are shared here; each recipe supplies its own rank function, its own
//! role → arc mapping, and its own gesture vocabulary, so the shared idea is
//! not copy-pasted while each recipe keeps its own layer names.

use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{MusicEvent, PortableScore, PortableSection, SongForm};

/// The block size and sampled schedule for a development arc over `bars`.
///
/// Short phases get 2-bar blocks so they still have room for a shape; long ones
/// keep 4-bar blocks so the layers do not flutter. Returns `None` when the
/// phase is too short for a two-block shape.
pub(crate) fn development_schedule(arc: &[u8], bars: u32) -> Option<(u32, Vec<u8>)> {
    if bars == 0 {
        return None;
    }
    let block_bars = if bars <= 8 { 2 } else { 4 };
    let blocks = (bars / block_bars) as usize;
    if blocks < 2 {
        return None;
    }
    let schedule: Vec<u8> = (0..blocks)
        .map(|block| arc[(block * arc.len()) / blocks])
        .collect();
    Some((block_bars, schedule))
}

/// Mask a section's events to the block schedule: keep only layers whose rank
/// is at or below the block's scheduled rank. The closing bar is left intact so
/// the phase still resolves on the seam, and the bed (rank 0) sits below every
/// schedule so it is never masked. Seam gestures (`-seam` lanes) are transition
/// material, not a layer, so they are exempt from the mask and always survive.
pub(crate) fn mask_to_schedule(
    section: &mut PortableSection,
    bar: u32,
    block_ticks: u32,
    schedule: &[u8],
    rank: fn(&MusicEvent) -> u8,
) {
    let last_bar_start = section.length_ticks.saturating_sub(bar);
    section.events.retain(|event| {
        let start = event.start_tick();
        if start >= last_bar_start {
            return true;
        }
        if matches!(
            event,
            MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. }
                if lane.ends_with("-seam")
        ) {
            return true;
        }
        let block = (start / block_ticks) as usize;
        rank(event) <= schedule.get(block).copied().unwrap_or(4)
    });
}

/// Apply a seeded ±1 jitter to each block rank: the seeded density push lets a
/// take thin or fill a block, so density follows the seed rather than only the
/// role.
fn jitter_schedule(rng: &mut DeterministicRandom, schedule: &mut [u8]) {
    for rank in schedule {
        match rng.integer(3) {
            0 => *rank = rank.saturating_sub(1).max(1),
            2 => *rank = (*rank + 1).min(4),
            _ => {}
        }
    }
}

/// Apply a one-rank density bias to every block after the first. The bed
/// (rank 0) is never masked and the first block keeps its rank, so the melody
/// still waits for the build at every density.
fn bias_schedule(schedule: &mut [u8], bias: i32) {
    if bias == 0 {
        return;
    }
    for (index, rank) in schedule.iter_mut().enumerate() {
        if index == 0 {
            continue;
        }
        *rank = ((i32::from(*rank)) + bias).clamp(1, 4) as u8;
    }
}

/// The one-rank density bias for a 0..1 trait value: high keeps more layers
/// audible (later blocks climb a rank), low thins them.
pub(crate) fn density_bias(value: f64, high: f64, low: f64) -> i32 {
    if value >= high {
        1
    } else if value < low {
        -1
    } else {
        0
    }
}

/// Run the shared development arc for one section: derive the block schedule
/// from the role's `arc`, apply an optional seeded jitter and an optional
/// one-rank density bias, clamp every block to at least `floor`, then mask the
/// section's layers to the schedule using `rank`. Returns `(block_bars,
/// schedule)` so a recipe can add its own embellishments afterwards (Suspense's
/// impact onsets).
#[allow(clippy::too_many_arguments)]
pub(crate) fn develop_section(
    section: &mut PortableSection,
    bar: u32,
    arc: &[u8],
    rank: fn(&MusicEvent) -> u8,
    seed: u32,
    jitter: bool,
    bias: i32,
    floor: u8,
) -> Option<(u32, Vec<u8>)> {
    if bar == 0 {
        return None;
    }
    let bars = section.length_ticks / bar;
    let (block_bars, mut schedule) = development_schedule(arc, bars)?;
    if jitter {
        let mut rng = DeterministicRandom::new(seed ^ hash_text(&format!("{}:arc", section.id)));
        jitter_schedule(&mut rng, &mut schedule);
    }
    bias_schedule(&mut schedule, bias);
    for block_rank in &mut schedule {
        *block_rank = (*block_rank).max(floor);
    }
    let block_ticks = bar * block_bars;
    mask_to_schedule(section, bar, block_ticks, &schedule, rank);
    section.events.sort_by_key(MusicEvent::start_tick);
    Some((block_bars, schedule))
}

/// Fold piercing highs down whole octaves (pitch class, and hence the harmony,
/// preserved) so a composed run never squeals over the rest of the mix.
pub(crate) fn fold_register_ceiling(section: &mut PortableSection, ceiling: u8) {
    for event in &mut section.events {
        if let MusicEvent::Note { pitch, .. } = event {
            let mut value = i32::from(*pitch);
            while value > i32::from(ceiling) {
                value -= 12;
            }
            *pitch = u8::try_from(value.max(0)).unwrap_or(*pitch);
        }
    }
}

/// The lane a seam event lives on: `-seam` lanes are transition material, not a
/// layer, so they are exempt from the mask and always survive.
pub(crate) fn seam_lane(event: &MusicEvent) -> &str {
    match event {
        MusicEvent::Note { lane, .. } | MusicEvent::Percussion { lane, .. } => lane,
    }
}

/// Collect the `(out, in)` join pairs a song form tours, including the loop-back
/// from the last step to the loop point.
pub(crate) fn plan_joins(form: &SongForm) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = form
        .steps
        .windows(2)
        .map(|window| (window[0].section.clone(), window[1].section.clone()))
        .collect();
    if let (Some(loop_from), Some(last)) = (form.loop_from, form.steps.last()) {
        if let Some(first) = form.steps.get(loop_from as usize) {
            pairs.push((last.section.clone(), first.section.clone()));
        }
    }
    pairs
}

/// Remove every seam event from the score. A section can appear in more than
/// one join and the last planned treatment wins, so the previous treatment is
/// cleared before the pass re-plans it.
pub(crate) fn clear_seams(score: &mut PortableScore) {
    for section in &mut score.sections {
        section
            .events
            .retain(|event| !seam_lane(event).ends_with("-seam"));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_seam_note(
    events: &mut Vec<MusicEvent>,
    serial: &mut usize,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    pitch: u8,
    voice: &str,
) {
    let index = *serial;
    *serial += 1;
    events.push(MusicEvent::Note {
        id: format!("{section}:seam-note:{index}"),
        section: section.to_string(),
        lane: format!("{section}-seam"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.4),
        pitch,
        voice: voice.to_string(),
        role: None,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_seam_perc(
    events: &mut Vec<MusicEvent>,
    serial: &mut usize,
    section: &str,
    start: u32,
    duration: u32,
    velocity: f64,
    voice: &str,
) {
    let index = *serial;
    *serial += 1;
    events.push(MusicEvent::Percussion {
        id: format!("{section}:seam-perc:{index}"),
        section: section.to_string(),
        lane: format!("{section}-seam"),
        start_tick: start,
        duration_ticks: duration.max(1),
        velocity: velocity.clamp(0.08, 0.3),
        voice: voice.to_string(),
    });
}
