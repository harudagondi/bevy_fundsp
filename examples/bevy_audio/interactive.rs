#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DspPlugin::default())
        .add_dsp_source(sine_wave, SourceType::Static { duration: 0.5 })
        .add_dsp_source(triangle_wave, SourceType::Static { duration: 0.5 })
        .add_system(interactive_audio)
        .run();
}

fn sine_wave() -> impl AudioUnit32 {
    // Note is A4
    sine_hz(440.0) >> split::<U2>() * 0.2
}

fn triangle_wave() -> impl AudioUnit32 {
    // Note is G4
    triangle_hz(392.0) >> split::<U2>() * 0.2
}

fn interactive_audio(
    input: Res<Input<KeyCode>>,
    mut assets: ResMut<Assets<AudioSource>>,
    dsp_manager: Res<DspManager>,
    mut audio: ResMut<Audio>,
) {
    if input.just_pressed(KeyCode::S) {
        audio.play_dsp(assets.as_mut(), dsp_manager.get_graph(sine_wave).unwrap());
    }

    if input.just_pressed(KeyCode::T) {
        audio.play_dsp(
            assets.as_mut(),
            dsp_manager.get_graph(triangle_wave).unwrap(),
        );
    }
}
