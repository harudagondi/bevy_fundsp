#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DspPlugin::default())
        .add_dsp_source(white_noise, SourceType::Dynamic)
        .add_startup_system(play_noise.in_base_set(StartupSet::PostStartup))
        .run();
}

fn white_noise() -> impl AudioUnit32 {
    white() >> split::<U2>() * 0.2
}

fn play_noise(
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    mut audio: ResMut<Audio<DspSource>>,
) {
    let source = dsp_manager
        .get_graph(white_noise)
        .unwrap_or_else(|| panic!("DSP source not found!"));
    audio.play_dsp(assets.as_mut(), source);
}
