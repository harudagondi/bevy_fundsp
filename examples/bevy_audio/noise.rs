#![allow(clippy::precedence)]

use bevy::prelude::*;
use bevy_fundsp::{
    backend::{DefaultBackend, DspAudioExt},
    dsp_manager::DspManager,
    dsp_source::{DspSource, SourceType},
    DspAppExt, DspPlugin,
};
use fundsp::hacker32::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DspPlugin::<DefaultBackend>::default())
        .add_dsp_source(white_noise, SourceType::Dynamic)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_noise)
        .run();
}

fn white_noise() -> impl AudioUnit32 {
    white() >> split::<U2>() * 0.2
}

fn play_noise(
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    audio: Res<Audio<DspSource>>,
) {
    let source = dsp_manager
        .get_graph(white_noise)
        .unwrap_or_else(|| panic!("DSP source not found!"));
    DspAudioExt::<DefaultBackend>::play_dsp(audio.as_ref(), assets.as_mut(), source);
}
