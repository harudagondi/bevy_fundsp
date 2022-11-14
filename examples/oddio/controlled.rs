#![allow(clippy::precedence)]

use bevy::prelude::*;
use bevy_fundsp::{dsp_source, prelude::*};
use bevy_oddio::{frames::Stereo, output::AudioSink, Audio, AudioPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(DspPlugin::default())
        .add_dsp_source(sine_wave, SourceType::Dynamic)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_sine)
        .add_system(change_pitch)
        .run();
}

const FREQUENCY_ID: i64 = 0;

fn sine_wave() -> impl AudioUnit32 {
    tag(FREQUENCY_ID, 440.0) >> sine() >> split::<U2>() * 0.2
}

#[derive(Resource)]
struct SineSink(Handle<AudioSink<DspSource>>);

fn play_sine(
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    mut audio: ResMut<Audio<Stereo, DspSource>>,
    mut commands: Commands,
) {
    let source = dsp_manager
        .get_graph(sine_wave)
        .unwrap_or_else(|| panic!("DSP source not found!"));
    let sink = audio.play_dsp(assets.as_mut(), source);
    commands.insert_resource(SineSink(sink));
}

fn change_pitch(
    sink: Res<SineSink>,
    mut assets: ResMut<Assets<AudioSink<DspSource>>>,
    time: Res<Time>,
) {
    if let Some(sink) = assets.get_mut(&sink.0) {
        let exp = time.elapsed_seconds_wrapped_f64().sin();
        let pitch_hz = 2.0.pow(exp) * 440.0;

        sink.control::<dsp_source::Iter, _>()
            .set(FREQUENCY_ID, pitch_hz);
    }
}
