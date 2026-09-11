use crate::rng::{hash_text, DeterministicRandom};
use crate::score::{
    FormOrigin, MusicEvent, PortableScore, PortableSection, SongForm, SongFormStep,
};
use crate::suspense::{extended_trace_rules, generate_suspense, SuspenseInput, GENERATOR_VERSION};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SuspenseArrangement {
    #[default]
    Original,
    Extended,
    Theme,
}

impl SuspenseArrangement {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "" | "original" => Ok(Self::Original),
            "extended" => Ok(Self::Extended),
            "theme" => Ok(Self::Theme),
            other => Err(format!("Unknown suspense arrangement: {other}")),
        }
    }
}

pub fn generate_suspense_arrangement(
    input: &SuspenseInput,
    arrangement: SuspenseArrangement,
) -> Result<PortableScore, String> {
    match arrangement {
        SuspenseArrangement::Original => generate_suspense(input),
        SuspenseArrangement::Extended => generate_extended(input),
        SuspenseArrangement::Theme => generate_theme(input),
    }
}

fn generate_extended(input: &SuspenseInput) -> Result<PortableScore, String> {
    let mut score = generate_suspense(input)?;
    let root = score
        .section("intro")
        .and_then(|section| {
            section.events.iter().find_map(|event| match event {
                MusicEvent::Note { lane, pitch, .. } if lane == "intro-drone" => Some(*pitch + 12),
                _ => None,
            })
        })
        .ok_or("Suspense arrangement requires the opening drone")?;
    let bar = score.bar_ticks();
    let seed = hash_text(&format!(
        "{}\0{}\0suspense-atmosphere-2",
        input.secret, input.seed
    ));
    for section in &mut score.sections {
        if matches!(
            section.id.as_str(),
            "verse" | "verse-b" | "chorus" | "chorus-final" | "bridge" | "solo"
        ) {
            extend_section(section, bar);
            atmosphere(section, root, bar, seed);
        }
        rhythm(section, bar);
        percussion_details(section, bar, seed);
        if matches!(section.id.as_str(), "solo" | "bridge-b") {
            spotlight(section, root, bar, seed);
        }
        occasional_effect(section, bar, seed);
        if matches!(section.id.as_str(), "solo" | "bridge-b" | "chorus-final") {
            let cell_lane = format!("{}-cell", section.id);
            section.events.retain(|event| {
                !matches!(event,
                MusicEvent::Note { voice, lane, .. } if voice == "glass" && lane == &cell_lane)
            });
        }
    }
    let variations = [
        ("verse", "scan-ii", "Scan II"),
        ("chorus", "breach-ii", "Breach II"),
    ];
    for (base, id, label) in variations {
        let position = score
            .sections
            .iter()
            .position(|section| section.id == base)
            .ok_or("Missing base section")?;
        score
            .sections
            .insert(position + 1, variation(id, label, root, bar, seed));
    }
    let position = score
        .sections
        .iter()
        .position(|section| section.id == "chorus-final")
        .ok_or("Suspense requires a final section")?;
    score.sections.insert(position, anomaly(root, bar, seed));
    let form = score.form.as_mut().ok_or("Suspense requires a form")?;
    form.origin = Some(FormOrigin::TransitionStart);
    for (base, id, _) in variations {
        let position = form
            .steps
            .iter()
            .position(|step| step.section == base)
            .ok_or("Missing base form step")?;
        form.steps.insert(
            position + 1,
            SongFormStep {
                section: id.into(),
                repeats: 1,
            },
        );
    }
    let position = form
        .steps
        .iter()
        .position(|step| step.section == "chorus-final")
        .ok_or("Suspense requires a final form step")?;
    form.steps.insert(
        position,
        SongFormStep {
            section: "anomaly".into(),
            repeats: 1,
        },
    );
    score.rules = extended_trace_rules();
    score.id.push_str(&format!(
        "-extended-v{}",
        GENERATOR_VERSION.replace('.', "-")
    ));
    score.title.push_str(" — Extended");
    score.validate()?;
    Ok(score)
}

