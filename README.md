# Bevy FunDSP

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking) [![CI](https://github.com/harudagondi/bevy_fundsp/actions/workflows/rust.yml/badge.svg)](https://github.com/harudagondi/bevy_fundsp/actions/workflows/rust.yml) ![Crates.io](https://img.shields.io/crates/v/bevy_fundsp) ![Crates.io](https://img.shields.io/crates/l/bevy_fundsp) ![docs.rs](https://img.shields.io/docsrs/bevy_fundsp)

A third party Bevy plugin that integrates [FunDSP] into [Bevy].

`bevy_fundsp` supports integration for `bevy_audio` and [`bevy_kira_audio`].

[FunDSP]: https://github.com/SamiPerttu/fundsp
[Bevy]: https://github.com/bevyengine/bevy
[`bevy_kira_audio`]: https://github.com/NiklasEi/bevy_kira_audio

⚠ **WARNING: Lower your volume before testing your sounds!** ⚠

Remember to lower the volume by passing the settings with `DspManager::add_graph_with_settings`
or multiplying your DSP graph with a low constant (lower than 1.0).

## Usage

```rust no_run
#![allow(clippy::precedence)]

use {bevy::prelude::*, bevy_fundsp::prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DspPlugin::default())
        .add_dsp_source(white_noise, SourceType::Dynamic)
        .add_systems(PostStartup, play_noise)
        .run();
}

fn white_noise() -> impl AudioUnit {
    white() >> split::<U2>() * 0.2
}

fn play_noise(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
) {
    let source = assets.add(
        dsp_manager
            .get_graph(white_noise)
            .unwrap_or_else(|| panic!("DSP source not found!"))
            .clone(),
    );
    commands.spawn(AudioSourceBundle {
        source,
        ..default()
    });
}

```

## Compatibility

| `bevy_fundsp` | `bevy` | `bevy_kira_audio` | `bevy_oddio` | `fundsp` |
| ------------- | ------ | ----------------- | ------------ | -------- |
| 0.11.0        | 0.18   | 0.24              |              | 0.20     |
| 0.10.0        | 0.17   | 0.24              |              | 0.20     |
| 0.9.0         | 0.16   | 0.23              |              | 0.20     |
| 0.8.0         | 0.15   | 0.21              |              | 0.20     |
| 0.7.0         | 0.14   | 0.20              |              | 0.20     |
| 0.6.0         | 0.13   | 0.19              |              | 0.20     |
| 0.5.0         | 0.12   | 0.18              |              | 0.20     |
| 0.4.1         | 0.11   | 0.17              | 0.5          | 0.20     |
| 0.4.0         | 0.11   | 0.16              | 0.5          | 0.15     |
| 0.3.0         | 0.10   | 0.15              | 0.4          | 0.12     |
| 0.2.0         | 0.9    | 0.13              | 0.3          | 0.9      |
| 0.1.0         | 0.8    | 0.11              |              | 0.6      |

## License

`bevy_fundsp` is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgement

I'd like to say thanks to the authors of [FunDSP] and [Bevy] for making this plugin possible.

> ## Ko-fi
>
> [![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/D1D11H8FF)
