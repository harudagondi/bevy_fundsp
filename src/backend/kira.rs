//! Implementation to integrate `bevy_fundsp` into `bevy_kira_audio`.

use {
    super::Backend,
    crate::dsp_source::{DspSource, Iter, Source},
    bevy::prelude::App,
    bevy_kira_audio::AudioSource,
    kira::{
        Frame, info::Info, sound::{
            Sound, SoundData, static_sound::StaticSoundData
        }
        // OutputDestination,
    },
    std::io::Cursor,
};

impl SoundData for DspSource {
    type Error = ();
    type Handle = ();

    fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
        Ok((Box::new(self.into_iter()), ()))
    }
}

impl Sound for Iter {
    // fn output_destination(&mut self) -> OutputDestination {
    //     OutputDestination::Track(kira::track::TrackId::Main)
    // }

    #[allow(clippy::cast_possible_truncation)]
    fn process(
        &mut self,
        out: &mut [Frame],
        dt: f64,
        _: &Info
    ) {
        self.advance(dt as f32);
        let frame = self.sample();
        out[0].left = frame[0];
        out[0].right = frame[1];
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
            sound: StaticSoundData::from_cursor(cursored)
                .unwrap_or_else(|err| panic!("Cannot read DSP source. Error: {err}")),
        }
    }
}
