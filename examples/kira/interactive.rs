#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*, bevy_kira_audio::prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .add_plugins(DspPlugin::default())
        .add_dsp_source(sine_wave, SourceType::Static { duration: 0.5 })
        .add_dsp_source(triangle_wave, SourceType::Static { duration: 0.5 })
        .add_systems(Update, interactive_audio)
        .run();
}

fn sine_wave() -> impl AudioUnit {
    // Note is A4
    sine_hz(440.0) >> split::<U2>() * 0.2
}

fn triangle_wave() -> impl AudioUnit {
    // Note is G4
    triangle_hz(392.0) >> split::<U2>() * 0.2
}

fn interactive_audio(
    input: Res<ButtonInput<KeyCode>>,
    mut assets: ResMut<Assets<AudioSource>>,
    dsp_manager: Res<DspManager>,
    audio: ResMut<Audio>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        let source = dsp_manager
            .get_graph(sine_wave)
            .unwrap_or_else(|| panic!("DSP source not found!"));
        let audio_source = DefaultBackend::convert_to_audio_source(source.clone());
        let audio_source = assets.add(audio_source);
        audio.play(audio_source);
    }

    if input.just_pressed(KeyCode::KeyT) {
        let source = dsp_manager
            .get_graph(triangle_wave)
            .unwrap_or_else(|| panic!("DSP source not found!"));
        let audio_source = DefaultBackend::convert_to_audio_source(source.clone());
        let audio_source = assets.add(audio_source);
        audio.play(audio_source);
    }
}
