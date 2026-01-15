//! Module for the [`Backend`] trait
//! that is implemented for each audio plugin in the Bevy ecosystem.

use {
    crate::dsp_source::DspSource,
    bevy::{prelude::App, utils::default},
};

#[cfg(feature = "bevy_audio")]
pub mod bevy_audio;
#[cfg(feature = "kira")]
pub mod kira;

/// The default backend.
#[allow(clippy::module_name_repetitions)]
#[cfg(feature = "bevy_audio")]
pub type DefaultBackend = bevy_audio::BevyAudioBackend;
/// The default backend.
#[allow(clippy::module_name_repetitions)]
#[cfg(feature = "kira")]
pub type DefaultBackend = kira::KiraBackend;

/// The backend trait used to convert [`DspSource`] into its concrete type.
pub trait Backend: Send + Sync + 'static {
    /// The static audio source.
    /// Usually stores a collection of sound bytes.
    type StaticAudioSource;

    /// Initialization of App that is specific for the given Backend.
    fn init_app(app: &mut App);
    /// Convert the given [`DspSource`] to the defined static audio source.
    fn convert_to_audio_source(dsp_source: DspSource) -> Self::StaticAudioSource;
}

/// Extension trait to add a helper method for playing DSP sources.
pub trait DspAudioExt {
    /// The [`Assets`](bevy::prelude::Assets)
    /// for the concrete `Audio` type in the given backend.
    type Assets;
    /// The settings that are usually passed
    /// to the concrete `Audio` type of the given backend.
    type Settings: Default;
    /// The audio sink that is usually returned
    /// when playing the given DSP source.
    type Sink;

    /// Play the given [`DspSource`] with the given settings.
    fn play_dsp_with_settings(
        &mut self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink;

    /// Play the given [`DspSource`] with the default settings.
    fn play_dsp(&mut self, assets: &mut Self::Assets, source: &DspSource) -> Self::Sink {
        self.play_dsp_with_settings(assets, source, default())
    }
}
