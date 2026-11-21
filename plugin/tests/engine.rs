mod support;

use hahaha::engine::{Division, Engine, Mode, Settings, Transport};

const SR: f32 = 48_000.0;
const BLOCK: usize = 512;
/// Samples per beat at 120 BPM.
const BEAT: usize = 24_000;
const FADE: usize = 240;

fn playing(start: usize, tempo: f64) -> Transport {
    Transport {
        tempo: Some(tempo),
        playing: true,
        pos_beats: Some(start as f64 * tempo / 60.0 / SR as f64),
        bar_start_beats: None,
        bar_beats: Some(4.0),
    }
}

fn settings(rate: Division, mode: Mode, mix: f64) -> Settings {
    Settings { rate, mode, mix, ..Settings::default() }
}

/// Runs mono input through the engine in host-sized blocks. `control` gets
/// each block's first sample and returns the settings, the treadle and the
/// transport for it.
fn render(input: &[f32], mut control: impl FnMut(usize) -> (Settings, bool, Transport)) -> Vec<f32> {
    let mut engine = Engine::new(SR);
    engine.set(control(0).0);
    engine.reset();
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    for (i, (l, r)) in left.chunks_mut(BLOCK).zip(right.chunks_mut(BLOCK)).enumerate() {
        let (s, treadle, transport) = control(i * BLOCK);
        engine.set(s);
        engine.set_treadle(treadle);
        engine.process(l, r, &transport);
    }
    assert_eq!(left, right);
    left
}

/// Deterministic noise, so each sample says where it came from.
fn noise(len: usize) -> Vec<f32> {
    let mut state = 0x2545_f491u32;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state as f32 / u32::MAX as f32) * 1.6 - 0.8
        })
        .collect()
}

/// Asserts that `out[t]` is `input[t - delay]` over `range`.
fn assert_repeats(out: &[f32], input: &[f32], range: std::ops::Range<usize>, delay: usize) {
    for t in range {
        assert!(
            (out[t] - input[t - delay]).abs() < 1e-5,
            "sample {t}: {} isn't input[{}] = {}",
            out[t],
            t - delay,
            input[t - delay]
        );
    }
}

fn block_at(sample: usize) -> usize {
    sample.div_ceil(BLOCK) * BLOCK
}

#[test]
fn passes_input_through_until_triggered() {
    let input = noise(BEAT * 2);
    let out = render(&input, |at| (Settings::default(), false, playing(at, 120.0)));
    assert_eq!(out, input);
}

#[test]
fn slice_starts_on_the_last_grid_line() {
    let input = noise(BEAT * 3);
    let press = block_at(BEAT + BEAT / 4);
    let eighth = BEAT / 2;
    let s = settings(Division::Eighth, Mode::Hold, 100.0);
    let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));

    let first_wrap = BEAT + eighth;
    assert_eq!(out[..first_wrap - FADE], input[..first_wrap - FADE], "live until the slice is captured");
    assert_repeats(&out, &input, first_wrap..first_wrap + eighth - FADE, eighth);
    assert_repeats(&out, &input, first_wrap + eighth..first_wrap + 2 * eighth - FADE, 2 * eighth);
}

#[test]
fn falls_back_to_120_bpm_without_a_host_tempo() {
    let input = noise(BEAT * 4);
    let press = 10 * BLOCK;
    let s = settings(Division::Quarter, Mode::Hold, 100.0);
    let out = render(&input, |at| (s, at >= press, Transport::default()));

    assert_eq!(out[..press + BEAT - FADE], input[..press + BEAT - FADE]);
    assert_repeats(&out, &input, press + BEAT..press + 2 * BEAT - FADE, BEAT);
}

#[test]
fn uses_the_host_tempo_while_stopped() {
    let input = noise(BEAT * 5);
    let press = 10 * BLOCK;
    let quarter = 32_000;
    let s = settings(Division::Quarter, Mode::Hold, 100.0);
    let stopped = Transport { tempo: Some(90.0), ..Transport::default() };
    let out = render(&input, |at| (s, at >= press, stopped));

    assert_eq!(out[..press + quarter - FADE], input[..press + quarter - FADE]);
    assert_repeats(&out, &input, press + quarter..press + 2 * quarter - FADE, quarter);
}

