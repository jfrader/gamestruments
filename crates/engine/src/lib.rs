pub mod pocket_circuit;
pub mod form_audio;
pub mod render;
pub mod rng;
pub mod score;
pub mod suspense;
pub mod suspense_arrangement;
pub mod synth;
pub mod transport;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use pocket_circuit::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};
pub use render::render_wav;
pub use score::{GameState, PortableScore, TraceState};
pub use suspense::{generate_suspense, SuspenseInput, SuspenseStyle};
pub use suspense_arrangement::{generate_suspense_arrangement, SuspenseArrangement};
pub use synth::Synth;
pub use form_audio::FormAudio;
pub use transport::AdaptiveTransport;
