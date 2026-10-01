use crate::handoff::buffer_rms;
use crate::score::{MusicEvent, PortableScore};
use crate::synth::{Solo, Synth};
use crate::transport::AdaptiveTransport;

/// Identity of the incoming section the hold probe is watching: its tonal
/// synth index and the transition's start tick. The start tick changes when a
/// new plan begins, so comparing it scopes the probe to one plan and stops a
/// previous transition's tail from releasing a later hold.
type IncomingId = (usize, u32);

pub struct FormAudio {
    sample_rate: f32,
    frames: u64,
    last_tick: Option<u32>,
    tonal: Vec<Synth>,
    origins: Vec<Option<u32>>,
    active: [Option<(usize, f32, f32)>; 2],
    drums: Vec<Synth>,
    /// The incoming section's index and plan start tick while a section
    /// crossfade is active, plus the buffer of its rendered samples (tonal and
    /// percussion) used to gate the hold. The buffer is a pre-master render, so
    /// the transport gates it against [`crate::handoff::RAW_MUSICAL_FLOOR`].
    incoming: Option<IncomingId>,
    incoming_probe: Vec<f32>,
    solo: Solo,
}

impl FormAudio {
    pub fn new(score: &PortableScore, sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frames: 0,
            last_tick: None,
            tonal: score
                .sections
                .iter()
                .map(|_| Synth::new(sample_rate))
                .collect(),
            origins: vec![None; score.sections.len()],
            active: [None, None],
            drums: score
                .sections
                .iter()
                .map(|_| Synth::new(sample_rate))
                .collect(),
            incoming: None,
            incoming_probe: Vec::new(),
            solo: Solo::Full,
        }
    }

    /// Let only the voices `solo` names through, in every section.
    pub fn set_solo(&mut self, solo: &Solo) {
        self.solo = solo.clone();
        for synth in self.tonal.iter_mut().chain(self.drums.iter_mut()) {
            synth.set_solo(solo);
        }
    }

    /// A fresh synth for a section that restarts, under the current solo.
    fn synth(&self) -> Synth {
        let mut synth = Synth::new(self.sample_rate);
        synth.set_solo(&self.solo);
        synth
    }

    pub fn tick(&self, ticks_per_second: f64) -> u32 {
        (self.frames as f64 * ticks_per_second / f64::from(self.sample_rate)).floor() as u32
    }

    pub fn fill(
        &mut self,
        score: &PortableScore,
        transport: &mut AdaptiveTransport,
        buffer: &mut [f32],
    ) {
        let ticks_per_second = score.ticks_per_second();
        let mut report_tick = 0;
        for sample in buffer {
            let tick = self.tick(ticks_per_second);
            if self.last_tick != Some(tick) {
                transport.advance(tick);
                self.active = [None, None];
                let mut incoming = None;
                for (slot, playback) in transport.playback_at(tick).into_iter().enumerate() {
                    let Some(playback) = playback else {
                        continue;
                    };
                    let Some(index) = score
                        .sections
                        .iter()
                        .position(|section| section.id == playback.section)
                    else {
                        continue;
                    };
                    let section = &score.sections[index];
                    if self.origins[index] != Some(playback.origin) {
                        self.tonal[index] = self.synth();
                        self.drums[index] = self.synth();
                        self.origins[index] = Some(playback.origin);
                    }
                    let local = tick.saturating_sub(playback.origin) % section.length_ticks;
                    for event in section
                        .events
                        .iter()
                        .filter(|event| event.start_tick() == local)
                    {
                        match event {
                            MusicEvent::Note { .. } => {
                                self.tonal[index].trigger(event, ticks_per_second)
                            }
                            MusicEvent::Percussion { .. } if playback.percussion => {
                                self.drums[index].trigger(event, ticks_per_second)
                            }
                            _ => (),
                        }
                    }
                    self.active[slot] = Some((index, playback.gain, playback.drum_gain));
                    // The incoming section owns slot 1 during a crossfade; its
                    // start tick identifies the plan the probe belongs to.
                    if slot == 1 {
                        incoming = Some((index, playback.origin));
                    }
                }
                if self.incoming != incoming {
                    // A plan began or ended: drop the previous transition's
                    // samples so its tail cannot release this hold early.
                    self.incoming_probe.clear();
                }
                self.incoming = incoming;
                self.last_tick = Some(tick);
            }
            let mut mix = 0.0;
            for (index, gain, drum_gain) in self.active.into_iter().flatten() {
                let mut tonal = [0.0];
                self.tonal[index].fill(&mut tonal);
                let mut rhythm = [0.0];
                self.drums[index].fill(&mut rhythm);
                mix += tonal[0] * gain + rhythm[0] * drum_gain;
                // Accumulate the incoming section's own (unattenuated) tonal and drums,
                // only from the plan's start tick on, so the probe gates the
                // hold on the incoming that actually began.
                if let Some((probe_index, probe_start)) = self.incoming {
                    if probe_index == index && tick >= probe_start {
                        self.incoming_probe.push(tonal[0] + rhythm[0]);
                    }
                }
            }
            *sample = mix;
            self.frames += 1;
            report_tick = tick;
        }
        if self.incoming.is_some() && !self.incoming_probe.is_empty() {
            transport.report_incoming_level(buffer_rms(&self.incoming_probe), report_tick);
        }
        self.incoming_probe.clear();
    }

    /// Stereo fill: uses the engine stereo path (with pans, split twins, opposite echoes).
    /// All game elements currently summed to centre in this helper (L=R) to preserve
    /// existing game balance while allowing the Godot player to consume two channels.
    /// (Full per-voice panning in game context can be enabled by routing the L/R here.)
    pub fn fill_stereo(
        &mut self,
        score: &PortableScore,
        transport: &mut AdaptiveTransport,
        left: &mut [f32],
        right: &mut [f32],
    ) {
        let len = left.len();
        assert_eq!(len, right.len());
        let ticks_per_second = score.ticks_per_second();
        let mut report_tick = 0;
        for i in 0..len {
            let tick = self.tick(ticks_per_second);
            if self.last_tick != Some(tick) {
                transport.advance(tick);
                self.active = [None, None];
                let mut incoming = None;
                for (slot, playback) in transport.playback_at(tick).into_iter().enumerate() {
                    let Some(playback) = playback else {
                        continue;
                    };
                    let Some(index) = score
                        .sections
                        .iter()
                        .position(|section| section.id == playback.section)
                    else {
                        continue;
                    };
                    let section = &score.sections[index];
                    if self.origins[index] != Some(playback.origin) {
                        self.tonal[index] = self.synth();
                        self.drums[index] = self.synth();
                        self.origins[index] = Some(playback.origin);
                    }
                    let local = tick.saturating_sub(playback.origin) % section.length_ticks;
                    for event in section
                        .events
                        .iter()
                        .filter(|event| event.start_tick() == local)
                    {
                        match event {
                            MusicEvent::Note { .. } => {
                                self.tonal[index].trigger(event, ticks_per_second)
                            }
                            MusicEvent::Percussion { .. } if playback.percussion => {
                                self.drums[index].trigger(event, ticks_per_second)
                            }
                            _ => (),
                        }
                    }
                    self.active[slot] = Some((index, playback.gain, playback.drum_gain));
                    // The incoming section owns slot 1 during a crossfade; its
                    // start tick identifies the plan the probe belongs to.
                    if slot == 1 {
                        incoming = Some((index, playback.origin));
                    }
                }
                if self.incoming != incoming {
                    // A plan began or ended: drop the previous transition's
                    // samples so its tail cannot release this hold early.
                    self.incoming_probe.clear();
                }
                self.incoming = incoming;
                self.last_tick = Some(tick);
            }
            let mut mix_l = 0.0;
            let mut mix_r = 0.0;
            for (index, gain, drum_gain) in self.active.into_iter().flatten() {
                let mut tl = [0.0f32];
                let mut tr = [0.0f32];
                self.tonal[index].fill_stereo(&mut tl, &mut tr);
                let mut rl = [0.0f32];
                let mut rr = [0.0f32];
                self.drums[index].fill_stereo(&mut rl, &mut rr);
                mix_l += tl[0] * gain + rl[0] * drum_gain;
                mix_r += tr[0] * gain + rr[0] * drum_gain;
                // Accumulate the incoming section's own tonal and drums, only from the
                // plan's start tick on, so the probe gates the hold on the
                // incoming that actually began.
                if let Some((probe_index, probe_start)) = self.incoming {
                    if probe_index == index && tick >= probe_start {
                        self.incoming_probe
                            .push((tl[0] + tr[0]) * 0.5 + (rl[0] + rr[0]) * 0.5);
                    }
                }
            }
            left[i] = mix_l;
            right[i] = mix_r;
            self.frames += 1;
            report_tick = tick;
        }
        if self.incoming.is_some() && !self.incoming_probe.is_empty() {
            transport.report_incoming_level(buffer_rms(&self.incoming_probe), report_tick);
        }
        self.incoming_probe.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::{generate_adventure, AdventureInput, AdventureStyle};
    use crate::arrangement::{apply_automatic_arrangement, ArrangementRecipe};
    use crate::score::{AdventureState, MusicEvent, PortableSection, SCORE_SCHEMA_VERSION};
    use crate::{generate_suspense_arrangement, SuspenseArrangement, SuspenseInput, SuspenseStyle};

    /// A sustained note, so a section of these keeps rendering above silence.
    fn note(section: &str, start_tick: u32, serial: u32) -> MusicEvent {
        MusicEvent::Note {
            id: format!("{section}-{serial}"),
            section: section.into(),
            lane: "melody".into(),
            start_tick,
            duration_ticks: 240,
            velocity: 0.8,
            pitch: 60,
            voice: "chip".into(),
            role: Some("melody".into()),
        }
    }

    fn percussion(section: &str, start_tick: u32, serial: u32) -> MusicEvent {
        MusicEvent::Percussion {
            id: format!("{section}-{serial}"),
            section: section.into(),
            lane: "drums".into(),
            start_tick,
            duration_ticks: 240,
            velocity: 1.0,
            voice: "snare".into(),
        }
    }

    /// A two-section, form-less score. `a` is a steady groove from tick zero;
    /// `b` opens with `incoming_rest_ticks` of rest before the same groove. One
    /// bar is 960 ticks and the crossfade is one bar.
    fn cue_score(incoming_rest_ticks: u32) -> PortableScore {
        let section = |id: &str, first_note_tick: u32| PortableSection {
            id: id.into(),
            label: id.into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (first_note_tick..3840)
                .step_by(240)
                .enumerate()
                .map(|(i, tick)| note(id, tick, i as u32))
                .collect(),
        };
        PortableScore {
            schema_version: SCORE_SCHEMA_VERSION,
            id: "cue-test".into(),
            title: "Cue Test".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 240,
            crossfade_bars: 1.0,
            default_section: "a".into(),
            sections: vec![section("a", 0), section("b", incoming_rest_ticks)],
            rules: Vec::new(),
            form: None,
        }
    }

    /// A three-section, form-less score. `a` grooves from tick zero, `b` from
    /// `b_first_note`, and `c` from `c_first_note` (a tick past its length
    /// leaves `c` entirely silent). One bar is 960 ticks, crossfade one bar.
    fn cue_trio_score(b_first_note: u32, c_first_note: u32) -> PortableScore {
        let section = |id: &str, first_note_tick: u32| PortableSection {
            id: id.into(),
            label: id.into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (first_note_tick..3840)
                .step_by(240)
                .enumerate()
                .map(|(i, tick)| note(id, tick, i as u32))
                .collect(),
        };
        PortableScore {
            schema_version: SCORE_SCHEMA_VERSION,
            id: "cue-trio-test".into(),
            title: "Cue Trio Test".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 240,
            crossfade_bars: 1.0,
            default_section: "a".into(),
            sections: vec![
                section("a", 0),
                section("b", b_first_note),
                section("c", c_first_note),
            ],
            rules: Vec::new(),
            form: None,
        }
    }

    fn render(score: &PortableScore, transport: &mut AdaptiveTransport, bars: f64) -> f32 {
        let rate = 8000.0f32;
        let mut fa = FormAudio::new(score, rate);
        let mut buf = vec![0.0f32; 512];
        let frames =
            (score.bar_ticks() as f64 * bars / score.ticks_per_second() * f64::from(rate)).ceil()
                as usize;
        let mut min_rms = f32::INFINITY;
        let mut frame = 0;
        while frame < frames {
            let n = (frames - frame).min(512);
            fa.fill(score, transport, &mut buf[..n]);
            min_rms = min_rms.min(buffer_rms(&buf[..n]));
            frame += n;
        }
        min_rms
    }

    #[test]
    fn a_section_cue_whose_incoming_opens_with_rests_never_reaches_silence() {
        // `b` opens with one and a half bars of rest, longer than the one-bar
        // crossfade: the outgoing would fade out before the incoming sounds.
        let score = cue_score(1440);
        let mut transport = AdaptiveTransport::new(score.clone(), Some("a")).unwrap();
        transport.request_section("b", 0);

        // The hold keeps the outgoing at full gain until the incoming reaches
        // the floor, then the crossfade runs for another bar: two bars in all.
        let min_rms = render(&score, &mut transport, 2.0);
        assert!(
            min_rms > 0.001,
            "the transition dipped to silence: min chunk rms {min_rms}"
        );
    }

    #[test]
    fn a_section_cue_whose_incoming_has_content_crossfades_unchanged() {
        // `b` has content from tick zero, so the incoming never sits silent: the
        // cue must crossfade through without dipping and land on `b` within the
        // authored length.
        let score = cue_score(0);
        let mut transport = AdaptiveTransport::new(score.clone(), Some("a")).unwrap();
        let plan = transport.request_section("b", 0).unwrap();
        assert_eq!(plan.end_tick - plan.start_tick, score.bar_ticks());

        // Render one and a half authored lengths. A cue gated in the wrong
        // domain (the raw probe against the mastered floor) holds to the time
        // bound and only lands at two lengths; one whose incoming reaches the
        // floor on its first buffer completes at the authored length.
        let min_rms = render(&score, &mut transport, 1.5);
        assert!(
            min_rms > 0.001,
            "the content crossfade dipped to silence: min chunk rms {min_rms}"
        );
        assert_eq!(
            transport.current_section(),
            "b",
            "a content incoming must crossfade within the authored length, not hold to the time bound"
        );
    }

    #[test]
    fn a_later_cues_hold_is_not_released_by_the_previous_transitions_probe() {
        // `b` sounds from its second bar; `c` never sounds. Cueing `b` and then
        // queueing `c` puts the end of the first transition and the start of the
        // second in one fill: `b`'s loud tail must not release `c`'s hold.
        let score = cue_trio_score(960, 6000);
        let mut transport = AdaptiveTransport::new(score.clone(), Some("a")).unwrap();
        transport.request_section("b", 0).unwrap();
        transport.advance(500);
        assert!(
            transport.request_section("c", 500).is_none(),
            "a cue queued behind an active transition defers to it"
        );

        // `b` is silent through its authored window, so its cue releases at the
        // bound (tick 960) and ends at 1920, where the queued `c` cue begins.
        let _ = render(&score, &mut transport, 2.5);

        // `c` never sounds, so its hold must still be holding at tick 2400. A
        // probe that carried `b`'s tail would have released it at 1920 and be
        // halfway through `c`'s crossfade here.
        assert_eq!(
            transport.section_gain("c", 2400),
            0.0,
            "the queued cue's probe leaked from the previous transition"
        );
        assert_eq!(
            transport.section_gain("b", 2400),
            1.0,
            "the outgoing must stay at full gain while the incoming is silent"
        );
    }

    #[test]
    fn native_full_form_has_one_rhythm_owner_and_no_extra_opening_bars() {
        let score = generate_suspense_arrangement(
            &SuspenseInput {
                secret: "test".into(),
                seed: "flow-cycle".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            SuspenseArrangement::AllPhases,
        )
        .unwrap();
        let bar = score.bar_ticks();
        let form = score.form.as_ref().unwrap();
        let form_ticks: u32 = form
            .steps
            .iter()
            .map(|step| score.section(&step.section).unwrap().length_ticks * step.repeats)
            .sum();
        let mut transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut previous = "intro".to_string();
        let mut entered = 0;
        let mut first = true;
        let mut verse_entries = 0;
        // The pool's forms leave `origin` unset, so `advance` marks a section
        // entered at the end of its crossfade while the playback owner switches
        // at the crossfade start. Once the first (transition-less) opening is
        // past, each owner window is the authored length plus that transition —
        // never anything more.
        let transition = ((score.crossfade_bars * f64::from(bar)).round() as u32).max(bar);
        for tick in (0..3 * form_ticks + bar).step_by(480) {
            transport.advance(tick);
            let playback = transport.playback_at(tick);
            let owners: Vec<_> = playback
                .iter()
                .flatten()
                .filter(|part| part.percussion)
                .collect();
            assert_eq!(owners.len(), 1);
            let owner = owners[0];
            if owner.section != previous {
                let len = score.section(&previous).unwrap().length_ticks;
                let expected = if first { len } else { len + transition };
                assert_eq!(
                    tick - entered,
                    expected,
                    "boundary after {previous} took {} ticks",
                    tick - entered
                );
                first = false;
                entered = tick;
                previous = owner.section.to_string();
                if owner.section == "verse" {
                    verse_entries += 1;
                }
                if verse_entries == 3 {
                    break;
                }
            }
        }
        assert_eq!(verse_entries, 3);
    }

    #[test]
    fn form_audio_clock_and_samples_do_not_depend_on_buffer_size() {
        let score = generate_suspense_arrangement(
            &SuspenseInput {
                secret: "test".into(),
                seed: "flow".into(),
                style: SuspenseStyle::Terminal,
                tension: 0.62,
                heat: 0.48,
                mystery: 0.72,
                pulse: 0.55,
            },
            SuspenseArrangement::Seeded,
        )
        .unwrap();
        let frames =
            (score.bar_ticks() as f64 * 40.0 / score.ticks_per_second() * 8000.0).ceil() as usize;
        let mut one = FormAudio::new(&score, 8000.0);
        let mut chunked = FormAudio::new(&score, 8000.0);
        let mut first_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let mut second_transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        let opening = first_transport.current_section().to_string();
        let mut expected = vec![0.0; frames];
        let mut actual = vec![0.0; frames];
        one.fill(&score, &mut first_transport, &mut expected);
        for chunk in actual.chunks_mut(317) {
            chunked.fill(&score, &mut second_transport, chunk);
        }
        assert_eq!(actual, expected);
        assert!(actual.iter().any(|value| value.abs() > 0.02));
        assert_ne!(
            first_transport.current_section(),
            opening,
            "the form must advance past its opening section"
        );
        assert_eq!(
            chunked.tick(score.ticks_per_second()),
            one.tick(score.ticks_per_second())
        );
    }

    fn percussion_led_cue_score() -> PortableScore {
        let a = PortableSection {
            id: "a".into(),
            label: "a".into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (0..3840)
                .step_by(60)
                .enumerate()
                .map(|(i, tick)| note("a", tick, i as u32))
                .collect(),
        };
        let b = PortableSection {
            id: "b".into(),
            label: "b".into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (0..3840)
                .step_by(60)
                .enumerate()
                .map(|(i, tick)| percussion("b", tick, i as u32))
                .collect(),
        };
        PortableScore {
            schema_version: SCORE_SCHEMA_VERSION,
            id: "percussion-led-test".into(),
            title: "Percussion Led Test".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 240,
            crossfade_bars: 1.0,
            default_section: "a".into(),
            sections: vec![a, b],
            rules: Vec::new(),
            form: None,
        }
    }

    #[test]
    fn percussion_led_incoming_crossfades_and_releases() {
        let score = percussion_led_cue_score();
        let mut transport = AdaptiveTransport::new(score.clone(), Some("a")).unwrap();
        transport.request_section("b", 0).unwrap();
        let min_rms = render(&score, &mut transport, 1.5);
        assert!(
            min_rms > 0.0001,
            "the crossfade dipped to silence: min chunk rms {min_rms}"
        );
        assert_eq!(
            transport.current_section(),
            "b",
            "the hold must release for a percussion-led incoming"
        );
    }

    fn drum_bus_carry_score() -> PortableScore {
        let a = PortableSection {
            id: "a".into(),
            label: "a".into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (0..3840)
                .step_by(60)
                .enumerate()
                .map(|(i, tick)| percussion("a", tick, i as u32))
                .collect(),
        };
        let b = PortableSection {
            id: "b".into(),
            label: "b".into(),
            feeling: "steady".into(),
            color: "#ffffff".into(),
            length_ticks: 3840,
            events: (1440..3840)
                .step_by(60)
                .enumerate()
                .map(|(i, tick)| percussion("b", tick, i as u32))
                .collect(),
        };
        PortableScore {
            schema_version: SCORE_SCHEMA_VERSION,
            id: "drum-bus-carry-test".into(),
            title: "Drum Bus Carry Test".into(),
            bpm: 120.0,
            beats_per_bar: 4,
            ticks_per_beat: 240,
            crossfade_bars: 1.0,
            default_section: "a".into(),
            sections: vec![a, b],
            rules: Vec::new(),
            form: None,
        }
    }

    #[test]
    fn drum_bus_carries_outgoing_percussion_during_hold() {
        let score = drum_bus_carry_score();
        let mut transport = AdaptiveTransport::new(score.clone(), Some("a")).unwrap();
        transport.request_section("b", 0).unwrap();

        let rate = 8000.0f32;
        let mut fa = FormAudio::new(&score, rate);
        let mut buf = vec![0.0f32; 512];
        let frames = (score.bar_ticks() as f64 * 1.0 / score.ticks_per_second() * f64::from(rate)).ceil() as usize;
        let mut max_rms = 0.0f32;
        let mut frame = 0;
        while frame < frames {
            let n = (frames - frame).min(512);
            fa.fill(&score, &mut transport, &mut buf[..n]);
            max_rms = max_rms.max(buffer_rms(&buf[..n]));
            frame += n;
        }

        assert!(
            max_rms > crate::handoff::RAW_MUSICAL_FLOOR,
            "outgoing drums were cut during the hold: max chunk rms {max_rms}"
        );
    }

    #[test]
    fn quest_complete_supersedes_an_inflight_form_step_driven_by_the_renderer() {
        // Reproduce the macOS package-smoke failure end to end: `advance_form`
        // commits a camp -> explore step, the renderer reports the incoming
        // explore (which is when the player reports "explore" even though the
        // step is still holding), and `quest_complete` must then supersede the
        // stale step instead of queueing behind its hold and crossfade.
        let score = apply_automatic_arrangement(
            generate_adventure(&AdventureInput {
                secret: "transport-regression".into(),
                seed: "adventure-tour".into(),
                style: AdventureStyle::Folk,
                wonder: 0.6,
                danger: 0.5,
                mystery: 0.6,
                motion: 0.58,
            })
            .unwrap(),
            ArrangementRecipe::Adventure,
            true,
        )
        .unwrap();

        let mut transport = AdaptiveTransport::new(score.clone(), None).unwrap();
        assert_eq!(transport.current_section(), "camp");
        transport.advance_form(0).unwrap();

        let rate = 8000.0f32;
        let mut fa = FormAudio::new(&score, rate);
        let mut buf = vec![0.0f32; 512];
        let tps = score.ticks_per_second();
        let mut min_rms = f32::INFINITY;
        let mut cued = false;
        let mut saw_explore_reported = false;
        let mut saw_victory_reported = false;
        let mut victory_landed = false;
        for _ in 0..4000 {
            fa.fill(&score, &mut transport, &mut buf);
            min_rms = min_rms.min(buffer_rms(&buf));
            let tick = fa.tick(tps);
            let reported = transport
                .playback_at(tick)
                .into_iter()
                .flatten()
                .last()
                .map(|part| part.section.to_string());
            if reported.as_deref() == Some("explore") {
                saw_explore_reported = true;
                if !cued {
                    // `get_current_section` reports the incoming section the
                    // moment the step starts, so the smoke test sends
                    // quest_complete while camp is still the transport's section.
                    assert_eq!(
                        transport.current_section(),
                        "camp",
                        "explore is reported while camp is still the current section"
                    );
                    transport
                        .request_adventure_state(
                            &AdventureState {
                                area_phase: "explore".into(),
                                discovery: 0.4,
                                threat: 0.1,
                                quest_complete: true,
                            },
                            tick,
                        )
                        .expect("quest_complete must supersede the in-flight form step");
                    cued = true;
                }
            }
            if reported.as_deref() == Some("victory") {
                saw_victory_reported = true;
            }
            if transport.current_section() == "victory" {
                victory_landed = true;
                break;
            }
        }
        assert!(saw_explore_reported, "explore was reported before victory");
        assert!(cued, "quest_complete was sent while explore was reported");
        assert!(saw_victory_reported, "victory was reported");
        assert!(victory_landed, "victory landed as the current section");
        assert!(
            min_rms > 0.001,
            "the supersede dipped to silence: min chunk rms {min_rms}"
        );
    }
}
