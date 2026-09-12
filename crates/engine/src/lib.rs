pub mod adventure;
pub mod form_audio;
pub mod medieval;
pub mod racing;
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
pub use form_audio::FormAudio;
pub use medieval::{generate_medieval, MedievalInput, MedievalStyle};
pub use racing::{generate_racing, GenerateInput, InstrumentPalette, Style};
pub use render::render_wav;
pub use score::{AdventureState, GameState, MedievalState, PortableScore, TraceState};
pub use suspense::{generate_suspense, SuspenseInput, SuspenseStyle};
pub use suspense_arrangement::{generate_suspense_arrangement, SuspenseArrangement};
pub use synth::Synth;
pub use transport::AdaptiveTransport;