#[test]
fn release_fades_back_to_dry() {
    let input = noise(BEAT * 4);
    let press = block_at(BEAT / 10);
    let release = block_at(2 * BEAT + BEAT / 3);
    let s = settings(Division::Eighth, Mode::Hold, 100.0);
    let out = render(&input, |at| (s, (press..release).contains(&at), playing(at, 120.0)));

    assert!(out[release - 1] != input[release - 1], "repeating before release");
    let dry = release + 480 + 1;
    assert_eq!(out[dry..], input[dry..]);
}

#[test]
fn latch_keeps_repeating_until_tapped_again() {
    let input = noise(BEAT * 5);
    let tap = |at: usize, t: usize| (t..t + BLOCK).contains(&at);
    let first = block_at(BEAT / 10);
    let second = block_at(3 * BEAT + BEAT / 3);
    let eighth = BEAT / 2;
    let s = settings(Division::Eighth, Mode::Latch, 100.0);
    let out = render(&input, |at| (s, tap(at, first) || tap(at, second), playing(at, 120.0)));

    assert_repeats(&out, &input, 2 * BEAT..2 * BEAT + eighth - FADE, 2 * BEAT);
    let dry = second + 480 + 1;
    assert_eq!(out[dry..], input[dry..]);
}

#[test]
fn rate_change_lands_at_the_next_repeat() {
    let input = noise(BEAT * 4);
    let press = block_at(BEAT + BEAT / 4);
    let change = block_at(BEAT * 7 / 4);
    let (eighth, sixteenth) = (BEAT / 2, BEAT / 4);
    let out = render(&input, |at| {
        let rate = if at >= change { Division::Sixteenth } else { Division::Eighth };
        (settings(rate, Mode::Hold, 100.0), at >= press, playing(at, 120.0))
    });

    let boundary = BEAT + 2 * eighth;
    assert_repeats(&out, &input, change..boundary - FADE, eighth);
    assert_repeats(&out, &input, boundary..boundary + sixteenth - FADE, boundary - BEAT);
    let next = boundary + sixteenth;
    assert_repeats(&out, &input, next..next + sixteenth - FADE, next - BEAT);
}

#[test]
fn longer_rate_reaches_back_before_the_slice() {
    let input = noise(BEAT * 4);
    let press = block_at(BEAT + BEAT / 10);
    let change = block_at(BEAT + BEAT / 5);
    let out = render(&input, |at| {
        let rate = if at >= change { Division::Half } else { Division::Sixteenth };
        (settings(rate, Mode::Hold, 100.0), at >= press, playing(at, 120.0))
    });

    let boundary = BEAT + BEAT / 4;
    assert_repeats(&out, &input, boundary..boundary + 2 * BEAT - FADE, boundary);
}

#[test]
fn mix_at_half_keeps_the_live_signal() {
    let input = noise(BEAT * 3);
    let press = block_at(BEAT / 10);
    let eighth = BEAT / 2;
    let s = settings(Division::Eighth, Mode::Hold, 50.0);
    let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));

    for t in eighth..2 * eighth - FADE {
        assert!((out[t] - (input[t] + input[t - eighth])).abs() < 1e-5, "sample {t}");
    }
}

#[test]
fn retrigger_grabs_a_fresh_slice() {
    let input = noise(BEAT * 5);
    let press = block_at(BEAT / 10);
    let lift = block_at(BEAT + BEAT / 3);
    let again = lift + BLOCK;
    let eighth = BEAT / 2;
    let s = settings(Division::Eighth, Mode::Hold, 100.0);
    let out = render(&input, |at| (s, at >= press && !(lift..again).contains(&at), playing(at, 120.0)));

    let wrap = BEAT + eighth;
    assert_repeats(&out, &input, wrap..wrap + eighth - FADE, eighth);
}

fn effects(rate: Division, fade: f64, bend: f64, roll: f64) -> Settings {
    Settings { rate, fade, bend, roll, ..Settings::default() }
}

