use bevy_oddio::{AudioSource, oddio::Frames};

use crate::DspSource;

impl DspSource {
    /// Convert this to an audio source.
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn into_audio_source(self, sample_rate: f64) -> AudioSource {
        let mut bytes: Vec<f32> = self.generate_raw_bytes(sample_rate).into_iter().map(f32::from).collect();

        let bytes = bevy_oddio::oddio::frame_stereo(&mut bytes);

        let frames = Frames::from_slice(sample_rate as u32, bytes);

        AudioSource { frames }
    }
}