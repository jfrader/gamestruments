//! Shared block-development arc for the composed arrangements.
//!
//! Suspense and the Racing composed arrangement both shape a phase so its
//! layers enter and leave across blocks instead of holding one texture for the
//! whole length. The schedule (which ranks are audible in each block) and the
//! masking rule are shared here; each recipe supplies its own rank function and
//! its own role → arc mapping, so the shared idea is not copy-pasted while each
//! recipe keeps its own layer vocabulary.

use crate::score::{MusicEvent, PortableSection};

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
