use bevy::prelude::App;
use bevy_oddio::{AudioSource, frames::{Stereo, FromFrame}, oddio::Signal, AudioApp};

use crate::dsp_source::{DspSourceIter, Source};

use super::Backend;

pub struct OddioBackend;

impl Signal for DspSourceIter {
    type Frame = Stereo;

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        self.advance(interval);
        for out_frame in out {
            let frame = Source::sample(self);
            let stereo: Stereo = FromFrame::from_frame(frame);
            *out_frame = stereo;
        }
    }
}

impl Backend for OddioBackend {
    type StaticAudioSource = AudioSource<Stereo>;
    type DynamicAudioSource = DspSourceIter;
    type Settings = ();

    fn init_app(app: &mut App) {
        app.add_audio_source()
    }

    fn convert_to_static_audio_source(
        dsp_source: crate::dsp_source::DspSource,
        settings: Self::Settings,
    ) -> Self::StaticAudioSource {
        todo!()
    }

    fn convert_to_dynamic_audio_source(dsp_source: crate::dsp_source::DspSource) -> Self::DynamicAudioSource {
        todo!()
    }
}