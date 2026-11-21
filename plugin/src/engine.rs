use std::f64::consts::FRAC_PI_2;

/// The longest slice, so a bar at 30 BPM in 4/4 still fits.
const MAX_SLICE_SECONDS: f64 = 8.0;
const WRAP_FADE_MS: f64 = 5.0;
const RELEASE_MS: f64 = 10.0;
const MIX_SMOOTH_MS: f64 = 20.0;
const FALLBACK_TEMPO: f64 = 120.0;

/// RATE: how long each repeated slice is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Division {
    Bar,
    Half,
    HalfTriplet,
    Quarter,
    QuarterTriplet,
    Eighth,
    EighthTriplet,
    Sixteenth,
    SixteenthTriplet,
    ThirtySecond,
    ThirtySecondTriplet,
}

impl Division {
    /// Length in quarter-note beats.
    pub fn beats(self, bar_beats: f64) -> f64 {
        match self {
            Self::Bar => bar_beats,
            Self::Half => 2.0,
            Self::HalfTriplet => 4.0 / 3.0,
            Self::Quarter => 1.0,
            Self::QuarterTriplet => 2.0 / 3.0,
            Self::Eighth => 0.5,
            Self::EighthTriplet => 1.0 / 3.0,
            Self::Sixteenth => 0.25,
            Self::SixteenthTriplet => 1.0 / 6.0,
            Self::ThirtySecond => 0.125,
            Self::ThirtySecondTriplet => 1.0 / 12.0,
        }
    }
}

/// How the treadle triggers: Hold repeats while it's down, Latch toggles
/// on each press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Hold,
    Latch,
}

/// The engine's raw values, in the units the controls show.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub rate: Division,
    pub mode: Mode,
    /// Live signal against repeats, 0 to 100%. Both are at full level at
    /// 50%, and 100% cuts the live signal while repeating.
    pub mix: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { rate: Division::Eighth, mode: Mode::Hold, mix: 100.0 }
    }
}

/// The host's transport at the start of a block.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transport {
    pub tempo: Option<f64>,
    pub playing: bool,
    pub pos_beats: Option<f64>,
    pub bar_start_beats: Option<f64>,
    /// Quarter-note beats per bar, from the time signature.
    pub bar_beats: Option<f64>,
}

/// Where a slice starts, worked out when the treadle goes down.
#[derive(Clone, Copy, Debug)]
struct Start {
    at: i64,
    /// The grid line it starts on, in beats, when the host is playing.
    beats: Option<f64>,
    len: i64,
}

/// A repeat in progress. Positions are absolute sample times; `slice`
/// holds the audio from `base` on.
#[derive(Clone, Copy, Debug)]
struct Burst {
    origin: Start,
    base: i64,
    start: i64,
    len: i64,
    pos: i64,
    passes: u32,
    fade: i64,
    /// The next pass's start and length, settled when its pre-roll
    /// begins `fade` samples before the wrap. The pre-roll is the audio
    /// just before that start, so each pass's attack lands on the grid
    /// whole.
    next: Option<(i64, i64)>,
}

impl Burst {
    fn in_preroll(&self) -> bool {
        self.pos >= self.len - self.fade
    }
}

/// The plugin's DSP, apart from the host plumbing so tests can run it
/// offline. The plugin passes in [`Settings`], the treadle and the
/// transport once per block.
pub struct Engine {
    sample_rate: f32,
    sr: f64,
    max_len: i64,
    wrap_fade: i64,
    history: Vec<[f32; 2]>,
    slice: Vec<[f32; 2]>,
    now: i64,

    settings: Settings,
    treadle: bool,
    pressed: bool,
    latched: bool,

    burst: Option<Burst>,
    pending: Option<Start>,
    env: f64,
    env_up: f64,
    env_down: f64,
    dry: f64,
    wet: f64,
    mix_coef: f64,