#[test]
fn fade_drops_each_repeat_by_its_share_of_12_db() {
    let input = noise(BEAT * 3);
    let press = block_at(BEAT / 10);
    let eighth = BEAT / 2;
    let s = effects(Division::Eighth, 50.0, 0.0, 0.0);
    let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));

    for t in [eighth, eighth + eighth / 2, 2 * eighth, 3 * eighth + 100] {
        let progress = (t - eighth) as f64 / eighth as f64;
        let level = 10f64.powf(-6.0 * progress / 20.0) as f32;
        let source = input[t % eighth];
        assert!((out[t] - source * level).abs() < 1e-4, "sample {t}: {} against {}", out[t], source * level);
    }
}

#[test]
fn roll_halves_each_repeat_at_full() {
    let input = noise(BEAT * 3);
    let press = block_at(BEAT / 10);
    let eighth = BEAT / 2;
    let s = effects(Division::Eighth, 0.0, 0.0, 100.0);
    let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));

    let mut wrap = eighth;
    for len in [eighth, eighth / 2, eighth / 4, eighth / 8] {
        assert_repeats(&out, &input, wrap..wrap + len - FADE.min(len / 4), wrap);
        wrap += len;
    }
}

/// Zero crossings per second over `range`.
fn crossings(xs: &[f32], range: std::ops::Range<usize>) -> f64 {
    let n = xs[range.clone()].windows(2).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count();
    n as f64 * SR as f64 / range.len() as f64
}

#[test]
fn bend_drops_or_raises_the_repeats() {
    let input = support::tone(220.0, 3.0, SR);
    let press = block_at(BEAT / 10);
    let check = BEAT + BEAT / 2..2 * BEAT;
    let dry = crossings(&input, check.clone());
    for (bend, lower) in [(-100.0, true), (100.0, false)] {
        let s = effects(Division::Quarter, 0.0, bend, 0.0);
        let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));
        let bent = crossings(&out, check.clone());
        assert!(if lower { bent < dry * 0.7 } else { bent > dry * 1.4 }, "bend {bend}: {bent} against {dry}");
    }
}

#[test]
fn retrigger_resets_the_effects() {
    let input = noise(BEAT * 5);
    let press = block_at(BEAT / 10);
    let lift = block_at(2 * BEAT + BEAT / 3);
    let again = lift + BLOCK;
    let eighth = BEAT / 2;
    let s = effects(Division::Eighth, 100.0, 0.0, 50.0);
    let out = render(&input, |at| (s, at >= press && !(lift..again).contains(&at), playing(at, 120.0)));

    let wrap = 2 * BEAT + eighth;
    for t in wrap..wrap + eighth - FADE {
        let level = 10f64.powf(-12.0 * (t - wrap) as f64 / eighth as f64 / 20.0) as f32;
        assert!((out[t] - input[t - eighth] * level).abs() < 1e-4, "sample {t}");
    }
}

#[test]
fn silence_stays_silent_and_extremes_stay_finite() {
    let silence = vec![0.0; BEAT * 2];
    let out = render(&silence, |at| (Settings::default(), at >= BLOCK, playing(at, 120.0)));
    assert!(out.iter().all(|x| *x == 0.0));

    let input = noise(BEAT * 4);
    for (rate, tempo) in [(Division::ThirtySecondTriplet, 999.0), (Division::Bar, 20.0), (Division::Bar, 1.0e9)] {
        for (fade, bend, roll) in [(0.0, 0.0, 0.0), (100.0, -100.0, 100.0), (0.0, 100.0, 100.0)] {
            let s = effects(rate, fade, bend, roll);
            let out = render(&input, |at| (s, at >= BLOCK, playing(at, tempo)));
            assert!(out.iter().all(|x| x.is_finite() && x.abs() <= 2.0), "{rate:?} at {tempo} BPM, {fade}/{bend}/{roll}");
        }
    }
}

/// Two bars of 16th-note plucks at 120 BPM over a kick on every beat.
fn riff() -> Vec<f32> {
    let sixteenth = BEAT / 4;
    let notes = [110.0, 110.0, 220.0, 164.8, 130.8, 110.0, 196.0, 146.8];
    (0..8 * BEAT)
        .map(|t| {
            let step = t / sixteenth;
            let since = (t % sixteenth) as f64 / SR as f64;
            let freq = notes[step % notes.len()] * if step % 16 >= 8 { 1.5 } else { 1.0 };
            let pluck = (2.0 * std::f64::consts::PI * freq * since).sin() * (-since * 18.0).exp();
            let beat = (t % BEAT) as f64 / SR as f64;
            let kick = (2.0 * std::f64::consts::PI * (50.0 * beat + 60.0 * (1.0 - (-beat * 30.0).exp()) / 30.0)).sin() * (-beat * 12.0).exp();
            (0.4 * pluck + 0.5 * kick) as f32
        })
        .collect()
}

