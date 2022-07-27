use bevy_oddio::{oddio::Frames, AudioSource};

use crate::{DspSource, StreamingDspSource};

impl DspSource {
    /// Convert this to an audio source.
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn into_audio_source(self, sample_rate: f64) -> AudioSource {
        let mut bytes: Vec<f32> = self
            .generate_raw_bytes()
            .into_iter()
            .map(f32::from)
            .collect();

        let bytes = bevy_oddio::oddio::frame_stereo(&mut bytes);

        let frames = Frames::from_slice(sample_rate as u32, bytes);

        AudioSource { frames }
    }
}

impl bevy_oddio::oddio::Signal for StreamingDspSource {
    type Frame = bevy_oddio::Stereo;

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_lossless
    )]
    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        for _ in 0..(interval as f64 * self.sample_rate) as usize {
            self.next_frame();
        }

        for o in out {
            let (left, right) = self.next_frame();
            o[0] = left;
            o[1] = right;
        }
    }
}
