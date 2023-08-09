#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*};

/// This is an experiment to try and avoid the .clone() that is happening in
/// interactive.rs.
///
/// I'd prefer keeping .add_dsp_source(). Maybe an idea for moving forward would be
/// to have DspManager keep handles to the assets?
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DspPlugin::default())
        // .add_dsp_source(sine_wave, SourceType::Static { duration: 0.5 })
        // .add_dsp_source(triangle_wave, SourceType::Static { duration: 0.5 })
        .add_systems(Startup, setup)
        .add_systems(Update, interactive_audio)
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

#[derive(Component, Clone, Copy, PartialEq)]
enum Dsp {
    Sine,
    Triangle,
}

fn setup(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
) {
    let sample_rate = 44_100.0; // This should come from somewhere else.
    commands.spawn(
        (AudioSourceBundle {
            source: assets.add(DspSource::new(sine_wave, sample_rate, SourceType::Static { duration: 0.5 })),
            settings: PlaybackSettings {
                paused: false,
                ..default()
            }
        },
        Dsp::Sine));

    commands.spawn(
        (AudioSourceBundle {
            source: assets.add(DspSource::new(triangle_wave, sample_rate, SourceType::Static { duration: 0.5 })),
            settings: PlaybackSettings {
                paused: true,
                ..default()
            }
        },
        Dsp::Triangle));
}

fn interactive_audio(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut AudioSink, &Dsp)>,
) {
    if input.just_pressed(KeyCode::S) {
        for (sink, _) in query.iter_mut().filter(|(_s, d)| **d == Dsp::Sine) {
            sink.toggle();
        }
    }

    if input.just_pressed(KeyCode::T) {
        for (sink, _) in query.iter_mut().filter(|(_s, d)| **d == Dsp::Triangle) {
            sink.toggle();
        }
    }
}