fn biggest_jump(xs: &[f32]) -> f32 {
    xs.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f32::max)
}

#[test]
fn wraps_and_rate_changes_dont_click() {
    let input = riff();
    let press = block_at(BEAT / 8);
    let rates = [Division::Quarter, Division::EighthTriplet, Division::Sixteenth, Division::SixteenthTriplet, Division::ThirtySecond, Division::Half];
    let out = render(&input, |at| {
        let rate = rates[(at / (BEAT * 3 / 4)) % rates.len()];
        (settings(rate, Mode::Hold, 100.0), at >= press, playing(at, 120.0))
    });
    assert!(biggest_jump(&out) <= biggest_jump(&input) * 1.1, "{} against {}", biggest_jump(&out), biggest_jump(&input));
}

#[test]
fn rolls_and_tape_stops_dont_click() {
    let input = riff();
    let press = block_at(BEAT / 8);
    for (fade, bend, roll) in [(0.0, 0.0, 100.0), (30.0, 0.0, 60.0), (0.0, -60.0, 0.0), (20.0, -100.0, 50.0)] {
        let s = effects(Division::Quarter, fade, bend, roll);
        let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));
        assert!(biggest_jump(&out) <= biggest_jump(&input) * 1.1, "{fade}/{bend}/{roll}: {} against {}", biggest_jump(&out), biggest_jump(&input));
    }
}

#[test]
fn tape_stop_matches_golden() {
    let input = riff();
    let press = block_at(2 * BEAT + BEAT / 8);
    let s = effects(Division::Half, 0.0, -50.0, 0.0);
    let out = render(&input, |at| (s, (press..7 * BEAT).contains(&at), playing(at, 120.0)));
    support::check_golden("tape-stop", &out, SR as u32);
}

#[test]
fn riser_roll_matches_golden() {
    let input = riff();
    let press = block_at(2 * BEAT + BEAT / 8);
    let s = effects(Division::Quarter, 0.0, 30.0, 60.0);
    let out = render(&input, |at| (s, (press..7 * BEAT).contains(&at), playing(at, 120.0)));
    support::check_golden("riser-roll", &out, SR as u32);
}

#[test]
fn fading_echo_matches_golden() {
    let input = riff();
    let press = block_at(BEAT + BEAT / 8);
    let s = effects(Division::EighthTriplet, 25.0, 0.0, 0.0);
    let out = render(&input, |at| (s, (press..7 * BEAT).contains(&at), playing(at, 120.0)));
    support::check_golden("fading-echo", &out, SR as u32);
}

#[test]
fn eighth_roll_matches_golden() {
    let input = riff();
    let (press, release) = (block_at(2 * BEAT + BEAT / 8), block_at(4 * BEAT));
    let s = settings(Division::Eighth, Mode::Hold, 100.0);
    let out = render(&input, |at| (s, (press..release).contains(&at), playing(at, 120.0)));
    support::check_golden("eighth-roll", &out, SR as u32);
}

#[test]
fn rate_sweep_matches_golden() {
    let input = riff();
    let press = block_at(BEAT + BEAT / 8);
    let out = render(&input, |at| {
        let rate = match at / BEAT {
            0..=2 => Division::Quarter,
            3 => Division::Eighth,
            4 => Division::SixteenthTriplet,
            _ => Division::ThirtySecond,
        };
        (settings(rate, Mode::Hold, 100.0), (press..7 * BEAT).contains(&at), playing(at, 120.0))
    });
    support::check_golden("rate-sweep", &out, SR as u32);
}

#[test]
fn half_mix_matches_golden() {
    let input = riff();
    let press = block_at(4 * BEAT);
    let s = settings(Division::QuarterTriplet, Mode::Hold, 50.0);
    let out = render(&input, |at| (s, at >= press, playing(at, 120.0)));
    support::check_golden("half-mix", &out, SR as u32);
}
