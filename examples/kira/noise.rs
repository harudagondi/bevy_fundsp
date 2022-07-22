#![allow(clippy::precedence)]

use bevy::prelude::*;
use bevy_fundsp::prelude::*;
use bevy_kira_audio::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(DspPlugin)
        .add_startup_system(init_dsp)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_noise)
        .run();
}

fn white_noise() -> impl AudioUnit32 {
    white() >> split::<U2>() * 0.2
}

fn init_dsp(mut dsp_manager: NonSendMut<DspManager>) {
    dsp_manager.add_graph(white_noise, 30.0); // length is in seconds
}

fn play_noise(dsp_assets: Res<DspAssets>, audio: Res<Audio>) {
    let white_noise = dsp_assets.get_graph(white_noise).unwrap();
    audio.play_looped(white_noise.clone());
}
