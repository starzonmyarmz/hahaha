# Ha Ha Ha

Tempo-synced stutter and beat repeat, from rolls to tape stops: a VST3 and CLAP plugin

Read before working:
- `DESIGN.md` before touching anything people see.
- The plan and its reasoning are in the Keep Up idea "Ha Ha Ha: tempo-synced repeater pedal" (`keepup idea show 65`).

## Commands

```sh
cargo test                                  # engine tests and golden renders, offline
BLESS=1 cargo test                          # rewrite golden renders after an intended change to the sound
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo xtask bundle hahaha --release     # target/bundled/Ha Ha Ha.{vst3,clap}
scripts/install.sh                          # bundle, sign ad hoc and install on macOS as X.Y.Z-dev+<sha>
scripts/clap-validator.sh "target/bundled/Ha Ha Ha.clap"
scripts/screenshot.sh                       # retake docs/hahaha.png
scripts/release --dry-run                   # what the next release would be
```

Work on the editor in a browser against the mock: `cd ui && python3 -m http.server 8765`, then open http://127.0.0.1:8765 (add `?version=1.2.0` to show a version). To load the page live in the plugin, set `HAHAHA_UI_URL` to that URL before starting the host.

## Layout

- `plugin/src/engine.rs`: the DSP, with no host types, so `cargo test` runs it offline. `Settings` holds the controls in their own units, `Transport` the host's tempo and position at the start of a block.
- `plugin/src/params.rs`: the parameters. Their defaults come from `Settings::default()`, and `settings()` turns them into the engine's `Settings`.
- `plugin/src/lib.rs`: host plumbing, plugin IDs, `PAGE` (title, window size, version), the host's transport and the `engaged` light.
- `plugin/src/editor.rs`, `bridge.rs`: the webview editor and its message bridge, with nothing specific to Ha Ha Ha. `editor::Lights` sends named on/off states that aren't parameters to the page.
- `plugin/tests/`: engine tests and golden renders in `golden/`. `support/` has the test tone and golden helpers.
- `ui/index.html`: the whole editor, compiled into the plugin, with a hand-written mock bridge (`mockBridge()`) for running it in a browser.
- `xtask/`, `bundler.toml`: nice-plug's bundler.
- `scripts/`: install, clap-validator, screenshot and release, with `pre-release.sh` retaking the screenshot for each release.
- `.claude/skills/`: `design-kickoff` (settle the look before building the editor), `screenshot` and `bless-goldens`. Use them rather than working from memory.

## Invariants and footguns

What was learned the hard way. Add to this list whenever something breaks in a way the code doesn't make obvious.

- `CLAP_ID` and `VST3_CLASS_ID` in `lib.rs` never change after the first release, and parameter IDs never change without a major release. Hosts use them to find the plugin and restore saved projects.
- After a change to `plugin/src` or `ui/`, run `scripts/install.sh` so the user can try it. It installs as `X.Y.Z-dev+<sha>`, so the panel shows which build the host loaded. REAPER keeps a plugin's dylib loaded: tell the user to restart it.
- Don't bump the version by hand. It only changes at release.

### Parameters and the engine

