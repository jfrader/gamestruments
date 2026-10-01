//! The match phases in the Suspense pool — a strategy game's arc from the
//! first build orders to the win or the loss — written once as 8-bar block
//! plans so every Suspense style plays the same shape in its own instruments.

/// Layers and endings of one 8-bar block, as bit flags.
pub(crate) type Block = u16;
pub(crate) const KICK: Block = 1;
pub(crate) const HATS: Block = 1 << 1;
pub(crate) const SNARE: Block = 1 << 2;
pub(crate) const BASS: Block = 1 << 3;
/// A running sixteenth-note line over the bass.
pub(crate) const RUN: Block = 1 << 4;
/// Off-beat hits on the cell voice.
pub(crate) const CELL: Block = 1 << 5;
pub(crate) const PAD: Block = 1 << 6;
pub(crate) const ARP: Block = 1 << 7;
/// A fill rolls through the block's last beat into the next.
pub(crate) const FILL: Block = 1 << 8;
/// The kick and bass drop out for the block's last bar.
pub(crate) const GAP: Block = 1 << 9;
/// A snare roll and a riser across the block's last bar.
pub(crate) const ROLL: Block = 1 << 10;
pub(crate) const GROOVE: Block = KICK | HATS | SNARE | BASS;
pub(crate) const BLOCK_BARS: u32 = 8;

/// One match phase: its blocks, whether the bass follows the progression,
/// and how it reads in the Lab.
pub(crate) struct MatchPhase {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) feeling: &'static str,
    pub(crate) color: &'static str,
    pub(crate) blocks: &'static [Block],
    pub(crate) moving: bool,
}

impl MatchPhase {
    pub(crate) fn bars(&self) -> u32 {
        self.blocks.len() as u32 * BLOCK_BARS
    }
}

pub(crate) const MATCH_PHASES: [MatchPhase; 10] = [
    MatchPhase {
        id: "build",
        label: "Build Order",
        feeling: "first orders / the machine starts",
        color: "#4d6b8a",
        blocks: &[
            KICK | HATS,
            KICK | HATS | SNARE,
            GROOVE,
            GROOVE | CELL | FILL,
        ],
        moving: false,
    },
    MatchPhase {
        id: "scout",
        label: "Recon",
        feeling: "eyes on the map / quiet feelers",
        color: "#4f7f8f",
        blocks: &[KICK | HATS | RUN, KICK | HATS | SNARE | RUN | GAP],
        moving: false,
    },
    MatchPhase {
        id: "expand",
        label: "Expansion",
        feeling: "new ground / the economy hums",
        color: "#3f8f7a",
        blocks: &[
            GROOVE,
            GROOVE | CELL,
            GROOVE | CELL | ARP,
            GROOVE | CELL | FILL,
        ],
        moving: true,
    },
    MatchPhase {
        id: "research",
        label: "Tech Up",
        feeling: "labs humming / something new",
        color: "#5a7fb0",
        blocks: &[HATS | ARP | PAD, KICK | HATS | ARP | PAD | ROLL],
        moving: true,
    },
    MatchPhase {
        id: "raid",
        label: "Raid",
        feeling: "hit and run / out before they know",
        color: "#b0663a",
        blocks: &[GROOVE | RUN, GROOVE | RUN | CELL | FILL],
        moving: false,
    },
    MatchPhase {
        id: "tension",
        label: "Standoff",
        feeling: "armies face off / nobody moves",
        color: "#7a5aa6",
        blocks: &[PAD | ARP, PAD | ARP | ROLL],
        moving: true,
    },
    MatchPhase {
        id: "siege",
        label: "Siege",
        feeling: "walls shaking / no way out",
        color: "#8f3f4a",
        blocks: &[
            KICK | SNARE | BASS,
            GROOVE | RUN,
            GROOVE | RUN | PAD,
            GROOVE | RUN | PAD | FILL,
        ],
        moving: true,
    },
    MatchPhase {
        id: "battle",
        label: "Battle",
        feeling: "everything committed",
        color: "#c2453a",
        blocks: &[
            GROOVE | CELL | ARP,
            GROOVE | CELL | ARP | RUN,
            GROOVE | CELL | ARP | GAP,
            GROOVE | CELL | ARP | RUN | FILL,
        ],
        moving: true,
    },
    MatchPhase {
        id: "victory",
        label: "Victory",
        feeling: "the map is yours",
        color: "#d9b34a",
        blocks: &[HATS | PAD | ARP, KICK | HATS | PAD | ARP],
        moving: true,
    },
    MatchPhase {
        id: "defeat",
        label: "Defeat",
        feeling: "the lights go out",
        color: "#3a3a48",
        blocks: &[PAD, PAD | HATS],
        moving: true,
    },
];

/// The block at `bar` of a phase that cycles `blocks`, and whether `bar` is
/// the last bar of its block within a section of `bars`.
pub(crate) fn block_at(blocks: &[Block], bar: u32, bars: u32) -> (Block, bool) {
    let index = bar / BLOCK_BARS;
    let end = ((index + 1) * BLOCK_BARS).min(bars);
    (blocks[index as usize % blocks.len()], bar + 1 == end)
}
