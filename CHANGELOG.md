# Changelog

## [Unreleased]

## [1.1.0] - 2026-10-01

### Added

- Suspense gains ten match phases for strategy games — Build Order, Recon, Expansion, Tech Up, Raid, Standoff, Siege, Battle, Victory and Defeat — written as 8-bar block plans that every style plays in its own instruments. The pool grows to 38 phases, so `seeded` and `all-phases` takes of Terminal, Cipher and Noir change for the same seed.
- Suspense gains a `trance` style: the same Suspense composition on club instruments. New voices: `techno-kick` and `clap` percussion, and `saw-bass`, `trance-pad` and `trance-lead` notes, which duck under every club kick.
- A Suspense style now changes only the instruments: every phase, including the pool phases that used to keep Terminal's voices in Cipher and Noir, plays in the style's sound world, and the notes are the same in every style. Cipher and Noir takes change for the same seed; Terminal is unchanged.
- The Audio Lab plays the engine itself: the same live player the Godot addon runs, so the Lab sounds like the kit (mono, no browser room or stereo stage). Cues, game signals, Audition and new versions that join on the bar all go through the engine. The browser synth is gone.
- The Rust engine's live player can audition part of a mix (`LivePlayer::set_solo`, behind the Audio Lab's Audition buttons): the melody, everything but it, or one instrument alone or muted, fading over 18 ms. `GamestrumentsPlayer` does not expose it yet.
- Calling `generate()` on a Godot player that is already playing crossfades the new score over the one still going, for the same two bars a section join uses. The old score is not cut off, and cues during the fade land on the incoming score.

### Changed

- Racing's generator is now `1.11.0`. The garage intro and the Ignition build were re-voiced (GURI-1240): garage's held chord and bass now sound through the bar and the phase gains a light offbeat pulse, and Ignition ramps across its eight bars instead of replaying the garage at reduced level. Grid, Cruise, Attack, Final Lap, Victory, Slipstream, Redline and Cooldown are unchanged. Non-reserved generated takes get new ids; the shipped `tiny-torque-level-004` catalog fixture was regenerated, and the frozen Original-material regression digests were re-baselined for garage alone.

### Fixed

