//! Implementation to integrate `bevy_fundsp` into `bevy_kira_audio`.

use std::io::Cursor;

use bevy::prelude::App;
use bevy_kira_audio::AudioSource;
use kira::{
    clock::clock_info::ClockInfoProvider,
    sound::{
        static_sound::{StaticSoundData, StaticSoundSettings},
        Sound, SoundData,
    },
};

use crate::dsp_source::{DspSource, Iter, Source};

use super::Backend;

impl SoundData for DspSource {
    type Error = ();
    type Handle = ();

    fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
        Ok((Box::new(self.into_iter()), ()))
    }
}

impl Sound for Iter {
    fn track(&mut self) -> kira::track::TrackId {
        kira::track::TrackId::Main
    }

    #[allow(clippy::cast_possible_truncation)]
    fn process(&mut self, dt: f64, _: &ClockInfoProvider) -> kira::dsp::Frame {
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

/// The backend for `kira`.
#[allow(clippy::module_name_repetitions)]
pub struct KiraBackend;

impl Backend for KiraBackend {
    type StaticAudioSource = AudioSource;

    fn init_app(_app: &mut App) {}

    fn convert_to_audio_source(dsp_source: DspSource) -> Self::StaticAudioSource {
        let bytes = dsp_source.to_bytes();
        let cursored = Cursor::new(bytes);
        AudioSource {
            sound: StaticSoundData::from_cursor(cursored, StaticSoundSettings::new())
                .unwrap_or_else(|err| panic!("Cannot read DSP source. Error: {err}")),
        }
    }
}
