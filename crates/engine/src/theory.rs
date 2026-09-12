//! Shared music-theory helpers used by the deterministic recipe generators.
//!
//! These are pure functions with no per-recipe state, so every generator that
//! maps scale degrees onto MIDI pitches can share one definition instead of
//! re-deriving the same scale maths.

/// Chromatic pitch-class names, index 0 = C.
pub const NOTE_NAMES: [&str; 12] = [
    "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
];

/// Church modes (and the minor alias) as semitone offsets from the root.
///
/// Unknown names fall back to dorian, the traditional modal default.
pub fn mode_intervals(mode: &str) -> Vec<i32> {
    match mode {
        "ionian" => vec![0, 2, 4, 5, 7, 9, 11],
        "dorian" => vec![0, 2, 3, 5, 7, 9, 10],
        "phrygian" => vec![0, 1, 3, 5, 7, 8, 10],
        "lydian" => vec![0, 2, 4, 6, 7, 9, 11],
        "mixolydian" => vec![0, 2, 4, 5, 7, 9, 10],
        "aeolian" | "natural-minor" => vec![0, 2, 3, 5, 7, 8, 10],
        _ => vec![0, 2, 3, 5, 7, 9, 10],
    }
}

/// Map a (possibly out-of-range) scale degree onto a MIDI pitch.
pub fn scale_pitch(root: i32, degree: i32, intervals: &[i32]) -> i32 {
    let len = intervals.len() as i32;
    let idx = degree.rem_euclid(len);
    let octave = (degree as f64 / len as f64).floor() as i32;
    root + octave * 12 + intervals[idx as usize]
}

/// MIDI pitch to a portable note token (e.g. 60 -> "c4").
pub fn midi_to_note(midi: i32) -> String {
    let pitch_class = midi.rem_euclid(12);
    let name = NOTE_NAMES[pitch_class as usize];
    let octave = (midi / 12) - 1;
    format!("{name}{octave}")
}

/// Shortest two-decimal representation of a finite float for score ids.
pub fn json_num(value: f64) -> String {
    let mut text = format!("{value:.2}");
    while text.ends_with('0') && text.contains('.') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text.is_empty() {
        "0".to_string()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::{midi_to_note, mode_intervals, scale_pitch};

    #[test]
    fn modes_have_seven_degrees() {
        for mode in [
            "ionian",
            "dorian",
            "phrygian",
            "lydian",
            "mixolydian",
            "aeolian",
        ] {
            assert_eq!(mode_intervals(mode).len(), 7);
        }
    }

    #[test]
    fn degree_mapping_wraps_octaves() {
        let dorian = mode_intervals("dorian");
        assert_eq!(scale_pitch(60, 0, &dorian), 60);
        assert_eq!(scale_pitch(60, 7, &dorian), 72);
        assert_eq!(scale_pitch(60, -1, &dorian), 58);
    }

    #[test]
    fn note_tokens_round_trip_octaves() {
        assert_eq!(midi_to_note(60), "c4");
        assert_eq!(midi_to_note(61), "c#4");
        assert_eq!(midi_to_note(59), "b3");
    }
}