    len: i64,
    len_beats: f64,
    samples_per_beat: f64,
    on_grid: bool,
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let max_len = (MAX_SLICE_SECONDS * sr).round() as i64;
        let wrap_fade = ((WRAP_FADE_MS * sr / 1000.0).round() as i64).max(1);
        let history_len = 2 * max_len + sr.ceil() as i64;
        let slice_len = 2 * max_len + wrap_fade + 2;
        let mut engine = Self {
            sample_rate,
            sr,
            max_len,
            wrap_fade,
            history: vec![[0.0; 2]; history_len as usize],
            slice: vec![[0.0; 2]; slice_len as usize],
            now: 0,
            settings: Settings::default(),
            treadle: false,
            pressed: false,
            latched: false,
            burst: None,
            pending: None,
            env: 0.0,
            env_up: 1.0 / wrap_fade as f64,
            env_down: 1000.0 / (RELEASE_MS * sr),
            dry: 1.0,
            wet: 1.0,
            mix_coef: 1.0 - (-1000.0 / (MIX_SMOOTH_MS * sr)).exp(),
            len: 0,
            len_beats: 0.0,
            samples_per_beat: 0.0,
            on_grid: false,
        };
        engine.reset();
        engine
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn reset(&mut self) {
        self.history.fill([0.0; 2]);
        self.now = 0;
        self.treadle = false;
        self.pressed = false;
        self.latched = false;
        self.burst = None;
        self.pending = None;
        self.env = 0.0;
        let (dry, wet) = mix_gains(self.settings.mix);
        self.dry = dry;
        self.wet = wet;
    }

