# Ha Ha Ha

Read `README.md` first for what the plugin does and how to build and run
it. This file has the maintainer details it leaves out. The plan and its
reasoning live in the Keep Up idea "Ha Ha Ha: tempo-synced repeater
pedal" (`keepup idea show 65`).

## Layout

- `plugin/src/engine.rs`: all DSP, with no plugin framework types, so tests
  drive it directly. `Settings` holds the controls in their own units,
  `Transport` the host's tempo and position at the start of a block.
- `plugin/src/params.rs`: host parameters, mapped to `Settings`.
- `plugin/src/lib.rs`: plumbing, including the host's transport.

## Engine

- Times are absolute sample counts (`now`). `history` is a ring that's
  always written. A burst copies `history` from one max slice length
  before its start into `slice` and keeps writing `slice` as it plays,
  so a long hold never loses its audio and a longer Rate mid-burst can
  reach back before the original start.
- A slice is played in phase with the grid: on the first pass the read
  position is the live input, so nothing is heard until the first wrap.
- Each pass's start and length are settled `fade` samples before the
  wrap, when its pre-roll (the audio just before its start) begins to
  crossfade in. A Rate change waits for that point.
- FADE, BEND and ROLL count from the first repeat and reset with a new
  burst. FADE glides through each pass (`Burst::progress`), so its level
  never steps. BEND is varispeed: `pos` is fractional, read with a cubic
  Hermite, and advances by `speed()`, which drifts up to 12 semitones a
  beat, clamped to three octaves down and two up. At zero BEND reads
  land on whole samples, so the step 2 goldens stay bit-identical. ROLL
  shrinks the next pass's length when it's settled and keeps its start.
- The envelope (`env`) is equal-power between live and repeats; the
  wrap crossfade is linear, because tail and pre-roll are often alike.
- Keep allocations in `Engine::new`: `assert_process_allocs` aborts on
  any in `process`. Retriggering copies up to two max slice lengths
  once, which is the costliest thing it does.
- `wraps_and_rate_changes_dont_click` compares the biggest
  sample-to-sample jump in the output with the input's. Keep it passing
  after any change to the crossfades.

## Adding a parameter

1. `Settings` in `engine.rs` and its `Default`.
2. `HaHaHaParams` in `params.rs`, and `settings()`. Parameter IDs must
   never change after a release.
3. The mock's `specs` in `ui/index.html`.
4. The controls table in `README.md`.
- `plugin/src/editor.rs` and `plugin/src/bridge.rs`: nothing specific to
  Ha Ha Ha. The page's size and HTML come from `PAGE` in `lib.rs`, and the
  bridge works with any `#[derive(Params)]` struct.
- `ui/index.html`: the whole editor, compiled into the plugin, with a mock
  bridge for running it in a browser.

## Sound changes and golden renders

`plugin/tests/golden` holds renders the tests compare against. After an
intended change to the sound, rerun with `BLESS=1 cargo test` to rewrite
them. Never commit rewritten renders until the user has listened to
them, and say in the commit what changed in the sound.

## Things clap-validator catches

Run `scripts/clap-validator.sh "target/bundled/Ha Ha Ha.clap"` after any
DSP change, not just in CI.

- Subnormal output. Decay tails and smoothers decay into subnormals in
  silence. Flush state and output below about 1e-30 to zero.
- NaN or runaway read positions from pitch drift. Clamp read rates.
- `v2s_f32_percentage` doesn't append `%`, so `with_unit("%")` is right
  with it.

## Design

Settled with the user; keep to it.

- Object: a compact pedal shaped like a BOSS, with a knob row on top and
  a wide hinged treadle with a rubber pad, because the whole plate is
  held down to repeat. Borrow the form factor only, not BOSS colours,
  label strip or logo layout. It should feel like an indie-shop
  noise-punk pedal.
- Controls, left to right: RATE, FADE, BEND, ROLL, MIX. A HOLD/LATCH
  toggle and an LED (lit while repeating) above the knobs. The treadle
  is the trigger and sinks while repeating. There's no separate bypass:
  the treadle replaces the template's `stomp`.
- Version: small, in the bottom-right corner of the face (`v1.0.0`).
- Palette: bubblegum-pink powder coat (#F4A6C0), maroon ink (#6B1F3A),
  cream knobs (#F6EEDD) with chrome skirts, red LED (#FF3B3B).
- Art: a goofy, rounded laughing skull printed on the treadle in maroon
  with cream fills, with HA HA HA spilling from its mouth, each HA
  smaller or more worn than the last. **It's static and doesn't react
  to settings or input.** That's a deliberate exception to the
  template's "art follows parameters" principle: the LED and the sinking
  treadle carry the state.
- Ink: illustrated noise punk. No SVG strokes: every line is a filled
  shape whose thickness swells and thins like brush ink, with rounded
  corners and ends, and a worn screen-print texture knocked out of the
  ink so the pink shows through.

## Editor

- The window size is set in `PAGE.size` in `lib.rs` and by the page's
  outer element in `ui/index.html`. Change them together.
- The local preview server gets cached by the browser. Add `?v=N` after
  each edit. Keep the mock's `specs` in step with the plugin's
  parameters.
- Fonts can't load from the network: embed them as base64 `@font-face`
  data URLs, and keep the file and its license in `ui/fonts`.
- After a visible UI change, retake `docs/hahaha.jpg` with the
  `screenshot` skill.

## Testing in REAPER

- `scripts/install.sh` installs into `~/Library/Audio/Plug-Ins`. REAPER
  keeps a plugin's dylib loaded, so restart it after reinstalling.
- REAPER only monitors input on armed tracks.

## Releases

`release.yml` runs on `vX.Y.Z` tags, which must match `version` in the
root `Cargo.toml`. To release: bump `version`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## nice-plug fork

The plugin is pinned to the `patina` branch of a nice-plug fork, which
adds `ActivateContext::gui_context()`. To upgrade, rebase that branch onto
upstream, push it, and update the `rev` in both `plugin/Cargo.toml` and
`xtask/Cargo.toml`.
