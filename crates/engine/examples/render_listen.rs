use gamestruments_engine::render::render_wav;
use gamestruments_engine::{
    generate_pocket_circuit, GenerateInput, InstrumentPalette, Style,
};
use std::fs;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let out = args.first().cloned().unwrap_or_else(|| "out.wav".into());
    let kind = args.get(1).cloned().unwrap_or_else(|| "menu".into());

    let (score, section, phrases) = match kind.as_str() {
        "menu" => (
            generate_pocket_circuit(&GenerateInput {
                secret: String::new(),
                seed: "level-004".into(),
                style: Style::Funk,
                palette: InstrumentPalette::default(),
                energy: 0.58,
                complexity: 0.75,
                brightness: 0.55,
                syncopation: 0.9,
            }),
            "grid",
            3,
        ),
        _ => (
            generate_pocket_circuit(&GenerateInput {
                secret: "guri-pc-dev-salt".into(),
                seed: args.get(2).cloned().unwrap_or_else(|| "race-12".into()),
                style: Style::Funk,
                palette: InstrumentPalette::default(),
                energy: 0.62,
                complexity: 0.6,
                brightness: 0.52,
                syncopation: 0.7,
            }),
            "cruise",
            4,
        ),
    };

    let wav = render_wav(&score, section, phrases, 22050);
    fs::write(&out, wav).expect("write wav");
    println!(
        "wrote {out} ({section} x{phrases}, bpm {})",
        score.bpm
    );
}