    pub fn set(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn set_treadle(&mut self, down: bool) {
        if down && !self.treadle {
            self.pressed = true;
        }
        self.treadle = down;
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], transport: &Transport) {
        self.begin_block(transport);
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let [ol, or] = self.tick([*l, *r]);
            *l = ol;
            *r = or;
        }
    }

    fn want(&self) -> bool {
        match self.settings.mode {
            Mode::Hold => self.treadle,
            Mode::Latch => self.latched,
        }
    }

    fn begin_block(&mut self, transport: &Transport) {
        let tempo = transport.tempo.filter(|t| t.is_finite() && *t > 0.0).unwrap_or(FALLBACK_TEMPO).clamp(20.0, 999.0);
        let bar_beats = transport.bar_beats.filter(|b| b.is_finite() && *b > 0.0).unwrap_or(4.0);
        self.samples_per_beat = self.sr * 60.0 / tempo;
        self.len_beats = self.settings.rate.beats(bar_beats);
        self.len = ((self.len_beats * self.samples_per_beat).round() as i64).clamp(4 * self.wrap_fade, self.max_len);

        let pressed = std::mem::take(&mut self.pressed);
        match self.settings.mode {
            Mode::Hold => self.latched = false,
            Mode::Latch if pressed => self.latched = !self.latched,
            Mode::Latch => {}
        }
        let want = self.want();

        let grid = match (transport.playing, transport.tempo, transport.pos_beats) {
            (true, Some(_), Some(beats)) if beats.is_finite() => Some((beats, transport.bar_start_beats.unwrap_or(0.0))),
            _ => None,
        };
        self.on_grid = grid.is_some();

        if want && (pressed || (self.burst.is_none() && self.pending.is_none())) {
            let start = match grid {
                Some((beats, bar_start)) => {
                    let offset = (beats - bar_start).rem_euclid(self.len_beats);
                    let back = ((offset * self.samples_per_beat).round() as i64).min(self.len - 1);
                    Start { at: self.now - back, beats: Some(beats - offset), len: self.len }
                }
                None => Start { at: self.now, beats: None, len: self.len },
            };
            if self.burst.is_some() && self.env > 0.0 {
                self.pending = Some(start);
            } else {
                self.begin_burst(start);
            }
        }
    }

    fn begin_burst(&mut self, origin: Start) {
        let base = origin.at - self.max_len;
        for at in base..self.now {
            self.slice[(at - base) as usize] = self.history[self.history_index(at)];
        }
        let len = origin.len;
        self.burst = Some(Burst {
            origin,
            base,
            start: origin.at,
            len,
            pos: (self.now - origin.at) % len,
            passes: 0,
            fade: self.wrap_fade.min(len / 4),
            next: None,
        });
        self.settle_next();
    }

    fn settle_next(&mut self) {
        let Some(b) = self.burst else { return };
        if b.next.is_some() || !b.in_preroll() {
            return;
        }
        let len = self.len;
        let start = if len == b.len { b.start } else { self.restart(&b.origin, len) };
        let last = b.base + self.slice.len() as i64 - len - self.wrap_fade - 1;
        let start = start.clamp(b.base + self.wrap_fade, last.min(self.now));
        self.burst.as_mut().unwrap().next = Some((start, len));
    }

    fn history_index(&self, at: i64) -> usize {
        at.rem_euclid(self.history.len() as i64) as usize
    }

    /// Where a slice of `len` samples starts after RATE changes mid-burst.
    /// On the grid it's the grid line at or before the original start, so
    /// the repeat stays in time; off the grid a shorter slice keeps the
    /// start and a longer one keeps the end.
    fn restart(&self, origin: &Start, len: i64) -> i64 {
        match origin.beats {
            Some(beats) if self.on_grid => {
                let snapped = (beats / self.len_beats).floor() * self.len_beats;
                origin.at - ((beats - snapped) * self.samples_per_beat).round() as i64
            }
            _ if len <= origin.len => origin.at,
            _ => origin.at + origin.len - len,
        }
    }

    fn tick(&mut self, input: [f32; 2]) -> [f32; 2] {
        let index = self.history_index(self.now);
        self.history[index] = input;

        let want = self.want();
        let mut repeat = [0.0f64; 2];
        let mut heard = false;
        if let Some(b) = &self.burst {
            let at = (self.now - b.base) as usize;
            if at < self.slice.len() {
                self.slice[at] = input;
            }
            let current = (b.passes > 0).then(|| self.slice[(b.start + b.pos - b.base) as usize]);
            let preroll = b.next.filter(|_| b.in_preroll()).map(|(start, _)| self.slice[(start + b.pos - b.len - b.base) as usize]);
            heard = current.is_some() || preroll.is_some();
            repeat = match (current, preroll) {
                (Some(c), Some(p)) => {
                    let w = (b.pos - (b.len - b.fade)) as f64 / b.fade as f64;
                    [0, 1].map(|ch| c[ch] as f64 * (1.0 - w) + p[ch] as f64 * w)
                }
                (Some(x), None) | (None, Some(x)) => [x[0] as f64, x[1] as f64],
                (None, None) => [0.0; 2],
            };
        }

        let target = if heard && want && self.pending.is_none() { 1.0 } else { 0.0 };
        if self.env < target {
            self.env = (self.env + self.env_up).min(1.0);
        } else if self.env > target {
            self.env = (self.env - self.env_down).max(0.0);
        }

        let (dry, wet) = mix_gains(self.settings.mix);
        self.dry = flush(self.dry + (dry - self.dry) * self.mix_coef);
        self.wet = flush(self.wet + (wet - self.wet) * self.mix_coef);

        let output = if self.env > 0.0 {
            let x = self.env * FRAC_PI_2;
            let live_gain = 1.0 - (1.0 - self.dry) * (1.0 - x.cos());
            let repeat_gain = self.wet * x.sin();
            [0, 1].map(|ch| flush(input[ch] as f64 * live_gain + repeat[ch] * repeat_gain) as f32)
        } else {
            input
        };

        self.advance();
        self.now += 1;
        self.settle(want);
        output
    }

    fn advance(&mut self) {
        let wrap_fade = self.wrap_fade;
        if let Some(b) = &mut self.burst {
            b.pos += 1;
            if b.pos >= b.len {
                (b.start, b.len) = b.next.take().unwrap_or((b.start, b.len));
                b.passes += 1;
                b.pos = 0;
                b.fade = wrap_fade.min(b.len / 4);
            }
        }
        self.settle_next();
    }

    /// Once the repeats have faded out, starts a pending retrigger or
    /// goes idle.
    fn settle(&mut self, want: bool) {
        if self.env > 0.0 {
            return;
        }
        if let Some(start) = self.pending.take() {
            self.begin_burst(start);
        } else if !want {
            self.burst = None;
        }
    }
}

/// Live and repeat gains for MIX: the live signal is at full level up to
/// 50% and cut at 100%, the repeats the other way round.
fn mix_gains(mix: f64) -> (f64, f64) {
    let m = (mix / 100.0).clamp(0.0, 1.0);
    ((2.0 * (1.0 - m)).min(1.0), (2.0 * m).min(1.0))
}

fn flush(x: f64) -> f64 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}
