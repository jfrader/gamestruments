pub mod adventure;
pub mod arrangement;
pub mod form_audio;
pub mod racing;
pub mod racing_arrangement;
pub mod render;
pub mod rng;
pub mod score;
pub mod suspense;
pub mod suspense_arrangement;
pub mod synth;
pub mod theory;
pub mod transport;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use adventure::{generate_adventure, AdventureInput, AdventureStyle};
pub use arrangement::{apply_automatic_arrangement, ArrangementRecipe};
pub use form_audio::FormAudio;
pub use racing::{generate_racing, GenerateInput, InstrumentPalette, Style};
pub use racing_arrangement::{generate_racing_arrangement, RacingArrangement};
pub use render::render_wav;
pub use score::{AdventureState, GameState, PortableScore, TraceState};
pub use suspense::{generate_suspense, SuspenseInput, SuspenseStyle};
pub use suspense_arrangement::{generate_suspense_arrangement, SuspenseArrangement};
pub use synth::Synth;
pub use transport::AdaptiveTransport;
