use bevy::{prelude::App, utils::default};

use crate::dsp_source::DspSource;

#[cfg(feature = "bevy_audio")]
pub mod bevy_audio;
// #[cfg(feature = "kira")]
// pub mod kira;
// #[cfg(feature = "oddio")]
// pub mod oddio;

#[cfg(feature = "bevy_audio")]
pub type DefaultBackend = bevy_audio::BevyAudioBackend;
// #[cfg(feature = "kira")]
// pub type DefaultBackend = kira::KiraBackend;
// #[cfg(feature = "oddio")]
// pub type DefaultBackend = oddio::OddioBackend;

pub trait Backend: Send + Sync + 'static {
    type StaticAudioSource;
    type DynamicAudioSource;

    fn init_app(app: &mut App);
    fn convert_to_static_audio_source(dsp_source: DspSource) -> Self::StaticAudioSource;
    fn convert_to_dynamic_audio_source(dsp_source: DspSource) -> Self::DynamicAudioSource;
}

pub trait DspAudioExt<B: Backend> {
    type Assets;
    type Settings: Default;
    type Sink;

    fn play_dsp_with_settings(
        &self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink;

    fn play_dsp(&self, assets: &mut Self::Assets, source: &DspSource) -> Self::Sink {
        self.play_dsp_with_settings(assets, source, default())
    }
}
