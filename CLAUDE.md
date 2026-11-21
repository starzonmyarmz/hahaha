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
- `plugin/src/lib.rs`: plumbing, including the host's transport and the
  `engaged` light.
- `plugin/src/editor.rs` and `plugin/src/bridge.rs`: nothing specific to
  Ha Ha Ha. The page's size and HTML come from `PAGE` in `lib.rs`, and the
  bridge works with any `#[derive(Params)]` struct. `editor::Lights`
  sends named on/off states that aren't parameters to the page.
- `ui/index.html`: the whole editor, compiled into the plugin, with a mock
  bridge for running it in a browser.

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
4. A control for it in the page's markup (`data-knob` for a knob).
5. `PRESETS` and `NORMALIZE` in `ui/index.html`, if presets should set
   it.
6. The controls table in `README.md`, and the presets table if it's in
   them.

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
- Grit, asked for after the first pass ("more grit/distress/splatter"):
  `#rough` displaces every edge so the ink bleeds, `#grain` knocks out
  fine speckle, big worn blotches and horizontal scratches, the cream
  layer sits a few units off register, and there are ink splatters
  with drips, drool from the jaw and drips off the first two HAs. Keep
  it at least this rough.
- Knobs: 60 px cream caps on light chrome knurled skirts, spaced across
  a 540 px face. The first pass had 50 px knobs with dark knurls, which
  looked cramped and heavy.
- Type: the HAs are Shrikhand (subset to "HA"); every label is Fredoka
  SemiBold (subset to letters, digits, "." and "/"). Picked from
  Chewy, Rubik Wet Paint, Lacquer and Shrikhand rendered on the treadle:
  Shrikhand is the only one that's heavy and rounded with no sharp
  corners and still reads at 28 px.
- Presets: a cream window between chrome ◀ ▶ buttons under the knobs,
  in maroon ink. Red is only for the LED.

## Editor

- The window size is set in two places that must change together:
  `PAGE.size` in `lib.rs`, and `.pedal`'s `inline-size` and `block-size`
  in `ui/index.html` (the window leaves 24 px around the pedal).
- The treadle is momentary on the page: pressing sets `repeat` and
  letting go clears it, inside one begin/end gesture. Hold and Latch are
  worked out in the engine, so a MIDI footswitch behaves the same. The
  LED and the treadle's sunk state follow the `engaged` light, not the
  parameter; the mock copies the engine's Latch logic to fake it.
- Presets live in `PRESETS` in `ui/index.html`, in plain units, and
  `NORMALIZE` maps each field to its parameter's normalized value. The
  window works out which preset is showing from the current values, so
  nothing extra is saved with a project.
- Check layout in WebKit, not just Chromium: `scripts/screenshot.sh`
  renders in WebKit, the engine the macOS plugin window uses. An SVG in
  a grid row inside a `<button>` overflowed there, which is why the
  treadle's art and pad are absolutely positioned.
- The local preview server gets cached by the browser. Add `?v=N` after
  each edit. Keep the mock's `specs` in step with the plugin's
  parameters.
- Fonts can't load from the network: embed them as base64 `@font-face`
  data URLs, and keep the file and its license in `ui/fonts`.
- After a visible UI change, retake `docs/hahaha.png` with the
  `screenshot` skill.

## Testing in REAPER

- `scripts/install.sh` installs into `~/Library/Audio/Plug-Ins`. REAPER
  keeps a plugin's dylib loaded, so restart it after reinstalling.
- REAPER only monitors input on armed tracks.

## Bump and install after every change

Whenever a request changes what the plugin does or shows (anything
under `plugin/src` or `ui/`, or its dependencies), finish by bumping
the version and installing the new build, without being asked. The
version is printed on the panel, so the user can tell which build the
host loaded.

1. Commit the change itself first.
2. Run `scripts/bump.sh` with `patch` for a fix or a tweak, or `minor`
   for a new control or feature, or a change to the sound. Use `major`
   only when the user asks. It updates `version` in the root
   `Cargo.toml` and `Cargo.lock`.
3. Commit those two files on their own as "Bump the version to X.Y.Z".
4. Run `scripts/install.sh`. It builds a release bundle, signs it and
   installs it into `~/Library/Audio/Plug-Ins/VST3` and `.../CLAP`, then
   prints the version. Tell the user to restart REAPER to load it.

Bump once per request, not once per commit. Skip it for changes that
don't ship: docs, tests, CI, scripts and skills.

## Releases

`release.yml` runs on `vX.Y.Z` tags, which must match `version` in the
root `Cargo.toml`. Each change already bumps the version, so to release,
tag the current one: `git tag vX.Y.Z && git push origin main vX.Y.Z`.

## nice-plug fork

The plugin is pinned to the `patina` branch of a nice-plug fork, which
adds `ActivateContext::gui_context()`. To upgrade, rebase that branch onto
upstream, push it, and update the `rev` in both `plugin/Cargo.toml` and
`xtask/Cargo.toml`.