fn generate_theme(input: &SuspenseInput) -> Result<PortableScore, String> {
    let mut score = generate_suspense(input)?;
    let root = arrangement_root(&score)?;
    let bar = score.bar_ticks();
    let seed = hash_text(&format!(
        "{}\0{}\0suspense-theme-2",
        input.secret, input.seed
    ));
    let intro = score
        .section("intro")
        .cloned()
        .ok_or("Theme arrangement requires intro")?;
    densify_theme_intro(
        score
            .sections
            .iter_mut()
            .find(|section| section.id == "intro")
            .ok_or("Theme arrangement requires intro")?,
        bar,
    );
    for id in ["verse", "pre-chorus", "chorus", "chorus-final", "solo"] {
        let section = score
            .sections
            .iter_mut()
            .find(|section| section.id == id)
            .ok_or_else(|| format!("Theme arrangement requires {id}"))?;
        overlay_drone(section, &intro, bar);
        rhythm(section, bar);
        if id == "chorus-final" {
            theme_figure(section, root, bar, seed);
            theme_hat_break(section, bar);
            theme_drop_snare(section, bar);
        }
        if id == "solo" {
            theme_figure(section, root, bar, seed);
        }
    }
    if let Some(chorus) = score
        .sections
        .iter_mut()
        .find(|section| section.id == "chorus")
    {
        effect(
            chorus,
            "reverse-cymbal",
            chorus.length_ticks.saturating_sub(bar),
            bar,
            0.18,
        );
        chorus.events.sort_by_key(MusicEvent::start_tick);
    }
    score.form = Some(SongForm {
        steps: vec![
            theme_step("intro", 1),
            theme_step("verse", 1),
            theme_step("pre-chorus", 1),
            theme_step("chorus", 1),
            theme_step("chorus-final", 1),
            theme_step("solo", 2),
        ],
        loop_from: Some(4),
        origin: Some(FormOrigin::TransitionStart),
    });
    score.id.push_str("-theme");
    score.title.push_str(" — Theme");
    score.validate()?;
    Ok(score)
}

fn theme_step(section: &str, repeats: u32) -> SongFormStep {
    SongFormStep {
        section: section.into(),
        repeats,
    }
}

fn arrangement_root(score: &PortableScore) -> Result<u8, String> {
    score
        .section("intro")
        .and_then(|section| {
            section.events.iter().find_map(|event| match event {
                MusicEvent::Note { lane, pitch, .. } if lane == "intro-drone" => Some(*pitch + 12),
                _ => None,
            })
        })
        .ok_or_else(|| "Suspense arrangement requires the opening drone".into())
}

