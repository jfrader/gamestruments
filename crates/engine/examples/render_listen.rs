use gamestruments_engine::render::render_wav;
use gamestruments_engine::score::PortableScore;
use gamestruments_engine::{generate_racing, GenerateInput, InstrumentPalette, Style};
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dest = args
        .first()
        .cloned()
        .unwrap_or_else(|| "target/listen".into());
    let dest = Path::new(&dest);
    fs::create_dir_all(dest).expect("create dest dir");

    println!("Generating listening pack to {}", dest.display());

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    // 1. menu-grid: catalog level-004 funk grid x3, empty secret (reserved)
    {
        let json = include_str!("../../../catalog/racing/tiny-torque-level-004/score.json");
        let score: PortableScore = serde_json::from_str(json).expect("catalog json");
        let wav = render_wav(&score, "grid", 3, 48000);
        entries.push(("menu-grid.wav".into(), wav));
    }

    // 2. race-cruise x4 (phrases=4)
    {
        let score = generate_racing(&GenerateInput {
            secret: "guri-pc-dev-salt".into(),
            seed: "race-12".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.62,
            complexity: 0.6,
            brightness: 0.52,
            syncopation: 0.7,
        })
        .expect("race listening score must validate");
        let wav = render_wav(&score, "cruise", 4, 48000);
        entries.push(("race-cruise.wav".into(), wav));
    }

    // 3. one per style, seed "level-004", cruise x2, pocket secret
    let styles = [
        (Style::Fusion, "fusion"),
        (Style::Neon, "neon"),
        (Style::Funk, "funk"),
        (Style::Chip, "chip"),
    ];
    for (sty, name) in &styles {
        let score = generate_racing(&GenerateInput {
            secret: "guri-pc-dev-salt".into(),
            seed: "level-004".into(),
            style: *sty,
            palette: InstrumentPalette::default(),
            energy: 0.62,
            complexity: 0.6,
            brightness: 0.52,
            syncopation: 0.7,
        })
        .expect("style listening score must validate");
        let wav = render_wav(&score, "cruise", 2, 48000);
        entries.push((format!("style-{}-cruise-x2.wav", name), wav));
    }

    // 4. one per section for funk, x2, seed "race-12", pocket secret
    let sections = ["garage", "grid", "cruise", "attack", "final-lap", "victory"];
    for sec in &sections {
        let score = generate_racing(&GenerateInput {
            secret: "guri-pc-dev-salt".into(),
            seed: "race-12".into(),
            style: Style::Funk,
            palette: InstrumentPalette::default(),
            energy: 0.62,
            complexity: 0.6,
            brightness: 0.52,
            syncopation: 0.7,
        })
        .expect("section listening score must validate");
        let wav = render_wav(&score, sec, 2, 48000);
        entries.push((format!("section-{}-funk-x2.wav", sec), wav));
    }

    // Write WAVs + collect metadata
    let mut readme_lines = vec![
        "# Gamestruments Listening Pack (GURI-563)".to_string(),
        "".to_string(),
        "All renders @ 48000 Hz, mono 16-bit WAV from the Rust engine (after voice match audit).".to_string(),
        "Use for ear-QA against the Audio Lab sign-off.".to_string(),
        "".to_string(),
        "## Files and Checklist".to_string(),
        "".to_string(),
        "For each file, listen for:".to_string(),
        "- Voice identity per style (warm/glass/pulse/pluck/epiano/organ/supersaw/triangle/chip + kit)".to_string(),
        "- grid ↔ cruise groove swap in funk (different pocket feel, same voices)".to_string(),
        "- held pads / sustained layers in attack/final-lap/victory".to_string(),
        "- clean loop seam at phrase boundary (no click, smooth crossfade)".to_string(),
        "- balance of kit vs harmony (perc not overpowering, not buried)".to_string(),
        "- overall level consistency (no clipping, headroom)".to_string(),
        "".to_string(),
    ];

    for (fname, wav) in &entries {
        let out_path = dest.join(fname);
        fs::write(&out_path, wav).expect("write wav");
        let size = wav.len();
        // compute sha256 via shell (portable on linux)
        let sha = Command::new("sha256sum")
            .arg(&out_path)
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|s| s.split_whitespace().next().map(|h| h.to_string()))
            })
            .unwrap_or_else(|| "sha256-compute-failed".into());
        readme_lines.push(format!("- `{}` ({} bytes, sha256 `{}`)", fname, size, sha));
        readme_lines.push("  - [ ] voice identity clear (style-specific timbre)".to_string());
        if fname.contains("menu-grid")
            || fname.contains("section-grid")
            || fname.contains("section-cruise")
        {
            readme_lines.push("  - [ ] grid/cruise groove swap audible in funk family".to_string());
        }
        if fname.contains("attack") || fname.contains("final-lap") || fname.contains("victory") {
            readme_lines.push(
                "  - [ ] held pads / sustained harmony in attack/final-lap/victory".to_string(),
            );
        }
        readme_lines.push("  - [ ] loop seam clean at ~phrase length".to_string());
        readme_lines.push("  - [ ] kit vs harmony balance good".to_string());
        readme_lines.push("  - [ ] peak < 0.9, no distortion".to_string());
        readme_lines.push("".to_string());
        println!("wrote {} ({} bytes, sha256 {})", fname, size, sha);
    }

    // write README
    let readme_path = dest.join("README.md");
    fs::write(&readme_path, readme_lines.join("\n")).expect("write readme");
    println!("wrote {}", readme_path.display());
    println!("Pack complete. {} files.", entries.len());
}