- The engine works in f64. Each `Settings` field is in the units the control shows. When a few knobs drive many engine values, add a `Controls` struct whose `settings()` does the mapping (Ghosted, Altitude), so the engine can still be tested on raw values.
- The engine glides to new settings itself, with one-pole smoothers built from `coef()`, so turning a knob doesn't click. `reset()` jumps to the current settings; the plugin calls it on activate and reset.
- Keep `stomp` on effects. Glide the wet amount toward 0 or 1, and `flush()` it: anything gliding toward 0 decays into subnormals otherwise.
- Random numbers come from a seeded RNG (Ghosted's xorshift64* `Rng`), so golden renders repeat exactly.
- Tempo sync: read the host transport in `process` and pass the engine plain values. Fall back to 120 BPM when the host gives no tempo.
- There's no `stomp`: the treadle (`repeat`) is the effect's on/off, and with it up the output is dry.

### Adding a parameter

1. `Settings` in `engine.rs` and its `Default`.
2. `HaHaHaParams` in `params.rs`, and `settings()`.
3. The mock's `specs` in `ui/index.html`.
4. A control for it in the page's markup (`data-knob` for a knob).
5. `PRESETS` and `NORMALIZE` in `ui/index.html`, if presets should set it.
6. The controls table in `README.md`, and the presets table if it's in them.

### The repeater

- Times are absolute sample counts (`now`). `history` is a ring that's always written. A burst copies `history` from one max slice length before its start into `slice` and keeps writing `slice` as it plays, so a long hold never loses its audio and a longer Rate mid-burst can reach back before the original start.
- A slice plays in phase with the grid: on the first pass the read position is the live input, so nothing is heard until the first wrap.
- Each pass's start and length are settled `fade` samples before the wrap, when its pre-roll (the audio just before its start) begins to crossfade in. A Rate change waits for that point.
- FADE, BEND and ROLL count from the first repeat and reset with a new burst. FADE glides through each pass (`Burst::progress`), so its level never steps. BEND is varispeed: `pos` is fractional, read with a cubic Hermite, and advances by `speed()`, which drifts up to 12 semitones a beat, clamped to three octaves down and two up. At zero BEND reads land on whole samples, so the older goldens stay bit-identical. ROLL shrinks the next pass's length when it's settled and keeps its start.
- The envelope (`env`) is equal-power between live and repeats; the wrap crossfade is linear, because tail and pre-roll are often alike.
- Keep allocations in `Engine::new`: `assert_process_allocs` aborts on any in `process`. Retriggering copies up to two max slice lengths once, which is the costliest thing it does.
- `wraps_and_rate_changes_dont_click` compares the biggest sample-to-sample jump in the output with the input's. Keep it passing after any change to the crossfades.
- Never commit rewritten golden renders until the user has listened to them, and say in the commit what changed in the sound.

### Things clap-validator catches

Run `scripts/clap-validator.sh` after any DSP change, not just in CI.

- Subnormal output. Recursive filters, envelopes and smoothers decay into subnormals in silence. Flush state and output below about 1e-30 to zero.
- nice-plug's `v2s_f32_hz_then_khz` already appends Hz/kHz, so adding `with_unit(" Hz")` doubles it. `v2s_f32_gain_to_db` and `v2s_f32_percentage` don't, so `with_unit` is right with those.
- NaN from filters whose corner frequency is above Nyquist at low sample rates. Clamp corners below Nyquist.
- NaN or runaway read positions from pitch drift. Clamp read rates.

### Editor

- The window size is set in `PAGE.size` in `lib.rs` and by `.pedal`'s `inline-size` and `block-size` in `ui/index.html` (the window leaves 24 px around the pedal). Change them together.
- The treadle is momentary on the page: pressing sets `repeat` and letting go clears it, inside one begin/end gesture. Hold and Latch are worked out in the engine, so a MIDI footswitch behaves the same. The LED and the treadle's sunk state follow the `engaged` light, not the parameter; the mock copies the engine's Latch logic to fake it.
- Presets live in `PRESETS` in `ui/index.html`, in plain units, and `NORMALIZE` maps each field to its parameter's normalized value. The window works out which preset is showing from the current values, so nothing extra is saved with a project.
- Check layout in WebKit, not just Chromium: `scripts/screenshot.sh` renders in WebKit. An SVG in a grid row inside a `<button>` overflowed there, which is why the treadle's art and pad are absolutely positioned.
- The browser preview gets cached: add `?v=N` after each edit. Keep the mock's `specs` in `ui/index.html` in step with the plugin's parameters, including the skew of any skewed range. (This plugin predates the kit's generated `ui/mock.json`.)
- To show the page something that isn't a parameter (an LED, a scope, a meter), add an `editor::Lights` entry, an `Arc<AtomicBool>` shared with the audio thread. (This editor predates the kit's `WebEditor::with_feed`.)
- SVG reused through `<use>` doesn't get page CSS rules. Stamp it from a `<template>` instead, or put attributes on the elements themselves.
- Fonts can't load from the network: embed them as base64 `@font-face` data URLs, and keep the file and its license in `ui/fonts`.
- The macOS webview is the system WKWebView, so check a newer CSS feature is in Safari for the macOS versions you support. Windows uses WebView2.
- After a visible UI change, retake the screenshot with the `screenshot` skill, commit it with the change, and keep the README `alt` text current. Nothing on GitHub retakes it.

### Testing in REAPER

- REAPER only monitors input on armed tracks.
- Multi-output plugins: declare a second layout without the aux port so hosts without multi-out still load them, and keep a scratch buffer for the aux signal in that case (see Halfwit).

### nice-plug fork

Plugins pin the `patina` branch of [starzonmyarmz/nice-plug](https://github.com/starzonmyarmz/nice-plug/tree/patina): upstream plus one commit that adds `ActivateContext::gui_context()`. Keep the fork to that one commit.

- Upgrading: rebase `patina` onto upstream, push, and bump the `rev` in `plugin/` and `xtask/` `Cargo.toml` in every plugin. Then run the tests and clap-validator in each.
- Don't send the patch upstream. It was written with AI, and nice-plug's [AI policy](https://rust.audio/community/ai/) doesn't accept AI-written code, PR descriptions or comments.

## Conventions

- Conventional Commits with Title-Case scopes: `feat(Engine): …`. The body explains why.
- Tests ship in the same commit as the change. Pure logic is tested; the UI or host shell stays thin.
- A `feat`, `fix` or `perf` commit that someone using Ha Ha Ha would notice adds a line under `## [Unreleased]` in `CHANGELOG.md`, in the same commit, written for them, not as a code summary.
- This repo is public. Keep real names, logins, paths and work data out of code, fixtures, docs, the changelog and commits.
- Breaking means a renamed parameter ID or a preset that no longer loads.

## Releasing

Releases happen only when asked. Never bump, tag or publish on your own.

`scripts/release` works out the bump from commits since the last tag, finalizes the changelog, commits `chore(Release): X.Y.Z`, tags `vX.Y.Z` and pushes. The tag triggers `.github/workflows/release.yml`.
