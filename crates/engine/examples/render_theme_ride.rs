//! Render the Suspense Theme Ride phase (GURI-907) to a WAV for ear-QA.
//!
//! ```text
//! cargo run -p gamestruments-engine --example render_theme_ride -- /tmp/opencode/guri-907-theme-ride.wav
//! ```

use gamestruments_engine::render::render_wav;
use gamestruments_engine::suspense::SuspenseInput;
use gamestruments_engine::suspense::SuspenseStyle;
use gamestruments_engine::suspense_arrangement::generate_suspense_arrangement_take;
use gamestruments_engine::suspense_arrangement::SuspenseArrangement;
use gamestruments_engine::suspense_pool::Intent;
use std::fs;

fn main() {
    let dest = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/opencode/guri-907-theme-ride.wav".to_string());

    // The pool path is the only place `theme-ride` lives; `AllPhases` builds the
    // whole pool so the section exists to render.
    let score = generate_suspense_arrangement_take(
        &SuspenseInput {
            secret: "guri-907-audition".into(),
            seed: "theme-ride".into(),
            style: SuspenseStyle::Terminal,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.72,
            pulse: 0.55,
        },
        SuspenseArrangement::AllPhases,
        Intent::Arc,
        0,
    )
    .expect("audition score must validate");

    let section = score
        .section("theme-ride")
        .expect("theme-ride must be in the pool");
    let bars = section.length_ticks / score.bar_ticks();

    let wav = render_wav(&score, "theme-ride", 2, 48000);
    fs::write(&dest, &wav).expect("write wav");
    eprintln!(
        "wrote {dest}: theme-ride, {bars}-bar loop played twice (bars 1-{bars}, then 1-{bars} again; {} bars total), 48000 Hz mono 16-bit",
        bars * 2
    );
}
