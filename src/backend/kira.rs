use std::io::Cursor;

use bevy::prelude::App;
use bevy_kira_audio::{Audio, AudioChannel, AudioSource, AudioControl};
use kira::sound::{
    static_sound::{StaticSoundData, StaticSoundSettings},
    Sound, SoundData,
};

use crate::dsp_source::{DspSource, DspSourceIter, Source};

use super::{Backend, DspAudio};

impl SoundData for DspSource {
    type Error = ();
    type Handle = ();

    fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
        Ok((Box::new(self.into_iter()), ()))
    }
}

impl Sound for DspSourceIter {
    fn track(&mut self) -> kira::track::TrackId {
        kira::track::TrackId::Main
    }

    fn process(&mut self, dt: f64) -> kira::dsp::Frame {
        self.advance(dt as f32);
        let frame = self.sample();
        kira::dsp::Frame {
            left: frame[0],
            right: frame[1],
        }
    }

    fn finished(&self) -> bool {
        false
    }
}

pub struct KiraBackend;

impl Backend for KiraBackend {
    type StaticAudioSource = AudioSource;
    type DynamicAudioSource = DspSourceIter;
    type Settings = StaticSoundSettings;

    fn init_app(_app: &mut App) {}

    fn convert_to_static_audio_source(
        dsp_source: DspSource,
        settings: StaticSoundSettings,
    ) -> Self::StaticAudioSource {
        let bytes = dsp_source.to_bytes();
        let cursored = Cursor::new(bytes);
        AudioSource {
            sound: StaticSoundData::from_cursor(cursored, settings)
                .unwrap_or_else(|err| panic!("Cannot read DSP source. Error: {err}")),
        }
    }

    fn convert_to_dynamic_audio_source(dsp_source: DspSource) -> Self::DynamicAudioSource {
        dsp_source.into_iter()
    }
}

impl<Track> DspAudio for AudioChannel<Track> {
    type Sink = bevy_kira_audio::audio::PlayAudioCommand;
    type Settings = ();

    fn play_dsp_source<D: crate::dsp_graph::DspGraph>(
        &mut self,
        dsp_assets: &crate::dsp_manager::DspAssets,
        dsp_graph: D,
        _settings: Self::Settings,
    ) -> Self::Sink {
        let source = dsp_assets
            .get_graph(dsp_graph)
            .unwrap_or_else(|| panic!("DSP source not found!"));

        self.play(source)
    }
}
