//! Event collection shared by the composed recipes: stable per-lane ids,
//! swing applied to written straight sixteenths, and clipping to the section.

use std::collections::BTreeMap;

use crate::score::MusicEvent;

/// Collects a section's events with stable per-lane ids, applies the section's
/// swing to every onset, and keeps everything inside the section.
pub(crate) struct EventSink {
    section: &'static str,
    beat: u32,
    swing_point: f64,
    length: u32,
    pub(crate) events: Vec<MusicEvent>,
    counters: BTreeMap<&'static str, usize>,
}

impl EventSink {
    /// A sink for one section of `length` ticks. `swing_point` is where each
    /// beat's written midpoint is heard, as a fraction of the beat (0.5 plays
    /// straight, 2/3 is a triplet swing).
    pub(crate) fn new(section: &'static str, beat: u32, swing_point: f64, length: u32) -> Self {
        Self {
            section,
            beat,
            swing_point,
            length,
            events: Vec::new(),
            counters: BTreeMap::new(),
        }
    }

    fn swing(&self, tick: u32) -> u32 {
        let beat = f64::from(self.beat);
        let position = f64::from(tick % self.beat);
        let middle = beat / 2.0;
        let heard = self.swing_point * beat;
        let moved = if position <= middle {
            position * heard / middle
        } else {
            heard + (position - middle) * (beat - heard) / middle
        };
        tick - tick % self.beat + moved.round() as u32
    }

    /// Onset and length after the feel, clipped to the section.
    fn place(&self, start: u32, length: u32) -> Option<(u32, u32)> {
        let onset = self.swing(start);
        if onset >= self.length {
            return None;
        }
        let end = self.swing(start + length).min(self.length);
        Some((onset, end.saturating_sub(onset).max(1)))
    }

    fn next_id(&mut self, lane: &'static str) -> String {
        let counter = self.counters.entry(lane).or_insert(0);
        let id = format!("{}:{lane}:{counter}", self.section);
        *counter += 1;
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn note(
        &mut self,
        lane: &'static str,
        start: u32,
        length: u32,
        velocity: f64,
        pitch: i32,
        voice: &str,
        melody: bool,
    ) {
        let Some((start_tick, duration_ticks)) = self.place(start, length) else {
            return;
        };
        let pitch = u8::try_from(pitch).expect("note pitch must remain within MIDI range");
        let id = self.next_id(lane);
        self.events.push(MusicEvent::Note {
            id,
            section: self.section.to_string(),
            lane: lane.to_string(),
            start_tick,
            duration_ticks,
            velocity: velocity.clamp(0.04, 0.95),
            pitch,
            voice: voice.to_string(),
            role: melody.then(|| "melody".to_string()),
        });
    }

    pub(crate) fn hit(&mut self, start: u32, length: u32, velocity: f64, voice: &str) {
        let Some((start_tick, duration_ticks)) = self.place(start, length) else {
            return;
        };
        let id = self.next_id("percussion");
        self.events.push(MusicEvent::Percussion {
            id,
            section: self.section.to_string(),
            lane: "percussion".to_string(),
            start_tick,
            duration_ticks,
            velocity: velocity.clamp(0.04, 0.95),
            voice: voice.to_string(),
        });
    }

    pub(crate) fn finish(mut self) -> Vec<MusicEvent> {
        fn event_id(event: &MusicEvent) -> &str {
            match event {
                MusicEvent::Note { id, .. } | MusicEvent::Percussion { id, .. } => id,
            }
        }
        self.events.sort_by(|left, right| {
            left.start_tick()
                .cmp(&right.start_tick())
                .then_with(|| event_id(left).cmp(event_id(right)))
        });
        self.events
    }
}
