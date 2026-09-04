# Gamestruments Kit — Fran's Author QA Runbook (GURI-567)

**Artifact under test (RC):**
- `gamestruments-0.1.0-rc1-godot4.zip`
- Bytes: 2337255
- SHA-256: `8f705ead0886b9517cafefe722203b531f8b98c4dbd80f0426d3f6726c08a69f`

**Verify after download (always):**
```sh
sha256sum gamestruments-0.1.0-rc1-godot4.zip
# must match the value above
unzip -l gamestruments-0.1.0-rc1-godot4.zip | head
```

**This is author QA only.**  
The clean-room buyer test (independent tester given *only* the archive + docs, no repo access or hints) is defined separately in `docs/kit-plan.md` and must still be executed before launch.

Godot version for this runbook: 4.7+ (developed/validated on 4.7.2).

---

## (a) Download / extract the zip (copy-paste)

```sh
# From itch (or the release attachment) into a clean dir
mkdir -p ~/Downloads/gamestruments-rc
cd ~/Downloads/gamestruments-rc
# (download the zip here)
sha256sum gamestruments-0.1.0-rc1-godot4.zip   # verify
unzip gamestruments-0.1.0-rc1-godot4.zip
ls -1
# Expected top level: addons/  crates/  kit/  LICENSE.md  THIRD_PARTY_NOTICES.md
```

---

## (b) Open kit/demo in Godot 4.7 with the addon (copy-paste)

The demo is now self-contained: the archive includes an identical `addons/gamestruments/`
copy inside `kit/demo/addons/`. 

1. Extract the archive (see (a)).
2. Open the extracted `kit/demo/` folder **directly** as a Godot 4 project:
   ```
   godot --path /path/to/extracted/kit/demo
   ```
   (or File → Open Project in the editor and point at the `kit/demo/` folder that
   contains `project.godot`). The addon is already present at the correct relative
   location `res://addons/gamestruments/...` and `run/main_scene` is `res://kit_demo.tscn`.
3. (Optional but recommended for clean logs) Run the import step headless first:
   `godot --path /path/to/extracted/kit/demo --headless --import`
4. Run the scene (F5 or Play) or boot headless.

You should see the UI appear (or "Initialize godot-rust") with no missing-dependency
or ClassDB / extension errors. Press **Generate** to hear the first score. (The
`kit_demo.tscn` now correctly references `res://kit_demo.gd` for the demo-as-project case.)

---

## (c) Interactive checks (copy-paste steps)

Perform in the running demo scene. Note observations in the table at the end.

- **Generate with default**: Press Generate (uses "fusion" or the UI default + seed). Music should start immediately (garage section).
- **Change style + re-generate** (all four):
  - Select `fusion` → Generate → listen
  - Select `neon` → Generate → listen (brighter / different timbre)
  - Select `funk` → Generate → listen (groove emphasis)
  - Select `chip` → Generate → listen (chiptune character)
  - Re-generate after each; confirm audible difference.
- **Change seed, confirm different music**:
  - Edit seed to `qa-seed-002` → Generate
  - Edit seed to `qa-seed-003` → Generate
  - Confirm different motifs / section feel (deterministic per seed).
- **Override a voice**:
  - Put a non-empty string in one of the voice fields (e.g. `melody_voice = "saw"` or any style-supported voice name from api.md)
  - Generate again → confirm the melody voice changed character.
- **Click each of the six section buttons; confirm musical change**:
  - Garage
  - Grid
  - Cruise (uses current Energy as intensity)
  - Attack
  - Final Lap
  - **Victory** (uses `phase = "finish"`, `finish_result = "win"` — the 5-arg variant now supported in the binding)
  - Each should produce a bar-quantized transition. Victory should feel like a musical release + win stinger.
- **Listen for loop seam**:
  - Let any section (e.g. race/cruise) play for 20–30 s.
  - Confirm it loops seamlessly (no obvious click/pop at bar boundary; the transport is bar-quantized).
- **Volume sanity vs Godot master**:
  - Play at default.
  - Lower Godot's Master bus and Music bus independently.
  - Confirm the synth follows the bus volume (no clipping at 0 dB, audible at -12 dB).
- **Quit cleanly**:
  - Close the running scene / editor.
  - No crash, no repeated "ObjectDB leak" spam that grows, editor returns to normal state.

---

## (d) Claims check (tick against actual demo + shipped code)

Open in the extracted archive (or the checked-in copies):
- `kit/docs/README.md`
- `kit/docs/api.md`
- `kit/docs/limitations.md`

For each literal claim, run the corresponding action in the demo (or inspect source) and tick.

From `kit/docs/README.md` (Main Claims):
- [ ] `secret + seed + palette + style + traits` → unique deterministic adaptive score (test by changing one var at a time; same inputs = same music)
- [ ] Zero samples in the archive or runtime path (confirm: no .wav/.ogg/.import audio assets under kit/ or addons/)
- [ ] Bar-quantized crossovers on state changes via `set_race_state` (listen + watch the section labels if exposed)
- [ ] Godot 4.x native via GDExtension (mono synth only) (the player node appears, audio is mono 22050 Hz)

From `kit/docs/api.md` (key public surface):
- [ ] Exported: `project_secret`, `style`, `melody_voice`/`harmony_voice`/`drive_voice`/`bass_voice`, `energy`/`complexity`/`brightness`/`syncopation`
- [ ] `generate(seed: String)`
- [ ] `set_race_state(phase: String, intensity: float, pressure: float, final_lap: bool, finish_result: String = "none")` (4-arg and 5-arg both work)
- [ ] Player creates an internal AudioStreamGeneratorPlayback routed toward Music bus

From `kit/docs/limitations.md` (honest limits that must not be contradicted by demo):
- [ ] Only linux.x86_64 binary shipped (source for others)
- [ ] No samples, no Strudel, no pre-baked WAVs
- [ ] No authoring UI
- [ ] Determinism is semantic/musical (not byte-identical across Godot/gdext patches)
- [ ] "Victory" uses finish phase (documented)

If any claim is observably false in the demo → blocker.

---

## (e) Result recording template

Date: YYYY-MM-DD  
Godot: 4.7.x  
Tester: Fran (author QA)  
Artifact SHA verified before start: YES / NO

| Check | Result (pass / fail / note) | Evidence / comment |
|-------|-----------------------------|--------------------|
| Extract + sha match | | |
| Extension loads, no ClassDB error | | |
| Generate default | | |
| All 4 styles audible diff | | |
| Seed change → different music | | |
| Voice override | | |
| 6 section buttons (incl. Victory w/ 5-arg) | | |
| Seamless section loop | | |
| Volume follows buses | | |
| Clean quit (no crash) | | |
| README claims vs demo | | |
| api.md surface present | | |
| limitations.md not contradicted | | |

**Overall author sign-off**:  
[ ] All interactive checks passed  
[ ] Claims match shipped demo + source  
[ ] No new crashes/leaks introduced by packaging

**Next required step**: Independent clean-room buyer test (see kit-plan.md).

---

## Notes for this run (GURI-567)

- Performed on the exact archive produced by `tools/package_kit.sh` from commit 9f105fa (the RC packaging branch).
- Smoke also executed headless from the *extracted* copy only (never from source tree) — see verification log in PR.
- The 5-arg `set_race_state(..., finish_result)` path is exercised for Victory.
- Residual ObjectDB warning on some quit paths is pre-existing (documented in code + limitations); not introduced by packaging.

Update this file only when repeating author QA on a new artifact.
