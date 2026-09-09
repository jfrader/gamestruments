//! Native parity reference example.
//! Produces byte-for-byte reference artifacts that the WASM harness compares against.
//! Run: `cargo run -p gamestruments-engine --example parity_ref --release`
//!
//! Writes (in cwd):
//!   parity-funk-score.json / parity-funk-render.wav
//!   parity-chip-score.json / parity-chip-render.wav
//!
//! Also asserts that native generate is stable across two runs for the funk case.

use std::fs;

use gamestruments_engine::render::render_wav;
use gamestruments_engine::score::PortableScore;
use gamestruments_engine::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};

fn main() {
    println!("gamestruments-engine parity_ref: producing native references...");

    // Case 1: funk + default (empty) palette, specified traits
    let funk_json = r#"{"secret":"parity-secret","seed":"parity-001","style":"funk","palette":{"melody":"","harmony":"","drive":"","bass":""},"energy":0.62,"complexity":0.6,"brightness":0.52,"syncopation":0.7}"#;
    let (score_funk, wav_funk) = produce(funk_json, "cruise", 3);
    fs::write("parity-funk-score.json", &score_funk).expect("write funk score");
    fs::write("parity-funk-render.wav", &wav_funk).expect("write funk wav");
    println!(
        "wrote parity-funk-score.json + parity-funk-render.wav ({} bytes JSON, {} bytes WAV)",
        score_funk.len(),
        wav_funk.len()
    );

    // Case 2: chip + custom palette
    let chip_json = r#"{"secret":"parity-secret","seed":"parity-001","style":"chip","palette":{"melody":"chip","harmony":"chip","drive":"chip","bass":"triangle"},"energy":0.62,"complexity":0.6,"brightness":0.52,"syncopation":0.7}"#;
    let (score_chip, wav_chip) = produce(chip_json, "cruise", 3);
    fs::write("parity-chip-score.json", &score_chip).expect("write chip score");
    fs::write("parity-chip-render.wav", &wav_chip).expect("write chip wav");
    println!(
        "wrote parity-chip-score.json + parity-chip-render.wav ({} bytes JSON, {} bytes WAV)",
        score_chip.len(),
        wav_chip.len()
    );

    // Determinism: native must be stable (run twice, byte compare)
    let score1: PortableScore = serde_json::from_slice(&score_funk).unwrap();
    let score2 = generate_pocket_circuit(&GenerateInput {
        secret: "parity-secret".into(),
        seed: "parity-001".into(),
        style: Style::Funk,
        palette: InstrumentPalette::default(),
        energy: 0.62,
        complexity: 0.6,
        brightness: 0.52,
        syncopation: 0.7,
    })
    .expect("second native generation must validate");
    let j1 = serde_json::to_vec(&score1).unwrap();
    let j2 = serde_json::to_vec(&score2).unwrap();
    assert_eq!(j1, j2, "native score JSON must be identical across runs");
    println!("native determinism check: PASS (score JSON stable)");

    println!("parity_ref done.");
}

fn produce(input_json: &str, section: &str, phrases: usize) -> (Vec<u8>, Vec<u8>) {
    // mirror the WASM path: serde direct for input (here we parse for gen), then serialize, then parse the produced JSON and render
    #[derive(serde::Deserialize)]
    struct Pal {
        melody: String,
        harmony: String,
        drive: String,
        bass: String,
    }
    #[derive(serde::Deserialize)]
    struct Inp {
        secret: String,
        seed: String,
        style: String,
        palette: Pal,
        energy: f64,
        complexity: f64,
        brightness: f64,
        syncopation: f64,
    }
    let inp: Inp = serde_json::from_str(input_json).expect("parse input json in native ref");
    let style = Style::parse(&inp.style).expect("style");
    let palette = InstrumentPalette {
        melody: inp.palette.melody,
        harmony: inp.palette.harmony,
        drive: inp.palette.drive,
        bass: inp.palette.bass,
    };
    let gen = GenerateInput {
        secret: inp.secret,
        seed: inp.seed,
        style,
        palette,
        energy: inp.energy,
        complexity: inp.complexity,
        brightness: inp.brightness,
        syncopation: inp.syncopation,
    };

    let score = generate_pocket_circuit(&gen).expect("native parity score must validate");
    let score_bytes = serde_json::to_vec(&score).expect("serialize score");

    // parse the score JSON then render (exactly mirrors WASM render path)
    let score_parsed: PortableScore =
        serde_json::from_slice(&score_bytes).expect("reparse score json");
    let wav = render_wav(&score_parsed, section, phrases, 22050);
    (score_bytes, wav)
}
