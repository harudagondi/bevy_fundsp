# Bevy FunDSP

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking) [![Continuous integration](https://github.com/harudagondi/bevy_fundsp/actions/workflows/rust.yml/badge.svg)](https://github.com/harudagondi/bevy_fundsp/actions/workflows/rust.yml)

A third party Bevy plugin that integrates [FunDSP] into [Bevy]. This requires [`bevy_kira_audio`]. 

[FunDSP]: https://github.com/SamiPerttu/fundsp
[Bevy]: https://github.com/bevyengine/bevy
[`bevy_kira_audio`]: https://github.com/NiklasEi/bevy_kira_audio

⚠ **WARNING: Lower your volume before testing your sounds!** ⚠

Remember to lower the volume by passing the settings with `DspManager::add_graph_with_settings`
or multiplying your DSP graph with a low constant (lower than 1.0).

Currently this does not integrate with `bevy_audio`.

## Usage

```rust no_run
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
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            play_noise
        )
        .run();
}

fn white_noise() -> impl AudioUnit32 {
    white() >> split::<U2>() * 0.2
}

fn init_dsp(mut dsp_manager: ResMut<DspManager>) {
    dsp_manager.add_graph(white_noise, 30.0); // length is in seconds
}

fn play_noise(dsp_assets: Res<DspAssets>, audio: Res<Audio>) {
    let white_noise = dsp_assets.graph(&white_noise);
    audio.play_looped(white_noise.clone());
}

```

## Compatibility

| `bevy_fundsp` | `bevy` | `bevy_kira_audio`          | `fundsp` |
| ------------- | ------ | -------------------------- | -------- |
| main          | main   | main, branch = `bevy_main` | main     |

## License

`bevy_fundsp` is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgement

I'd like to say thanks to the authors of [FunDSP] and [Bevy] for making this plugin possible.