use std::{io::Cursor, any::TypeId};

use bevy_kira_audio::AudioSource;
use kira::sound::{
    static_sound::{StaticSoundData, StaticSoundSettings},
    FromFileError,
};

use crate::{DspSource, Settings, DspGraph, FnDspGraph, DspManager};

impl DspSource {
    /// Returns a [`StaticSoundData`].
    ///
    /// This is useful if you are using [`bevy_kira_audio`].
    ///
    /// [`StaticSoundData`]: kira::sound::static_sound::StaticSoundData
    ///
    /// # Errors
    ///
    /// This will return an error if the DSP graph cannot be parsed into a `StaticSoundData`.
    pub fn into_kira_sound_data(
        self,
        sample_rate: f64,
        settings: StaticSoundSettings,
    ) -> Result<StaticSoundData, FromFileError> {
        let raw_bytes = self.generate_raw_bytes(sample_rate);

        StaticSoundData::from_cursor(Cursor::new(raw_bytes), settings)
    }

    /// Convert this to an audio source.
    ///
    /// # Panics
    ///
    /// This can panic when `kira` is enabled and the source cannot be converted to a `StaticSoundData`
    #[must_use]
    pub fn into_audio_source(self, sample_rate: f64, settings: Settings) -> AudioSource {
        let sound = self
            .into_kira_sound_data(sample_rate, settings)
            .unwrap_or_else(|err| {
                panic!("Cannot convert DSP source to sound data. Error: {err:?}")
            });

        AudioSource { sound }
    }
}

impl DspGraph {
    /// Create a new graph from the graph function, its length in seconds, and `kira`'s [`StaticSoundSettings`].
    #[must_use]
    pub fn with_settings(func: Box<dyn FnDspGraph>, length: f64, settings: Settings) -> Self {
        Self {
            func,
            length,
            settings,
        }
    }
}

impl DspManager {
    /// Add a new graph into the manager with the given settings.
    pub fn add_graph_with_settings<F: FnDspGraph>(
        &mut self,
        f: F,
        length: f64,
        settings: Settings,
    ) -> &mut Self {
        self.graphs.insert(
            TypeId::of::<F>(),
            DspGraph::with_settings(Box::new(f), length, settings),
        );
        self
    }
}