fn densify_theme_intro(section: &mut PortableSection, bar: u32) {
    let pulse = bar / 8;
    let bars = section.length_ticks / bar;
    for index in 0..bars {
        for step in [1u32, 3, 5, 7] {
            section.events.push(MusicEvent::Percussion {
                id: format!("intro:theme:hat:{index}:{step}"),
                section: "intro".into(),
                lane: "intro-kit".into(),
                start_tick: index * bar + step * pulse,
                duration_ticks: pulse / 3,
                velocity: 0.14,
                voice: "hat".into(),
            });
        }
        if index >= 2 {
            section.events.push(MusicEvent::Percussion {
                id: format!("intro:theme:kick:{index}"),
                section: "intro".into(),
                lane: "intro-kit".into(),
                start_tick: index * bar,
                duration_ticks: pulse,
                velocity: if index == 2 { 0.34 } else { 0.24 },
                voice: "kick".into(),
            });
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn theme_figure(section: &mut PortableSection, root: u8, bar: u32, seed: u32) {
    let mut random =
        DeterministicRandom::new(seed ^ hash_text(&format!("{}:theme-figure", section.id)));
    let late = section.id == "solo";
    let first_bar = if late { 1 } else { 0 };
    let answer_bar = if late { 5 } else { 4 };
    texture(
        section,
        Texture {
            lane: "ronroco",
            voice: "dusk",
            start: first_bar * bar + bar / 8,
            duration: bar + bar / 2,
            pitch: root,
            velocity: 0.2,
        },
    );
    texture(
        section,
        Texture {
            lane: "ronroco",
            voice: "felt",
            start: answer_bar * bar + 3 * (bar / 8),
            duration: bar / 2,
            pitch: root.saturating_add(*random.pick(&[3u8, 5])),
            velocity: 0.12,
        },
    );
}

fn theme_hat_break(section: &mut PortableSection, bar: u32) {
    let sixteenth = bar / 16;
    let bar_index = 6u32;
    for step in 0..16u32 {
        if step % 2 == 0 {
            continue;
        }
        section.events.push(MusicEvent::Percussion {
            id: format!("{}:theme:hat-break:{bar_index}:{step}", section.id),
            section: section.id.clone(),
            lane: format!("{}-kit", section.id),
            start_tick: bar_index * bar + step * sixteenth,
            duration_ticks: sixteenth / 2,
            velocity: if step == 15 { 0.2 } else { 0.11 },
            voice: "hat".into(),
        });
    }
}

fn theme_drop_snare(section: &mut PortableSection, bar: u32) {
    let pulse = bar / 8;
    section.events.push(MusicEvent::Percussion {
        id: format!("{}:theme:snare:land", section.id),
        section: section.id.clone(),
        lane: format!("{}-kit", section.id),
        start_tick: 0,
        duration_ticks: pulse,
        velocity: 0.46,
        voice: "snare".into(),
    });
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn overlay_drone(section: &mut PortableSection, intro: &PortableSection, bar: u32) {
    for event in &intro.events {
        let MusicEvent::Note {
            lane,
            velocity,
            pitch,
            voice,
            ..
        } = event
        else {
            continue;
        };
        if !lane.ends_with("-drone") {
            continue;
        }
        section.events.push(MusicEvent::Note {
            id: format!("{}:theme-drone:{lane}", section.id),
            section: section.id.clone(),
            lane: format!("{}-theme-drone", section.id),
            start_tick: 0,
            duration_ticks: section.length_ticks.saturating_sub(bar / 4).max(bar),
            velocity: *velocity * 0.85,
            pitch: *pitch,
            voice: voice.clone(),
            role: None,
        });
    }
}

fn anomaly_extension(section: &mut PortableSection, root: u8, bar: u32, seed: u32) {
    let start = section.length_ticks;
    let scanning = matches!(section.id.as_str(), "verse" | "scan-ii");
    let base = if scanning { "verse" } else { "chorus" };
    let variation = seed ^ hash_text(&format!("{base}:anomaly-extension"));
    let source = anomaly(root, bar, variation);
    for mut event in source.events {
        match &mut event {
            MusicEvent::Note {
                id,
                section: owner,
                lane,
                start_tick,
                velocity,
                voice,
                ..
            } => {
                if lane.ends_with("-fractured-pulse") {
                    if scanning && *start_tick < 4 * bar && *start_tick % bar < bar / 2 {
                        continue;
                    }
                    if scanning {
                        *voice = "felt".into();
                    }
                    *velocity *= if scanning { 0.65 } else { 0.9 };
                    *lane = format!("{}-extension-pulse", section.id);
                } else if lane.ends_with("-drone") {
                    *velocity = if scanning { 0.1 } else { 0.14 };
                    *lane = format!("{}-extension-drone", section.id);
                } else {
                    *velocity *= if scanning { 0.7 } else { 0.9 };
                    *lane = format!("{}-extension-echo", section.id);
                }
                *id = format!("{}:extension:{id}", section.id);
                *owner = section.id.clone();
                *start_tick += start;
            }
            MusicEvent::Percussion {
                id,
                section: owner,
                lane,
                start_tick,
                velocity,
                voice,
                ..
            } => {
                if voice != "kick" && voice != "hat" {
                    continue;
                }
                if *start_tick == 0 && voice == "kick" {
                    *velocity = 0.28;
                }
                *id = format!("{}:extension:{id}", section.id);
                *owner = section.id.clone();
                *lane = format!("{}-kit", section.id);
                *start_tick += start;
            }
        }
        section.events.push(event);
    }
    section.length_ticks += 8 * bar;
    if !scanning {
        effect(
            section,
            "reverse-cymbal",
            section.length_ticks - bar,
            bar,
            0.1,
        );
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn variation(id: &str, label: &str, root: u8, bar: u32, seed: u32) -> PortableSection {
    let scanning = id == "scan-ii";
    let mut section = PortableSection {
        id: id.into(),
        label: label.into(),
        feeling: if scanning {
            "staggered motion / quiet responses".into()
        } else {
            "fractured drive / return to the pulse".into()
        },
        color: if scanning {
            "#739fa8".into()
        } else {
            "#cf7c70".into()
        },
        length_ticks: 0,
        events: Vec::new(),
    };
    anomaly_extension(&mut section, root, bar, seed);
    section
        .events
        .retain(|event| event.voice() != "reverse-cymbal");
    for event in &mut section.events {
        if let MusicEvent::Note {
            lane,
            duration_ticks,
            ..
        } = event
        {
            if lane.ends_with("-drone") {
                *duration_ticks += 8 * bar;
            }
        }
    }
    section.length_ticks = 16 * bar;
    for phrase_bar in 8_u32..16 {
        let straight_return = !scanning && phrase_bar >= 14;
        let steps: &[u32] = if straight_return {
            &[0, 1, 2, 3, 4, 5, 6, 7]
        } else if phrase_bar.is_multiple_of(2) {
            &[0, 3, 6]
        } else {
            &[1, 4, 7]
        };
        for (index, step) in steps.iter().enumerate() {
            let interval = if straight_return {
                [0, 7][index % 2]
            } else {
                [7, 3, 0][(index + (phrase_bar / 2) as usize) % 3]
            };
            section.events.push(MusicEvent::Note {
                id: format!("{id}:response:{phrase_bar}:{step}"),
                section: id.into(),
                lane: format!("{id}-response-pulse"),
                start_tick: phrase_bar * bar + step * bar / 8,
                duration_ticks: bar / 16,
                pitch: root + interval,
                voice: if straight_return || phrase_bar.is_multiple_of(2) {
                    "pulse".into()
                } else {
                    "felt".into()
                },
                velocity: if scanning { 0.14 } else { 0.19 },
                role: None,
            });
        }
        if !phrase_bar.is_multiple_of(2) && !straight_return {
            texture(
                &mut section,
                Texture {
                    lane: "response-echo",
                    voice: "dusk",
                    start: phrase_bar * bar + bar / 4,
                    duration: bar / 2,
                    pitch: root + [3, 0, 7, 0][((phrase_bar - 9) / 2) as usize],
                    velocity: if scanning { 0.12 } else { 0.14 },
                },
            );
        }
    }
    rhythm(&mut section, bar);
    if !scanning {
        effect(&mut section, "reverse-cymbal", 15 * bar, bar, 0.1);
    }
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

fn extend_section(section: &mut PortableSection, bar: u32) {
    let length = section.length_ticks;
    let mut continuation = Vec::new();
    for event in &mut section.events {
        if let MusicEvent::Note {
            lane,
            duration_ticks,
            ..
        } = event
        {
            if lane.ends_with("-drone") {
                *duration_ticks += length;
                continue;
            }
        }
        let leave_space = matches!(event.start_tick() / bar, 1 | 5);
        let mut next = event.clone();
        match &mut next {
            MusicEvent::Note {
                id,
                lane,
                start_tick,
                velocity,
                ..
            } => {
                if leave_space && (lane.ends_with("-pulse") || lane.ends_with("-arp")) {
                    continue;
                }
                id.push_str(":development");
                *start_tick += length;
                if lane.ends_with("-arp") {
                    *velocity *= 0.8;
                }
            }
            MusicEvent::Percussion { id, start_tick, .. } => {
                id.push_str(":development");
                *start_tick += length;
            }
        }
        continuation.push(next);
    }
    section.length_ticks *= 2;
    section.events.extend(continuation);
}

fn rhythm(section: &mut PortableSection, bar: u32) {
    if section.id == "intro" {
        return;
    }
    section.events.retain(|event| !matches!(event, MusicEvent::Percussion { voice, .. } if voice == "kick" || voice == "hat"));
    if matches!(section.id.as_str(), "break" | "outro" | "coda") {
        return;
    }
    for index in 0..section.length_ticks / bar {
        for (voice, step, velocity) in [
            ("kick", 0, if index == 0 { 0.42 } else { 0.28 }),
            ("kick", 4, 0.22),
            ("hat", 1, 0.16),
            ("hat", 3, 0.16),
            ("hat", 5, 0.16),
            ("hat", 7, 0.16),
        ] {
            section.events.push(MusicEvent::Percussion {
                id: format!("{}:flow:{voice}:{index}:{step}", section.id),
                section: section.id.clone(),
                lane: format!("{}-kit", section.id),
                start_tick: index * bar + step * bar / 8,
                duration_ticks: if voice == "hat" { bar / 24 } else { bar / 8 },
                velocity,
                voice: voice.to_string(),
            });
        }
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

struct Texture {
    lane: &'static str,
    voice: &'static str,
    start: u32,
    duration: u32,
    pitch: u8,
    velocity: f64,
}

fn texture(section: &mut PortableSection, note: Texture) {
    if section.events.iter().any(|event| {
        event.is_melody()
            && event.start_tick() < note.start + note.duration
            && event.start_tick() + event.duration_ticks() > note.start
    }) {
        return;
    }
    section.events.push(MusicEvent::Note {
        id: format!("{}:{}:{}:{}", section.id, note.lane, note.voice, note.start),
        section: section.id.clone(),
        lane: format!("{}-{}", section.id, note.lane),
        start_tick: note.start,
        duration_ticks: note.duration,
        velocity: note.velocity,
        pitch: note.pitch,
        voice: note.voice.to_string(),
        role: Some("melody".into()),
    });
}

fn atmosphere(section: &mut PortableSection, root: u8, bar: u32, seed: u32) {
    let mut random = DeterministicRandom::new(seed ^ hash_text(&section.id));
    let beat = bar / 4;
    for (index, phrase_bar) in [5, 9, 13].into_iter().enumerate() {
        if index > 0 && random.integer(3) == 0 {
            continue;
        }
        let interval = *random.pick(&[0, 3, 7]);
        texture(
            section,
            Texture {
                lane: "atmosphere",
                voice: "felt",
                start: phrase_bar * bar + beat,
                duration: beat + beat / 2,
                pitch: root + interval,
                velocity: 0.2,
            },
        );
    }
    for phrase_bar in [7, 15] {
        texture(
            section,
            Texture {
                lane: "atmosphere",
                voice: "dusk",
                start: phrase_bar * bar + beat,
                duration: 2 * beat,
                pitch: root + 7,
                velocity: 0.16,
            },
        );
    }
}

fn spotlight(section: &mut PortableSection, root: u8, bar: u32, seed: u32) {
    let start = if section.id == "solo" {
        8 * bar
    } else {
        4 * bar
    };
    let end = start + 4 * bar;
    section.events.retain(|event| {
        !event.is_melody()
            || event.start_tick() >= end
            || event.start_tick() + event.duration_ticks() <= start
    });
    let mut random =
        DeterministicRandom::new(seed ^ hash_text(&format!("{}:spotlight", section.id)));
    let theme = *random.pick(&[[0, 3, 7, 5, 3, 0], [7, 5, 3, 0, 3, 0]]);
    let eighth = bar / 8;
    for (index, (phrase_bar, step, length)) in [
        (0, 0, 3),
        (0, 5, 2),
        (1, 2, 4),
        (2, 0, 3),
        (2, 5, 2),
        (3, 2, 4),
    ]
    .into_iter()
    .enumerate()
    {
        texture(
            section,
            Texture {
                lane: "spotlight",
                voice: "felt",
                start: start + phrase_bar * bar + step * eighth,
                duration: length * eighth,
                pitch: root + theme[index],
                velocity: if index % 3 == 0 { 0.28 } else { 0.24 },
            },
        );
    }
    for phrase_bar in [1, 3] {
        texture(
            section,
            Texture {
                lane: "spotlight-answer",
                voice: "dusk",
                start: start + phrase_bar * bar + 7 * eighth,
                duration: eighth,
                pitch: root + 7,
                velocity: 0.14,
            },
        );
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn effect(section: &mut PortableSection, voice: &str, start: u32, duration: u32, velocity: f64) {
    section.events.push(MusicEvent::Percussion {
        id: format!("{}:effect:{voice}:{start}", section.id),
        section: section.id.clone(),
        lane: format!("{}-effects", section.id),
        start_tick: start,
        duration_ticks: duration,
        voice: voice.to_string(),
        velocity,
    });
}

fn occasional_effect(section: &mut PortableSection, bar: u32, seed: u32) {
    if matches!(section.id.as_str(), "intro" | "break" | "outro" | "coda") {
        return;
    }
    let bars = section.length_ticks / bar;
    if bars < 5 {
        return;
    }
    let mut random = DeterministicRandom::new(seed ^ hash_text(&format!("{}:effects", section.id)));
    if random.integer(3) != 0 {
        return;
    }
    if random.integer(2) == 0 {
        effect(
            section,
            "reverse-cymbal",
            section.length_ticks - bar,
            bar,
            0.14,
        );
    } else {
        let position = 4 + random.integer(bars - 4);
        effect(section, "air-impact", position * bar, bar / 4, 0.14);
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn percussion_details(section: &mut PortableSection, bar: u32, seed: u32) {
    if matches!(section.id.as_str(), "intro" | "break" | "outro" | "coda") {
        return;
    }
    let mut random = DeterministicRandom::new(seed ^ hash_text(&format!("{}:details", section.id)));
    let mut count = 0;
    for index in (7..section.length_ticks / bar).step_by(4) {
        if count == 2 || random.integer(4) != 0 {
            continue;
        }
        let start = index * bar + 3 * bar / 4;
        if section.events.iter().any(|event| matches!(event, MusicEvent::Percussion { start_tick, .. } if *start_tick == start)) { continue; }
        section.events.push(MusicEvent::Percussion {
            id: format!("{}:detail:{index}", section.id),
            section: section.id.clone(),
            lane: format!("{}-details", section.id),
            start_tick: start,
            duration_ticks: bar / 8,
            voice: "tom".into(),
            velocity: 0.1 + f64::from(random.integer(3)) * 0.01,
        });
        count += 1;
    }
    section.events.sort_by_key(MusicEvent::start_tick);
}

fn anomaly(root: u8, bar: u32, seed: u32) -> PortableSection {
    let mut section = PortableSection {
        id: "anomaly".into(),
        label: "Anomaly".into(),
        feeling: "fractured echoes / steady heartbeat".into(),
        color: "#b093e8".into(),
        length_ticks: 8 * bar,
        events: Vec::new(),
    };
    section.events.push(MusicEvent::Note {
        id: "anomaly:drone".into(),
        section: "anomaly".into(),
        lane: "anomaly-drone".into(),
        start_tick: 0,
        duration_ticks: 8 * bar - bar / 4,
        velocity: 0.14,
        pitch: root - 12,
        voice: "warm".into(),
        role: None,
    });
    let mut random = DeterministicRandom::new(seed ^ hash_text("anomaly"));
    let displacement = random.integer(3);
    for phrase_bar in 0..8 {
        let stride = if matches!(phrase_bar, 4 | 5) { 5 } else { 3 };
        for step in (0..16_u32).step_by(stride) {
            let shifted = (step + displacement + phrase_bar % 3) % 16;
            let interval = [0, 7, 10, 7][((step / stride as u32 + phrase_bar) % 4) as usize];
            section.events.push(MusicEvent::Note {
                id: format!("anomaly:pulse:{phrase_bar}:{step}"),
                section: "anomaly".into(),
                lane: "anomaly-fractured-pulse".into(),
                start_tick: phrase_bar * bar + shifted * bar / 16,
                duration_ticks: bar / 16,
                velocity: if shifted.is_multiple_of(4) {
                    0.24
                } else {
                    0.18
                },
                pitch: root + interval,
                voice: if phrase_bar.is_multiple_of(2) {
                    "pulse".into()
                } else {
                    "felt".into()
                },
                role: None,
            });
        }
    }
    for (phrase_bar, interval) in [(1, 7), (3, 3), (5, 0), (7, 7)] {
        texture(
            &mut section,
            Texture {
                lane: "echo-cells",
                voice: "dusk",
                start: phrase_bar * bar + bar / 4,
                duration: bar / 2,
                pitch: root + interval,
                velocity: 0.16,
            },
        );
    }
    rhythm(&mut section, bar);
    effect(&mut section, "reverse-cymbal", 3 * bar, bar, 0.14);
    effect(&mut section, "air-impact", 4 * bar, bar / 4, 0.14);
    section.events.sort_by_key(MusicEvent::start_tick);
    section
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suspense::SuspenseStyle;

    fn input(seed: &str) -> SuspenseInput {
        SuspenseInput {
            secret: "test".into(),
            seed: seed.into(),
            style: SuspenseStyle::Terminal,
            tension: 0.62,
            heat: 0.48,
            mystery: 0.72,
            pulse: 0.55,
        }
    }

    #[test]
    fn theme_keeps_game_sections_but_plays_an_additive_form() {
        let original = generate_suspense(&input("theme-bed")).unwrap();
        let theme =
            generate_suspense_arrangement(&input("theme-bed"), SuspenseArrangement::Theme).unwrap();
        assert!(theme.section("break").is_some());
        assert_eq!(
            serde_json::to_value(original.section("break")).unwrap(),
            serde_json::to_value(theme.section("break")).unwrap()
        );
        let form = theme.form.as_ref().unwrap();
        let played: Vec<&str> = form
            .steps
            .iter()
            .map(|step| step.section.as_str())
            .collect();
        assert_eq!(
            played,
            [
                "intro",
                "verse",
                "pre-chorus",
                "chorus",
                "chorus-final",
                "solo"
            ]
        );
        assert!(!played.contains(&"break"));
        assert_eq!(form.loop_from, Some(4));
        let intro = theme.section("intro").unwrap();
        assert!(intro.events.iter().any(|event| {
            matches!(event, MusicEvent::Percussion { voice, .. } if voice == "hat")
        }));
        let verse = theme.section("verse").unwrap();
        assert!(verse.events.iter().any(|event| {
            matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-theme-drone"))
        }));
        assert!(theme.section("chorus-final").unwrap().events.len() >= verse.events.len());
        assert!(theme.section("chorus-final").unwrap().events.len() > intro.events.len());
        assert!(theme.section("chorus-final").unwrap().events.iter().any(
            |event| matches!(event, MusicEvent::Percussion { voice, .. } if voice == "snare")
        ));
        assert!(original.sections.iter().all(|section| {
            section.events.iter().all(
                |event| !matches!(event, MusicEvent::Percussion { voice, .. } if voice == "snare"),
            )
        }));
    }

    #[test]
    fn only_two_arrangements_and_original_is_unchanged() {
        assert!(SuspenseArrangement::parse("theme").is_ok());
        assert!(SuspenseArrangement::parse("flow").is_err());
        assert!(SuspenseArrangement::parse("featured").is_err());
        let input = input("level-001");
        assert_eq!(
            serde_json::to_value(generate_suspense(&input).unwrap()).unwrap(),
            serde_json::to_value(
                generate_suspense_arrangement(&input, SuspenseArrangement::Original).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn extended_has_no_featured_leads_and_keeps_the_entrance_and_full_grid() {
        let input = input("level-001");
        let original = generate_suspense(&input).unwrap();
        let score = generate_suspense_arrangement(&input, SuspenseArrangement::Extended).unwrap();
        assert_eq!(
            serde_json::to_value(score.section("intro")).unwrap(),
            serde_json::to_value(original.section("intro")).unwrap()
        );
        assert_eq!(
            score.form.as_ref().unwrap().origin,
            Some(FormOrigin::TransitionStart)
        );
        for section in &score.sections {
            for event in &section.events {
                if let MusicEvent::Note {
                    lane,
                    pitch,
                    velocity,
                    ..
                } = event
                {
                    assert!(!lane.contains("featured"));
                    if lane.ends_with("-atmosphere") {
                        assert!(*pitch <= 65);
                        assert!(*velocity <= 0.2);
                        assert!(event.start_tick() >= 5 * score.bar_ticks());
                        assert!(section.events.iter().filter(|other| other.is_melody()).all(
                            |other| {
                                std::ptr::eq(event, other)
                                    || other.start_tick()
                                        >= event.start_tick() + event.duration_ticks()
                                    || other.start_tick() + other.duration_ticks()
                                        <= event.start_tick()
                            }
                        ));
                    }
                }
            }
        }
        assert_eq!(
            score.section("verse").unwrap().length_ticks,
            16 * score.bar_ticks()
        );
    }

    #[test]
    fn effects_are_optional_and_spotlights_are_limited_to_two_sections() {
        let mut swells = 0;
        let mut impacts = 0;
        let mut absent = 0;
        for seed in 0..24 {
            let score = generate_suspense_arrangement(
                &input(&seed.to_string()),
                SuspenseArrangement::Extended,
            )
            .unwrap();
            for section in &score.sections {
                let effects: Vec<_> = section.events.iter().filter(|event| matches!(event, MusicEvent::Percussion { lane, .. } if lane.ends_with("-effects"))).collect();
                if section.id == "anomaly" {
                    assert_eq!(effects.len(), 2);
                } else {
                    assert!(effects.len() <= 1);
                }
                if effects.is_empty() {
                    absent += 1;
                }
                for event in effects {
                    assert!(event.velocity() <= 0.14);
                    if event.voice() == "reverse-cymbal" {
                        swells += 1;
                    }
                    if event.voice() == "air-impact" {
                        impacts += 1;
                    }
                }
                let spotlights: Vec<_> = section.events.iter().filter(|event| matches!(event, MusicEvent::Note { lane, .. } if lane.contains("spotlight"))).collect();
                if matches!(section.id.as_str(), "solo" | "bridge-b") {
                    assert_eq!(spotlights.len(), 8);
                    let start = if section.id == "solo" { 8 } else { 4 } * score.bar_ticks();
                    assert!(spotlights.iter().all(|event| event.start_tick() >= start
                        && event.start_tick() + event.duration_ticks()
                            <= start + 4 * score.bar_ticks()));
                } else {
                    assert!(spotlights.is_empty());
                }
            }
        }
        assert!(swells > 0 && impacts > 0 && absent > 0);
    }

    #[test]
    fn anomaly_is_one_bounded_detour_with_the_same_drum_grid() {
        let score = generate_suspense_arrangement(&input("anomaly"), SuspenseArrangement::Extended)
            .unwrap();
        let phase = score.section("anomaly").unwrap();
        let steps = &score.form.as_ref().unwrap().steps;
        let index = steps
            .iter()
            .position(|step| step.section == "anomaly")
            .unwrap();
        assert_eq!(steps[index - 1].section, "solo");
        assert_eq!(steps[index + 1].section, "chorus-final");
        assert_eq!(
            steps
                .iter()
                .filter(|step| step.section == "anomaly")
                .count(),
            1
        );
        assert_eq!(phase.length_ticks, 8 * score.bar_ticks());
        assert!(phase
            .events
            .iter()
            .any(|event| event.start_tick() % (score.bar_ticks() / 8) != 0));
        for bar in 0..8 {
            let kicks = phase
                .events
                .iter()
                .filter(|event| {
                    event.voice() == "kick" && event.start_tick() / score.bar_ticks() == bar
                })
                .count();
            let hats = phase
                .events
                .iter()
                .filter(|event| {
                    event.voice() == "hat" && event.start_tick() / score.bar_ticks() == bar
                })
                .count();
            assert_eq!((kicks, hats), (2, 4));
        }
    }

    #[test]
    fn extensions_preserve_the_existing_material_and_keep_the_new_groove_continuous() {
        let original = generate_suspense(&input("extensions")).unwrap();
        let bar = original.bar_ticks();
        for id in ["verse", "chorus"] {
            let mut section = original.section(id).unwrap().clone();
            let before = section.clone();
            let start = section.length_ticks;
            anomaly_extension(&mut section, 48, bar, 1234);
            assert_eq!(section.length_ticks, start + 8 * bar);
            let extension_pulse: Vec<_> = section
                .events
                .iter()
                .filter(|event| {
                    matches!(event,
                MusicEvent::Note { lane, .. } if lane.ends_with("extension-pulse"))
                })
                .collect();
            if id == "verse" {
                let early = extension_pulse
                    .iter()
                    .filter(|event| event.start_tick() < start + 4 * bar)
                    .count();
                assert!(
                    early < extension_pulse.len() - early,
                    "Scan should grow from sparse to fuller pulses"
                );
            } else {
                assert!(extension_pulse.iter().any(|event| event.voice() == "pulse"));
            }
            let prefix: Vec<_> = section
                .events
                .iter()
                .filter(|event| event.start_tick() < start)
                .collect();
            assert_eq!(
                serde_json::to_value(prefix).unwrap(),
                serde_json::to_value(&before.events).unwrap()
            );
            for index in 0..8 {
                let base = start + index * bar;
                let kicks: Vec<_> = section
                    .events
                    .iter()
                    .filter(|event| {
                        event.voice() == "kick"
                            && event.start_tick() >= base
                            && event.start_tick() < base + bar
                    })
                    .map(MusicEvent::start_tick)
                    .collect();
                let hats: Vec<_> = section
                    .events
                    .iter()
                    .filter(|event| {
                        event.voice() == "hat"
                            && event.start_tick() >= base
                            && event.start_tick() < base + bar
                    })
                    .map(MusicEvent::start_tick)
                    .collect();
                assert_eq!(kicks, [base, base + bar / 2]);
                assert_eq!(
                    hats,
                    [
                        base + bar / 8,
                        base + 3 * bar / 8,
                        base + 5 * bar / 8,
                        base + 7 * bar / 8
                    ]
                );
            }
            for event in section
                .events
                .iter()
                .filter(|event| event.start_tick() >= start)
            {
                assert_ne!(event.voice(), "glass");
                if let MusicEvent::Note { lane, velocity, .. } = event {
                    if lane.ends_with("extension-pulse") {
                        assert!(event.duration_ticks() <= bar / 16);
                        assert!(*velocity <= 0.216);
                        if id == "verse" {
                            assert_eq!(event.voice(), "felt");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn independent_variations_develop_instead_of_repeating_the_first_half() {
        let score =
            generate_suspense_arrangement(&input("pairs"), SuspenseArrangement::Extended).unwrap();
        let bar = score.bar_ticks();
        let steps = &score.form.as_ref().unwrap().steps;
        for (base, variant) in [("verse", "scan-ii"), ("chorus", "breach-ii")] {
            assert_eq!(score.section(base).unwrap().length_ticks, 16 * bar);
            let section = score.section(variant).unwrap();
            assert_eq!(section.length_ticks, 16 * bar);
            let index = steps.iter().position(|step| step.section == base).unwrap();
            assert_eq!(steps[index + 1].section, variant);
            let first: Vec<_> = section.events.iter().filter(|event| matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-extension-pulse"))).map(MusicEvent::start_tick).collect();
            let second: Vec<_> = section.events.iter().filter(|event| matches!(event, MusicEvent::Note { lane, .. } if lane.ends_with("-response-pulse"))).map(|event| event.start_tick() - 8 * bar).collect();
            assert!(!first.is_empty() && !second.is_empty());
            assert_ne!(first, second);
            assert!(section.events.iter().all(|event| event.voice() != "glass"));
        }
    }

    #[test]
    fn details_are_seeded_sparse_and_sometimes_absent_without_altering_the_beat() {
        let mut none = 0;
        let mut some = 0;
        let mut previous = None;
        for seed in 0..24 {
            let input = input(&seed.to_string());
            let score =
                generate_suspense_arrangement(&input, SuspenseArrangement::Extended).unwrap();
            assert_eq!(
                serde_json::to_value(&score).unwrap(),
                serde_json::to_value(
                    generate_suspense_arrangement(&input, SuspenseArrangement::Extended).unwrap()
                )
                .unwrap()
            );
            let scan = score.section("verse").unwrap();
            let details: Vec<_> = scan.events.iter().filter(|event| matches!(event, MusicEvent::Percussion { lane, .. } if lane.ends_with("-details"))).collect();
            assert!(details.len() <= 2);
            if details.is_empty() {
                none += 1;
            } else {
                some += 1;
            }
            let kick_hat: Vec<_> = scan
                .events
                .iter()
                .filter(|event| event.voice() == "kick" || event.voice() == "hat")
                .map(|event| {
                    (
                        event.start_tick(),
                        event.voice().to_string(),
                        event.velocity(),
                    )
                })
                .collect();
            if let Some(previous) = &previous {
                assert_eq!(previous, &kick_hat);
            }
            previous = Some(kick_hat);
        }
        assert!(none > 0 && some > 0);
    }
}
