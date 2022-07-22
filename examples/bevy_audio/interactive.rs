#![allow(clippy::precedence)]

use bevy::prelude::*;
use bevy_fundsp::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DspPlugin)
        .add_startup_system(init_dsp)
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

fn init_dsp(mut dsp_manager: ResMut<DspManager>) {
    // length is in seconds
    dsp_manager
        .add_graph(sine_wave, 5.0)
        .add_graph(triangle_wave, 5.0);
}

fn interactive_audio(input: Res<Input<KeyCode>>, dsp_assets: Res<DspAssets>, audio: Res<Audio>) {
    if input.just_pressed(KeyCode::S) {
        audio.play(dsp_assets.graph(&sine_wave));
    }

    if input.just_pressed(KeyCode::T) {
        audio.play(dsp_assets.graph(&triangle_wave));
    }
}
