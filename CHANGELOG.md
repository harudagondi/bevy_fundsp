# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0-rc.1] - 10-05-2025

- Updated fundsp to 0.20
- Updated bevy_kira_audio to 0.17

## [0.4.0] - 17-08-2023

Updated bevy and other crate dependencies.

## [0.3.0] - 14-11-2022

Updated bevy and other crate dependencies.

### Added

- Implemented `Signal` for `IterMono` (`bevy_oddio`)

## [0.2.0] - 14-11-2022

Reworked the majority of the internals.

## Added

- A way to play streaming DSP sources. See `SourceType::Dynamic`.
- You can play DSP sources using `Audio::play_dsp`.
- Two iterators on streaming audio sources: `Iter` and `IterMono`.

### Changed

- Adding the DSP plugin.
  - You must now call `DspPlugin::default()`.
- The method on adding DSP sources.
  - No more initializing using `DspAssets`!
  - Just add your DSP function using `app.add_dsp_source`
- Playing DSP sources require `Audio` to be mutable. (Use `ResMut`)
- A lot of internals are shuffled around.

### Removed

- `DspAssets`. Initialize DSP graphs using `App::add_dsp_source` instead.
- `FnDspGraph`. This is now reworked to the trait `DspGraph` and can work with any type now.
- `StreamingDspSource`. This is now reworked to `Iter` (`bevy_kira_audio` and `bevy_oddio`) and `IterMono` (`bevy_audio`). 

## [0.1.0] - 01-08-22

- New bevy plugin! `bevy_fundsp` integrates fundsp with bevy.

[Unreleased]: https://github.com/harudagondi/bevy_fundsp/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/harudagondi/bevy_fundsp/compare/v0.1.0...v0.3.0
[0.2.0]: https://github.com/harudagondi/bevy_fundsp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/harudagondi/bevy_fundsp/releases/tag/v0.1.0