mod bridge;
mod editor;
pub mod engine;
mod params;

use engine::Engine;
use nice_plug::prelude::*;
use params::HaHaHaParams;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static PAGE: editor::Page = editor::Page {
    title: "Ha Ha Ha",
    size: (588, 748),
    html: include_str!("../../ui/index.html"),
    version: match option_env!("PLUGIN_VERSION") {
        Some(version) => version,
        None => env!("CARGO_PKG_VERSION"),
    },
    background: (27, 26, 25, 255),
    dev_url_var: "HAHAHA_UI_URL",
};

pub struct HaHaHa {
    params: Arc<HaHaHaParams>,
    engine: Engine,
    engaged: Arc<AtomicBool>,
}

impl Default for HaHaHa {
    fn default() -> Self {
        Self {
            params: Arc::new(HaHaHaParams::default()),
            engine: Engine::new(48_000.0),
            engaged: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn transport(t: &nice_plug::context::process::Transport) -> engine::Transport {
    let bar_beats = match (t.time_sig_numerator, t.time_sig_denominator) {
        (Some(n), Some(d)) if n > 0 && d > 0 => Some(n as f64 * 4.0 / d as f64),
        _ => None,
    };
    engine::Transport {
        tempo: t.tempo,
        playing: t.playing,
        pos_beats: t.pos_beats(),
        bar_start_beats: t.bar_start_pos_beats(),
        bar_beats,
    }
}

impl Plugin for HaHaHa {
    const NAME: &'static str = "Ha Ha Ha";
    const VENDOR: &'static str = "Daniel Marino";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type Editor = editor::WebEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        Some(editor::WebEditor::new(&PAGE, self.params.clone(), vec![("engaged", self.engaged.clone())]))
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        if self.engine.sample_rate() != buffer_config.sample_rate {
            self.engine = Engine::new(buffer_config.sample_rate);
        }
        self.engine.set(self.params.settings());
        self.engine.reset();
        true
    }

    fn reset(&mut self) {
        self.engine.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set(self.params.settings());
        self.engine.set_treadle(self.params.repeat.value());
        let transport = transport(context.transport());
        if let [left, right] = buffer.as_slice() {
            self.engine.process(left, right, &transport);
        }
        self.engaged.store(self.engine.engaged(), Ordering::Relaxed);
        ProcessStatus::Normal
    }
}

impl ClapPlugin for HaHaHa {
    const CLAP_ID: &'static str = "com.iamdanielmarino.hahaha";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A tempo-synced repeater");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Glitch, ClapFeature::Stereo];
}

impl Vst3Plugin for HaHaHa {
    const VST3_CLASS_ID: [u8; 16] = *b"HaHaHaDanielMari";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}

nice_export_clap!(HaHaHa);
nice_export_vst3!(HaHaHa);
