# Design

How Ha Ha Ha looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A bubblegum-pink noise-punk pedal with a hinged treadle, screen-printed in maroon with a laughing skull that won't stop saying HA.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.

## This plugin

Settled with the user; keep to it.

- **Object:** a compact pedal shaped like a BOSS, with a knob row on top and a wide hinged treadle with a rubber pad, because the whole plate is held down to repeat. Borrow the form factor only, not BOSS colours, label strip or logo layout. It should feel like an indie-shop noise-punk pedal.
- **Palette:** see Tokens. Red is only for the LED.
- **Wordmark font:** the HAs are Shrikhand (subset to "HA"); every label is Fredoka SemiBold (subset to letters, digits, "." and "/"). Shrikhand was picked over Chewy, Rubik Wet Paint and Lacquer rendered on the treadle: it's the only one that's heavy and rounded with no sharp corners and still reads at 28 px.
- **Controls:** left to right, RATE, FADE, BEND, ROLL, MIX, as 60 px cream caps on light chrome knurled skirts spaced across a 540 px face (a first pass had 50 px knobs with dark knurls, which looked cramped and heavy). A HOLD/LATCH toggle and an LED, lit while repeating, sit above the knobs. Presets are a cream window between chrome ◀ ▶ buttons under the knobs, in maroon ink. The treadle is the trigger and sinks while repeating; it replaces the kit's `stomp`, so there's no separate bypass.
- **Version placement:** small, in the bottom-right corner of the face (`v1.0.0`).
- **Art:** a goofy, rounded laughing skull printed on the treadle in maroon with cream fills, with HA HA HA spilling from its mouth, each HA smaller or more worn than the last. It's static and doesn't react to settings or input, a deliberate exception to "art can follow parameters": the LED and the sinking treadle carry the state.
- **Ink:** illustrated noise punk. No SVG strokes: every line is a filled shape whose thickness swells and thins like brush ink, with rounded corners and ends, and a worn screen-print texture knocked out of the ink so the pink shows through.
- **Grit**, asked for after the first pass ("more grit/distress/splatter"): `#rough` displaces every edge so the ink bleeds, `#grain` knocks out fine speckle, big worn blotches and horizontal scratches, the cream layer sits a few units off register, and there are ink splatters with drips, drool from the jaw and drips off the first two HAs. Keep it at least this rough.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value. They live as custom properties at the top of `ui/index.html`.

| Token | Value | Used for |
|---|---|---|
| `--board` | `#1a1617` | the page behind the pedal |
| `--coat`, `--coat-hi`, `--coat-lo` | `#f4a6c0`, `#fbc9d9`, `#d97c9c` | the bubblegum-pink powder coat and its light and shade |
| `--ink`, `--ink-soft` | `#6b1f3a`, 62% `--ink` | the maroon print: labels and art |
| `--cream`, `--cream-hi`, `--cream-lo` | `#f6eedd`, `#fffaf0`, `#cdbf9f` | knob caps, the art's fills and the preset window |
| `--chrome-hi`, `--chrome`, `--chrome-lo` | `#fbfbf9`, `#c4c6c8`, `#6d7074` | knob skirts, the Hold/Latch toggle and the preset buttons |
| `--red`, `--red-off` | `#ff3b3b`, `#5a1a1f` | the LED only |
| `--rubber` | `#262224` | the treadle's pad |
| `--pill` | `#1a1012` | the value readout |
| `--display`, `--label` | Shrikhand, Fredoka | the HAs and every label |

## Components

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

Existing debt to clean up, not to copy.
