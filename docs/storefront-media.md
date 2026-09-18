# Storefront media capture

How the screenshots and the video for the itch.io page are produced. Run this
against a **packaged release**, not the development checkout: the native-audio
evidence has to come from the exact archive buyers download.

## 1. Native screenshots (required)

The Godot package smoke captures the three examples at desktop and narrow
viewports, plus the missing-addon error state, from the extracted kit:

```bash
cd <repo>
cargo build -p gamestruments-godot --release
__GLX_VENDOR_LIBRARY_NAME=mesa LIBGL_ALWAYS_SOFTWARE=1 \
  node tests/godot-package-smoke.mjs \
    --godot /usr/bin/godot \
    --library target/release/libgamestruments_godot.so \
    --screenshots /tmp/opencode/shots
```

- `__GLX_VENDOR_LIBRARY_NAME=mesa LIBGL_ALWAYS_SOFTWARE=1` is required on this
  machine (`docs/kit-qa-runbook.md`): repeated viewport readbacks stall on the
  NVIDIA driver, and Mesa's V-sync warning is expected.
- Godot 4.7.2 headless **editor** startup crashes with the extension; this runs
  the runtime, which is the passing path.
- Files land as `<scene>-<state>.png` and `narrow-<scene>-<state>.png`.
  Pick the desktop shots for the page; the `-neg` ones show the missing-addon
  error and are for the docs, not the storefront.

## Visual direction (learned the hard way)

A raw Movie Maker capture of the Godot example is a grey dev panel with
buttons: technically native, useless as a showcase. The storefront video's
visual identity is the **Audio Lab's orbit**, so the cut is built from Lab
footage:

1. Record three ~13 s Lab clips (Suspense, Racing, Adventure) at 1280x720 with
   Playwright's isolated profile; the orbit reacts to the music.
2. Compose them with cross-fades and the storyboard captions.
3. Mux the engine's offline WAV (`tools/render-ab.mjs`) as the audio, with the
   explicit `-map` above.

The native kit still supplies the **screenshots** for the page, and a short
native segment if a cut needs "this is the real engine" evidence — but it is not
the slideshow's main visual.

### The audio must be the Lab's own

Rendering the beds separately and muxing them onto separately recorded clips
produces a video whose music is not the music you can hear in it. Record the
session and its audio together:

1. Drive the Lab with **random seeds** (`#level-seed` + apply), one recipe and
   style per clip, so the take is audible and varied.
2. Capture the Lab's mix **in-page**: patch `AudioContext` and `AudioNode.connect`
   before the app loads, tee every connection to `ctx.destination` into a
   `MediaStreamAudioDestinationNode`, and record it with `MediaRecorder`.
   Playwright's own video has no audio track.
3. Trim each clip's lead-in by `videoDuration - audioDuration - 0.2` so the
   picture and the music start together, then cross-fade and caption as above.
   Verified with `volumedetect`: a real `max_volume`, not `-91.0 dB`.

## 2. Native video (required for promotion)

Record the **packaged** examples with Godot's Movie Maker so the audio is the
real native engine, not browser playback:

```bash
# from the extracted archive, with the packaged addon in place
godot --path <extract>/kit/examples \
  --write-movie /tmp/opencode/gamestruments-native.avi \
  --fixed-fps 30 --resolution 1280x800
```

- Drive the three examples (playback, game signals, song form) and the state
  changes you want on camera; the storyboard and captions live in
  `storefront/listing.md` under "Video sequence (pending)" (35–45 s, music and
  captions only, no voiceover).
- **Known gotcha (measured 2026-09-18):** on this machine Movie Maker writes a
  correct `pcm_s16le` audio stream but the samples are **digital silence**
  (`mean_volume: -91.0 dB`), with either the default audio driver or
  `--audio-driver Dummy`. The video is good; the audio has to be supplied
  separately.
- **Workaround:** render the same score to WAV with the engine's own offline
  renderer (from the packaged source, same generator version as the archive)
  and mux it onto the Movie Maker video:

  ```bash
  # `-map` is mandatory: the AVI already carries its own silent PCM track and
  # ffmpeg would otherwise pick it as "best audio" and mux silence.
  ffmpeg -y -i /tmp/opencode/native.avi -i /tmp/opencode/muxaudio/mono.wav \
    -map 0:v:0 -map 1:a:0 -c:v copy -c:a aac -b:a 192k -shortest \
    /tmp/opencode/gamestruments-native-1.0.1.mp4
  ```

  Always verify before publishing:
  `ffmpeg -i out.mp4 -map 0:a -af volumedetect -f null -` must show a real
  `max_volume`, not `-91.0 dB`.

### Captions: keep them off the music

Fran, after watching a cut full of "phase name — description" subtitles: "don't
add 'phase name - description' subtitles, let the user feel it." A storefront
cut carries at most a closing call to action and the source label; it does not
narrate each phase. Explain the product on the page, not over the music.

## 3. Browser footage (optional, must be labelled)

The Audio Lab preview may be recorded for the final "try the demo" beat, but it
is **browser** sound and must be identified as such against the native footage.

```bash
# isolated profile, never the shared MCP browser session
node --input-type=module -e '
import { chromium } from "@playwright/test";
const b = await chromium.launch();
const p = await b.newPage({ viewport:{width:1280,height:800}, recordVideo:{dir:"/tmp/opencode/video",size:{width:1280,height:800}} });
await p.goto("https://gamestruments.gurisitos.games/#lab", { waitUntil:"domcontentloaded" });
await p.waitForSelector("#section-list li"); await p.waitForTimeout(1500);
await p.locator("#center-play").click(); await p.waitForTimeout(8000);
const v = p.video(); await p.close(); console.log(await v.path()); await b.close();'
```

## 4. Publishing

- Cover: 315:250 (630x500 recommended). Screenshots: 3–5.
- Upload on the itch page, then tick the matching boxes in
  `storefront/listing.md` (the "Media Capture Checklist" and "Exact Storefront
  Media" tables) so approval covers the exact files.
- Keep the archive SHA-256 and the media list together in the release packet.
