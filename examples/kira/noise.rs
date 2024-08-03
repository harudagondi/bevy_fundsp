#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*, bevy_kira_audio::prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .add_plugins(DspPlugin::default())
        .add_dsp_source(white_noise, SourceType::Static { duration: 60.0 })
        .add_systems(Startup, play_noise)
        .run();
}

fn white_noise() -> impl AudioUnit {
    white() >> split::<U2>() * 0.2
}

fn play_noise(
    mut assets: ResMut<Assets<AudioSource>>,
    dsp_manager: Res<DspManager>,
    audio: Res<Audio>,
) {
    let source = dsp_manager
        .get_graph(white_noise)
        .unwrap_or_else(|| panic!("DSP source not found!"));
    let audio_source = DefaultBackend::convert_to_audio_source(source.clone());
    let audio_source = assets.add(audio_source);
    audio.play(audio_source);
}
