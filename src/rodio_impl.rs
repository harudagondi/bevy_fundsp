use bevy::audio::AudioSource;

use crate::DspSource;

impl DspSource {
    /// Convert this to an audio source.
    #[must_use]
    pub fn into_audio_source(self) -> AudioSource {
        let bytes: std::sync::Arc<[u8]> = self.generate_raw_bytes().into();

        AudioSource { bytes }
    }
}
