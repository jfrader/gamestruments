# Godot integration examples

Three small, independent scenes that show how to call the Gamestruments addon
from a real Godot 4.7 project. They are reference integrations, not a game:
each one focuses on a single part of the public API so you can copy just the
piece you need.

- One Godot project: `kit/examples/project.godot`.
- One shared addon copy: `res://addons/gamestruments/`.
- No browser, network, or Audio Lab UI is involved. Audio is generated and
  played by the addon.

## Run

1. Open `kit/examples/project.godot` in Godot 4.7.x.
2. F5 runs **01 Playback** only (by design; supporting reference, not a full Lab UI).
3. To run another example, open its scene and press F6. The three scenes are independent.

| Example | Scene | Script | What it shows |
|---|---|---|---|
| 01 Playback | `01-playback/playback.tscn` | `01-playback/playback.gd` | Generate and play a Suspense Theme title bed; check `generate()`; restart; live section readout. |
| 02 Game signals | `02-game-signals/game_signals.tscn` | `02-game-signals/game_signals.gd` | Map simulated race events to `set_race_state` requests (original Racing); see requested vs currently playing. |
| 03 Song form | `03-song-form/song_form.tscn` | `03-song-form/song_form.gd` | Suspense form: `set_trace_state`, `cue_section`, `set_form_hold`, `advance_form`. |

Each folder has a short README with what to copy and where your own callbacks
plug in.

## Addon availability

The archive ships the entire `addons/gamestruments/` directory at
`kit/examples/addons/gamestruments/`. Copy the whole directory (matching
placement); restart Godot and wait for import. Godot loads the platform-matching
binary; the folder contains libraries for all packaged targets (not simultaneous
on one OS).

A repository checkout may not include the native binaries. The examples create
the player with `ClassDB.instantiate("GamestrumentsPlayer")` and show a clear
in-scene error if the class is unavailable, so a missing addon does not crash
the editor. Once the addon is installed in your own project, you can use the
typed `GamestrumentsPlayer` node directly.
