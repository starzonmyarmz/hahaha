# Ha Ha Ha

A tempo-synced repeater pedal, as a VST3 and CLAP plugin for macOS and
Windows. Hold the treadle and it grabs the last beat division you played
and stutters it on the grid, locked to your host's tempo. Let go and
you're back to dry.

It's built footswitch-first for playing live, and every control can be
automated in the studio.

## Installing

Download the zip from
[Releases](https://github.com/starzonmyarmz/hahaha/releases) and follow
`INSTALL.txt` inside it.

## Building

```sh
cargo xtask bundle hahaha --release        # target/bundled/Ha Ha Ha.{vst3,clap}
scripts/install.sh                         # bundle, sign ad hoc and install on macOS
scripts/clap-validator.sh "target/bundled/Ha Ha Ha.clap"
```

## Tests

`cargo test` runs the engine offline, in host-sized blocks, against
stored golden renders and synthetic checks. After an intended change to
the sound, rerun with `BLESS=1` to rewrite the renders, and listen to
them before committing.

## Working on the UI

The editor is a webview showing `ui/index.html`, which is compiled into
the plugin. Without the plugin, the page runs against a mock, so you can
work on it in a browser:

```sh
cd ui && python3 -m http.server 8765    # open http://127.0.0.1:8765
```

Add `?version=1.2.0` to show a version other than `0.0.0`. To load
the page live in the plugin, set `HAHAHA_UI_URL` to that URL
before starting the host.

## CI and releases

- `check.yml` runs on pushes to main and on PRs: the tests and
  clap-validator on macOS, and clap-validator on the Windows build.
- `release.yml` runs on `vX.Y.Z` tags. The tag must match `version` in
  the root `Cargo.toml`. It builds a universal macOS bundle and a
  Windows bundle, validates both, and publishes `hahaha-X.Y.Z.zip`
  with `INSTALL.txt`.
- `screenshot.yml` runs on pushes to main that touch `ui/`, the version
  or `plugin/src/lib.rs`. It retakes `docs/hahaha.jpg` with
  `scripts/screenshot.sh`, sets its width in the README, and commits both
  if the image changed.

## Built with

- [nice-plug](https://codeberg.org/RustAudio/nice-plug), pinned to
  [a fork](https://github.com/starzonmyarmz/nice-plug/tree/patina) that
  adds `ActivateContext::gui_context()`.
- Started from
  [plugin-template](https://github.com/starzonmyarmz/plugin-template).
