# Gamestruments Godot 4 Kit — Release Plan

**Updated:** 2026-09-08
**Tracked by:** GURI-485 under GURI-564
**Target:** a cross-platform adaptive racing music kit at $24.99.

## Gate Order

1. **Product and contract**
   - Racing-specific six-section scope and public API are explicit.
   - Browser Audio Lab and shipped Godot audio are not conflated.
   - Buyer claims map to evidence.
2. **Engine safety**
   - Rust generation validates every score before returning it.
   - Native and WASM callers receive structured failure instead of invalid output or traps.
   - Determinism and validity pass across at least 256 varied seeds.
3. **Runtime lifecycle**
   - Repeated instantiate, generate, play, state-change, and free cycles exit without leaked-object errors.
   - A fresh extracted demo loads the extension and executes the public API.
4. **Native platforms**
   - Linux x86_64, Windows x86_64, macOS arm64, and macOS x86_64 compile.
   - A universal macOS library contains both architectures.
   - The release Godot binary on each hosted OS loads its library, generates, plays, transitions, frees, and exits cleanly.
5. **Buyer archive**
   - One archive contains all three native libraries, a self-contained demo, complete rebuild inputs, changelog, licenses, and notices.
   - Extracted source rebuilds with the pinned toolchain.
   - Archive scan finds no audio assets, secrets, private paths, or authoring-only packages.
6. **Independent acceptance**
   - A tester with only the candidate archive and buyer docs reaches first sound and a state transition without verbal help.
   - A human listens to the exact packaged Godot runtime on every platform and approves its audio.
   - Storefront screenshots and audio/video evidence come from the packaged Godot demo.
7. **Release and launch**
   - Source is an immutable remote commit and tag.
   - The release packet records archive digest, workflow run, platform evidence, listing fields, price, and rollback plan.
   - itch.io is changed only after explicit approval of the exact packet; there is no automatic publication.

Failure at any gate returns the work to the relevant implementation or documentation gate. Passing CI alone does not satisfy independent acceptance or publication approval.

## Claim-to-Evidence Matrix

| Claim | Required evidence |
|---|---|
| Deterministic score from namespace, seed, style, palette, traits, and version | Rust repeat-generation assertions plus native/WASM parity fixtures |
| Safe generated score | Engine validation tests and 256-seed stress test across all styles and trait ranges |
| Six adaptive racing sections with bar-quantized crossover | Transport tests plus exact-runtime Godot smoke and human demo run |
| Zero samples and offline runtime | Archive audio-extension scan and source dependency review |
| Linux, Windows, and universal macOS support | Successful target-native release jobs and GDExtension mappings |
| Godot 4.7+ support | Fresh-project smoke with the pinned 4.7.2 release on every target OS |
| MIT source included and rebuildable | Extracted archive build using root manifests, lockfile, and pinned toolchain |
| Exact buyer sound shown publicly | Media captured from the immutable packaged Godot demo, not the Web Audio Lab |

## Clean-Room Buyer Tasks

The independent tester receives only `gamestruments-<version>-godot4.zip` and proposed listing copy.

1. Identify Godot and platform requirements from `kit/docs/README.md`.
2. Open `kit/demo/` directly and reach audible output.
3. Copy the root addon into a new Godot 4.7 project.
4. Add `GamestrumentsPlayer`, configure a non-empty namespace, and check `generate(seed)` succeeds.
5. Trigger grid, cruise, attack, final-lap, and victory states and observe musical transitions.
6. Generate the same input twice, then change the seed, and confirm stable/suitably different results.
7. Follow one troubleshooting path without repository access.
8. Confirm the archive contents, limitations, and listing claims agree.
9. Record OS, Godot version, time to first sound, failures, unclear wording, and listening notes.

## Risks

- **Native lifecycle:** Godot playback resources can leak at shutdown. Automated repeated-free smoke blocks release.
- **Cross-platform ABI:** compilation does not prove loadability. Every advertised binary must run under Godot on its own OS.
- **Unsigned native files:** macOS quarantine or platform security may require buyer action. Troubleshooting must remain explicit and target testing must use downloaded artifacts.
- **Audio expectation mismatch:** the mono runtime is intentionally leaner than the stereo Web Audio Lab. Storefront evidence must use the runtime.
- **Narrow genre model:** current parameters and sections are racing-specific. The product is sold that way rather than as a general adaptive-music engine.
- **Determinism drift:** output identity is scoped to generator version. Version changes require fixture regeneration and changelog coverage.
- **Licensing:** any AGPL authoring dependency or missing MPL notice blocks the archive.

## Post-Launch Measurement

- Track itch views, purchases, conversion, refunds, and support categories manually; the kit has no telemetry.
- Review weekly for the first month, monthly through the first quarter, then quarterly.
- Treat fewer than 20 purchases in 60 days with support above one ticket per five buyers as a positioning or usability investigation trigger.
- Pause and audit if refund rate exceeds 15% in the first 100 sales, three independent buyers cannot reach sound after following the docs, or any license/distribution defect appears.
- Small samples are directional; do not present them as validated market conclusions.
