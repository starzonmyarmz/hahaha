use crate::engine::{Division, Mode, Settings};
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rate {
    #[name = "1 bar"]
    Bar,
    #[name = "1/2"]
    Half,
    #[name = "1/2T"]
    HalfTriplet,
    #[name = "1/4"]
    Quarter,
    #[name = "1/4T"]
    QuarterTriplet,
    #[name = "1/8"]
    Eighth,
    #[name = "1/8T"]
    EighthTriplet,
    #[name = "1/16"]
    Sixteenth,
    #[name = "1/16T"]
    SixteenthTriplet,
    #[name = "1/32"]
    ThirtySecond,
    #[name = "1/32T"]
    ThirtySecondTriplet,
}

impl From<Rate> for Division {
    fn from(rate: Rate) -> Self {
        match rate {
            Rate::Bar => Self::Bar,
            Rate::Half => Self::Half,
            Rate::HalfTriplet => Self::HalfTriplet,
            Rate::Quarter => Self::Quarter,
            Rate::QuarterTriplet => Self::QuarterTriplet,
            Rate::Eighth => Self::Eighth,
            Rate::EighthTriplet => Self::EighthTriplet,
            Rate::Sixteenth => Self::Sixteenth,
            Rate::SixteenthTriplet => Self::SixteenthTriplet,
            Rate::ThirtySecond => Self::ThirtySecond,
            Rate::ThirtySecondTriplet => Self::ThirtySecondTriplet,
        }
    }
}

#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Hold,
    Latch,
}

impl From<Trigger> for Mode {
    fn from(trigger: Trigger) -> Self {
        match trigger {
            Trigger::Hold => Self::Hold,
            Trigger::Latch => Self::Latch,
        }
    }
}

#[derive(Params)]
pub struct HaHaHaParams {
    #[id = "rate"]
    pub rate: EnumParam<Rate>,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "trigger"]
    pub trigger: EnumParam<Trigger>,
    /// The treadle. Holding it (or tapping it in Latch) repeats.
    #[id = "repeat"]
    pub repeat: BoolParam,
}

impl Default for HaHaHaParams {
    fn default() -> Self {
        let s = Settings::default();
        Self {
            rate: EnumParam::new("Rate", Rate::Eighth),
            mix: FloatParam::new("Mix", s.mix as f32, FloatRange::Linear { min: 0.0, max: 100.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_rounded(0)),
            trigger: EnumParam::new("Trigger", Trigger::Hold),
            repeat: BoolParam::new("Repeat", false),
        }
    }
}

impl HaHaHaParams {
    pub fn settings(&self) -> Settings {
        Settings { rate: self.rate.value().into(), mode: self.trigger.value().into(), mix: self.mix.value() as f64 }
    }
}