- A Godot player no longer glitches as a section blend finishes or a held section loops. The renderer restarted the section that was still sounding, cutting its ringing notes and jumping its phrase back by the blend length. A sounding section now keeps playing from where it began fading in, as the Audio Lab's browser synth always did. Song-form timing is unchanged.
- The Audio Lab uses far less CPU. Every animation frame rewrote all its section rows, even while stopped, and re-blurred the orbit's glow. It now writes only what changed, and the glow is a static layer. In a desktop Chromium window, playing Suspense Trance dropped from about 1.4 cores to about a quarter of one, and a stopped Lab from about 1.2 cores to about 6% of one.
- `kit/docs/limitations.md` shows how to quit while the music plays without Godot's `ObjectDB instance was leaked at exit` warning. Godot 4.7 prints it for any audio still playing at quit ([godot#76745](https://github.com/godotengine/godot/issues/76745)). The kit contract no longer promises that removing the player alone avoids it.
- A Godot game can now change seed and choose the incoming section in one `generate(seed, opening_section)` call. The new score waits for the same bar-aligned handoff as before but starts directly on that section, instead of fading into its default section and then stacking a second phase transition on top.
- The garage intro no longer falls into near-silence at the end of every bar: its held chord stopped at 69% of the bar and the Funk style dropped its only hat, so the sparse kit could not cover the gap. Ignition inherited the same hole and, opening at reduced level, read as a dropout rather than a build.
- A seed change no longer dips the mix toward silence during the crossfade: the outgoing holds at full level until the incoming score's level clears a musical floor (its RMS, not its first non-zero sample), then crossfades with the incoming gain-matched to the outgoing so the summed level stays even.
- A section cue (`cue_section`) no longer drops the summed Master bus to silence: the outgoing section holds at full level until the incoming section's rendered level clears the musical floor, then runs the crossfade, so a section that opens with rests no longer leaves a hole after the cue.
- A section cue whose incoming is carried by drums now releases on time (the hold gate reads the incoming's full render, tonal plus percussion, not just its tonal), and the drum bus crossfades with the section instead of hard-switching the pattern at the cue start.
- Offline WAV renders no longer compress the score or drop its tail: the render advanced its tick cursor by `ceil(chunk / sample_rate * tps) + 1` per internal chunk — more ticks than the chunk's samples actually covered — so long exports drifted ahead of the audio and each phrase's ending came back as the next phrase's opening. The cursor is now derived from the produced-sample count and every event is scheduled at its true sample offset, so exported and chunked WAVs keep their timing and their full tail.
- A game-state change (`set_race_state`, `set_trace_state`, `set_adventure_state`) now supersedes the song-form transition already in flight instead of waiting for it to finish: a `quest_complete` that lands while the form is stepping still reaches the victory section promptly, rather than only after the outgoing section's hold and crossfade.

## [1.0.4] - 2026-09-21

### Fixed

- Live playback costs another ~32% less CPU on the production render path (synth-only ~43%): each voice caches its velocity gain and oscillator detune multipliers at trigger time instead of recomputing three invariant `powf` calls per sample. Output is bit-exact.

## [1.0.3] - 2026-09-21

### Added

- Godot `GamestrumentsPlayer` now exposes `sample_rate` (default `48000`). Lower it — e.g. `22050` — when a game's material is band-limited, to cut live synthesis CPU; it is read when the node enters the tree and when a score is generated, so set it before `add_child`/`generate`.

### Fixed

- Live playback uses substantially less CPU: realtime mastering no longer runs two expensive loudness/true-peak meters when no report is requested, while the limiter still enforces its true-peak ceiling. Synth filters also reuse unchanged coefficients instead of recalculating trigonometry for every sample.

## [1.0.2] - 2026-09-19

### Fixed

- Godot `GamestrumentsPlayer` no longer drifts out of time. The score clock is now derived from produced samples rather than the render frame, and every note is scheduled at its true sample offset inside the buffer, so timing is independent of frame rate. Previously the score position crept 15-35 s ahead of the sound after ~9 minutes of playback, and a stalled frame could fire a whole window of notes at one instant. The generated buffer is also handed to Godot with a single `push_buffer` call instead of one `push_frame` per sample, removing ~48k GDExtension crossings per second and cutting live playback CPU substantially.
- Audio Lab: the engine button reserves a fixed width, so toggling Play/Stop — and the label swapping between Start engine, the current phase, and a crossover readout — no longer reflows the topbar into a second row; a long phase name truncates with an ellipsis inside the fixed control.
- Audio Lab: removed the eyebrow text above the title so the wordmark no longer overlaps it on narrow screens.

## [1.0.1] - 2026-09-17

### Added

- Suspense's phase pool regained Theme Ride: a full 4/4 rock-backbeat ride (kick on beats 1 and 3, snare on 2 and 4, offbeat hats, every bar) that carries the Decrypt solo's high-register cell and the Full Breach late hook over the shared harmonic arc. The backbeat is pinned deterministically for this phase alone, so every other phase's seeded kit choice is unchanged.
- Audio Lab: added a Score Debugger view — one step grid per bar (8th or 16th subdivision, inferred from the events) with hit counts, first/last tick and velocity range per voice, plus Solo/Mute per voice and a Play bar action that holds the transport on that bar.

### Changed

- Suspense's `seeded` and `all-phases` takes change in this release: the pool gained Theme Ride (28 phases), so the composer draws a different form and a take can now include the ride. A same-seed take is deterministic within 1.0.1 but is not the song 1.0.0 produced. Racing and Adventure are unchanged.

## [1.0.0] - 2026-09-17

### Added

- Racing now has a Defeat outro: a dark, resolved-down version of the Finish material cued on a loss or DNF, while Victory still plays on a win. The composed arrangements also carry a Recovery incident (drums fall away plus a single impact, cued on a reset — never a plain crash) and its Wrong-Way tense variant, both short phases that resolve back into the flow. Original and Extended stay byte-identical and keep Victory as their only finish.
- Racing's Seeded and All phases arrangements now share one eleven-phase pool: the six base sections, the four authored phases (Ignition, Slipstream, Redline, Cooldown), and the drumless Breather. Cooldown resolves as a post-outro release — a distinct terminal from Victory, so the two no longer compete for one outro slot — and the composer's step band widened to 6–12 so a composed song can reach the whole pool. All phases tours every pool phase; Seeded composes over them. Original and Extended stay byte-identical.
- Adventure gained six new pool phases — a combat set (Skirmish, Assault, Chase) and a happiness set (Festival, Reunion, Dawn) — each with its own mode, register, and percussion density. The combat phases share the Peak role with combat/boss and the happiness phases ride the groove and break bands with town/sanctuary, so the seeded composer interleaves them with their matching scenes instead of the quest always falling back on the same eight sections. State-driven selection still returns only the eight original sections.
- Racing Seeded now has a dedicated Breather phase — a drumless Race Flow passage (no kit) the composer can place before a flow phase, so a drumless moment exists on purpose instead of as an accident inside a groove.
- Racing now has a Seeded arrangement: an opt-in song form over the six Racing sections, chosen by a seeded composer from role and energy metadata derived from the authored plans. The form always opens in the garage, closes on victory, and loops from a groove; role-aware lengths keep the intro and outro at four bars while peaks stretch to eight or twelve. Original and Extended remain byte-identical.
- Adventure now has a Seeded arrangement: an opt-in song form over the eight Adventure sections, chosen by a seeded composer from role and energy metadata derived from the authored scene plans. The form always opens at camp, closes on victory, and loops from a groove; role-aware lengths keep the intro, break, and outro at their authored length while builds, grooves, and peaks stretch one extra 16-bar movement. Original remains byte-identical.
- Racing now has an Extended arrangement with four new phases — Ignition, Slipstream, Redline, and Cooldown — woven into a ten-section autoplay tour. Extended is the Lab default; Original remains available with its six musical parts unchanged. Existing game integrations still default to Original.
- Added Adventure, an eight-section fantasy quest recipe (camp, explore, town, dungeon, combat, boss, sanctuary, victory). Camp, dungeon, boss, and sanctuary are 16 bars; explore, town, combat, and victory are 32, and each section develops its material across phrases rather than repeating copied halves. Styles are folk (earthy medieval folk), dark (dark medieval fantasy), and orchestral (orchestral RPG), using synthesized harp, recorder, vielle, and bell voices plus frame-drum and tambourine percussion — acoustic-inspired synthesis, not sample recordings. Godot exposes `recipe = "adventure"` and `set_adventure_state(area_phase, discovery, threat, quest_complete)`, and the Audio Lab can audition it.
- Racing and Adventure now support an opt-in `autoplay` (default off). When enabled, the engine attaches a song form that tours the recipe's sections — Racing tours garage, grid, cruise (twice), attack, final-lap, and victory, then loops from grid; Adventure tours all eight sections, then loops from explore. With autoplay off, both recipes remain state-driven and unchanged. The Lab enables autoplay for Racing and Adventure; Suspense is unchanged, and Press Play is still required.
- Suspense is composed from a 27-phase pool: the fourteen base sections plus Scan II, Breach II, Anomaly, and ten new phases (Half-Time, Sparse, Sub-Groove, Syncopated, Drive, Drum Break, False Stop, Filter Break, Harmonic Bridge, Step-Up Bridge). One seed picks a shared harmonic arc, seeded figures, kit modes and phase lengths, and a transition gesture for every join, so the selected phases play as one piece instead of stitched phrases. Versions are a contiguous family of takes of that piece: Version 1 is the first take, not a separate base derivation.
- Added Suspense, a song-form recipe for long tense sessions (first consumer: Arkhos). Music moves through intro, verse, refrain, pre-chorus, chorus, post-chorus, interlude, bridge, solo, outro, and coda instead of looping one four-bar game-state bed. The Audio Lab can audition Racing or Suspense. Godot `GamestrumentsPlayer` accepts `recipe = "suspense"` and `set_trace_state`.
- Audio Lab: added a prominent play/pause control to the unobstructed center of a music-reactive orbit, with beat, rhythm, melody, and bar motion; clear icon-label spacing; synchronized header controls; and a Space shortcut outside form fields.

### Changed

- Adventure's liveliness raised across the board: the tempo floor rose from 58 to 70 BPM (and the Folk/Dark/Orchestral bases from 82/68/90 to 92/78/100 with a wider motion response), the combat and happiness phases carry denser percussion, and melody/harmony/bass/percussion velocities came up, so combat and boss clearly drive and the happy phases stay bright and rhythmic without turning camp and sanctuary into noise. Adventure scores regenerate under generator version 5.0.0.
- Racing's race-groove phases (Starting Grid, Race Flow, Position Fight, and Final Lap) now keep the kit in every block: the drum groove never drops out and restarts mid-loop. A drumless passage is now a deliberate Breather phase before the flow rather than a hole inside a groove. Original and Extended stay byte-identical.
- Racing Seeded now develops and hands off like Suspense: each phase's layers enter and leave across its blocks (the bass bed stays continuous), every join gets a seeded transition gesture over a shared pitch class drawn from Racing's own harmony, and the generation traits reshape the seeded lengths, density and seams while staying neutral at the Lab default. A register ceiling keeps seeded Racing from producing piercing highs. Original and Extended stay byte-identical.
- Racing's Seeded race-groove phases (Ignition, Race Flow, Slipstream, Position Fight, Redline, and Final Lap) now pulse like Starting Grid: their harmony comps rhythmically on the eighth grid instead of holding one whole-note pad per bar, and the bass carries a matching pulse, so the flow and peak phases share the build's forward motion while keeping their own grooves, register and authored breaks. Starting Grid and the Original/Extended arrangements stay byte-identical.
- The seeded arrangements now answer each trait knob with continuous magnitude instead of re-rolling a random branch: Racing's energy widens the tempo and stretches the phases, complexity keeps more layers audible and adds passing onsets, and syncopation displaces more onsets off the grid; Suspense's tension raises the arc and seam intensity, heat thickens the kit and adds closing licks, mystery stretches and darkens the cell, and pulse widens the tempo and thickens the tick; Adventure's motion and danger push the tempo and percussion, danger darkens the register, wonder brightens the harmony, and mystery adds drone and bell accents while thinning voices and slowing the pulse. The default presets stay byte-identical (Racing Original/Extended, Adventure default, and the Suspense seeded pool), and every knob now produces a large, monotonic, audible difference across its full range.
- Suspense now has one authority: the phase pool. The frozen Original, Extended, and Theme presets are retired; the Audio Lab and Godot offer `all-phases` and `seeded` (default). The retired `original`/`extended`/`theme` names still parse, but they resolve to the seeded pool, so scores requested by those names change. The per-preset `progress`-only Disconnect cue is gone.
- Audio Lab: the hi-hat and tambourine now carry their fixed stereo placement (hat right, tambourine left), matching the engine's per-voice pans instead of sitting unpanned as pure-noise voices.
- Suspense phases breathe more: breaks and wait states can shrink to 1–2 bars, and momentum phases can stretch to 24/32 bars, so a version's progression runs longer the way Adventure and Racing sections do instead of every strong phase resolving at 16.
- Fixed piercing high notes in the Suspense pool: any note that landed above the register ceiling (a machine pulse reaching the A#5 range, a Decrypt answer at C6) is folded down an octave — pitch class preserved — and tilted quieter, so the pool's top sits under the low end.
- The Suspense machine pulse is now a steady tick on the bar's chord instead of alternating root and fifth at 16ths, which read as a ringtone.
- Suspense musical pass: every phase now develops instead of holding one texture — the layers enter and leave across its blocks, the shape follows the phase's role (peaks climb out of a sparse exposition, grooves keep the kit and open up, bridges stay light), and a material entry lands on an impact. Breaks and wait states stay short and untouched.
- Audio Lab: the game-phase buttons now use the same names as the sections they cue — Exploit reads Breach, Alert reads Complication, Camp reads Trailhead Camp, Grid reads Starting Grid — so a phase is never called two things in two places. The generated ids, engine labels, presets and frozen takes are unchanged.
- Adventure: Folk now foregrounds plucked harp and recorder/fiddle phrasing, while Orchestral opens with bowed strings, longer melodic lines, and occasional woodwind answers. Warmer mode-aware harmony gives the safe phases clearer, brighter arrivals instead of accidental diminished-chord passages; the dangerous phases retain their darker contrast.
- Audio Lab: replaced the game-type toggle row with a single large selector that shows the active recipe and its description and opens a list of every recipe, so adding a recipe no longer means adding another button.
- Replaced the bundled racing-game demo with three independent Godot integration examples: generate/play the Suspense seeded pool, react to Racing game signals, and direct a Suspense song form. Open `kit/examples/project.godot` instead of `kit/demo/project.godot`; the existing browser Audio Lab remains the single showcase UI. The examples work offline without it.
- Audio Lab: the orbit now shows one ring per musical part in the current section (melody, harmony, bass, drums, plus recipe-specific parts like suspense's drone and cell), each pulsing and rotating from its own note events with its instrument named in the tooltip.
- Audio Lab: fixed the mobile Suspense layout — the Sound World fieldset is no longer squeezed to a sliver and the Sound World/Arrangement buttons each take a full-width row; added a regression test.
- Audio Lab: added ◀/▶ section-step buttons flanking the engine button to cue the previous/next section (wrapping at the ends), with tooltips naming the target; the engine button is slightly narrower to make room.
- Audio Lab: on phones the fixed controls bar now reserves its exact measured height (so it no longer covers the eyebrow/title at rest), and the game-type and sound-world buttons each take a full-width row instead of being squeezed side by side.
- Audio Lab: the engine button is now a live readout — it shows the current section while playing, previews the next one while waiting for the bar, and sweeps between section colours during a crossover ("Handshake → Scan"). The volume + engine row is a fixed top bar on phones.
- Audio Lab: cleaned up the Game signals hierarchy (single heading plus helper, no redundant "Game phase" title), fixed the double separator above Final lap, aligned the toggle with its label, and balanced the generator summary wrap.
- Audio Lab: fixed the mobile layout — the game-type buttons no longer overlap or clip, labels are readable, the game signals are reachable with less scrolling, and the phase buttons use a 2x2 grid for Racing and a 3x2 grid for Suspense.
- Audio Lab: moved the game signal controls (phase, intensity, pressure, final lap) out of the collapsed advanced panel to the top of Music controls, right after the level seed, so the adaptive behavior is immediately visible.
- Audio Lab: published a live HTML5 demo on itch.io at https://gurisitosgames.itch.io/gamestruments-audio-lab-demo and linked it from the kit page.
- Audio Lab: added a master volume control (defaults to 100%, persisted in the browser) and improved the mobile layout with 44px touch targets and stacked header controls.
- Audio Lab polish: clearer control hierarchy, larger hit areas and readable labels, consistent spacing, reserved cue-feedback space, and visible queued/blending/held/loading states. Cue feedback and cancellation stay accessible while scrolling the crossover list.
- Audio Lab: Game type, Sound world and Arrangement now sit above the central player. Detailed controls and crossover sections scroll independently on desktop; expanded Game signals remain reachable, with normal page scrolling on narrow screens.
- Suspense's pool now has independent 16-bar Scan/Scan II and Breach/Breach II pairs, with developed variations rather than repeated eight-bar extensions. Lab and game APIs can hold, advance or resume the form to match gameplay. The drum grid stays continuous.
- Audio Lab: complete score-driven music-section selection replaces hard Jump actions. Cue/queued/blending states and cancellation are visible; Suspense's game signals are separated into an advanced panel. Waiting cues cannot cut an active blend, and canceled cues no longer leave stale release timers or future voices behind.
- Removed the confirmed sustained glass-cell beep from Decrypt, Other Hall and Full Breach. Anomaly, the newer melody passages, effects and drums are unchanged.
- Suspense v2 is no longer a pop song: Santaolalla-style drone + 2–3 note cell, Mr. Robot pulse/clock, static minor harmony, no snare backbeat. Form now also passes through a drop (Break) and a second inverted bridge (Other Hall) without rewriting the v2 beds.
- Removed an internal project name from buyer-visible artifacts: the racing recipe is now **Racing** (`recipe = "racing"`), and the engine module, score ids, catalog path, buyer docs, listing, and Audio Lab use that public name.
- Kit repositioned as **Gamestruments — Adaptive Music for Godot 4**: Gamestruments is the product; the racing recipe is simply Racing, and the demo remains Night Circuit. Storefront slug moves to `gamestruments-godot`.
- Standard price lowered to $12.99 (set 2026-09-10; revisitable after launch).
- `GamestrumentsPlayer.generate` and `set_race_state` now return success booleans with descriptive Godot errors, and generator version `1.10.1` validates every score before exposing it to native or WASM callers.
- Audio Lab: moved score and phase status above the orbit so it never competes with playback, and placed the transport first on narrow screens.
- Audio Lab: restored idle orbit motion and fixed score, seed, comparison, and generation controls that could fail on out-of-range generated melody events.

### Fixed

- Rendered music now plays at a normal listening level instead of sitting roughly 20 dB below it, and every render is limited so it never reaches 0 dBFS. Renders also move from 22 050 Hz to 48 000 Hz mono.
- Native Suspense transitions now follow the pool's generated rules, matching the Audio Lab: there is no `progress`-only Disconnect rule, so only `extract`, `complete`, or `progress >= 0.95` drive the endings.
- Racing Extended: removed the piercing octave lift from the new phases and rewrote Slipstream as a single call-and-response melody with rests and resolved phrase endings. Cooldown now eases into a closing phrase instead of mechanically dropping an octave. The six original musical sections remain unchanged.
- Fixed occasional sharp drum-click spikes at certain tempos in the Audio Lab without reducing the overall mix level or changing the musical parts.
- Godot player teardown now leaves its audio child under Godot's ownership instead of manually freeing it during scene exit; repeated native example scene lifecycles are checked for hangs and leaks.
- Suspense alert/heat cues re-arm after the form leaves the cued section instead of being swallowed. There is no `progress`-only Disconnect rule; `extract`, `complete`, or `progress >= 0.95` drive the endings.
- Fixed the default Suspense opening: Handshake enters Scan on a downbeat kick, then the kick and hats stay on the grid instead of stopping after that first hit. Full rhythm dropouts are reserved for Break and endings.
- Fixed the missing Suspense kick at Scan bar 9: form entrances are prepared by the audio scheduler at their exact boundary instead of rounding a late animation frame up to bar 10. The music and two-bar bed fades are unchanged.
- Audio Lab: Suspense phase buttons now actually change beds. Boot/Scan/Exploit had no adaptive rule so clicks did nothing; Complete/Extract could pin the hold so later clicks never left.
- Audio Lab: switching from Racing to Suspense no longer dies on a missing `garage` section and keeps playing the racing score.
- Fresh Godot projects without a `Music` bus now play through `Master` until a dedicated music bus is added.
- Fixed invalid generated events reaching downstream renderers or trapping WASM, and fixed Godot playback cleanup so repeated generate/play/transition/free cycles exit without extension-owned leaks.
- Audio Lab: made the orbit still while paused and replaced sharp playback jumps with slow continuous rotation and restrained musical pulses in every desktop motion mode.
- Audio Lab: removed duplicate application state introduced during the UI split so playback, regeneration, phase changes, and both play controls stay synchronized.
