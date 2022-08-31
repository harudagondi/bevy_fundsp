//! Implementation to integrate `bevy_fundsp` into `bevy_audio`.

use crate::dsp_source::{DspSource, IterMono};
use bevy::{
    audio::{play_queued_audio_system, AudioSink},
    prelude::{
        App, Assets, Audio, AudioOutput, AudioSource, CoreStage, Decodable, Handle,
        PlaybackSettings,
    },
};
use std::any::Any;

use super::{Backend, DspAudioExt};

/// The backend for `bevy_audio`.
#[allow(clippy::module_name_repetitions)]
pub struct BevyAudioBackend;

impl Decodable for DspSource {
    type Decoder = IterMono;
    type DecoderItem = f32;

    fn decoder(&self) -> Self::Decoder {
        self.clone().into_iter().into_mono()
    }
}

impl rodio::Source for IterMono {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn sample_rate(&self) -> u32 {
        self.0.sample_rate as u32
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Backend for BevyAudioBackend {
    type StaticAudioSource = AudioSource;

    fn init_app(app: &mut App) {
        app.init_resource::<Audio<DspSource>>()
            .init_non_send_resource::<AudioOutput<DspSource>>()
            .add_system_to_stage(CoreStage::PostUpdate, play_queued_audio_system::<DspSource>);
    }

    fn convert_to_audio_source(
        dsp_source: crate::dsp_source::DspSource,
    ) -> Self::StaticAudioSource {
        let bytes = dsp_source.to_bytes().into();

        AudioSource { bytes }
    }
}

impl DspAudioExt for Audio<AudioSource> {
    type Assets = Assets<AudioSource>;
    type Settings = PlaybackSettings;
    type Sink = Handle<AudioSink>;

    fn play_dsp_with_settings(
        &self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink {
        let audio = BevyAudioBackend::convert_to_audio_source(source.clone());
        let audio: &AudioSource = <dyn Any>::downcast_ref(&audio)
            .unwrap_or_else(|| panic!("Cannot downcast static audio source"));
        let handle = assets.add(audio.clone());
        let settings: &PlaybackSettings = <dyn Any>::downcast_ref(&settings)
            .unwrap_or_else(|| panic!("Cannot downcast playback settings"));
        self.play_with_settings(handle, settings.clone())
    }
}

impl DspAudioExt for Audio<DspSource> {
    type Assets = Assets<DspSource>;
    type Settings = PlaybackSettings;
    type Sink = Handle<AudioSink>;

    fn play_dsp_with_settings(
        &self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink {
        let handle = assets.add(source.clone());
        self.play_with_settings(handle, settings)
    }
}
