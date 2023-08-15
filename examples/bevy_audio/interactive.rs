#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*};

/// This is the most direct way to use Bevy 0.11 with bevy_audio but the
/// dsp_manager feels a little strained since you can add a DspSource directly
/// as an asset now.
///
/// I tried an experiment to avoid the .clone() in interactive_component.rs.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DspPlugin::default())
        .add_dsp_source(sine_wave, SourceType::Static { duration: 0.5 })
        .add_dsp_source(triangle_wave, SourceType::Static { duration: 0.5 })
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
    dsp_manager: Res<DspManager>,
) {
    commands.spawn((
        AudioSourceBundle {
            source: assets.add(
                dsp_manager
                    .get_graph(sine_wave)
                    .unwrap()
                    // HACK: This doesn't feel right.
                    .clone(),
            ),
            settings: PlaybackSettings {
                paused: false,
                ..default()
            },
        },
        Dsp::Sine,
    ));

    commands.spawn((
        AudioSourceBundle {
            source: assets.add(
                dsp_manager
                    .get_graph(triangle_wave)
                    .unwrap()
                    // HACK: This doesn't feel right.
                    .clone(),
            ),
            settings: PlaybackSettings {
                paused: true,
                ..default()
            },
        },
        Dsp::Triangle,
    ));
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
