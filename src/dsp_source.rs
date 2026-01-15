//! Module for [`DspSource`],
//! a type that is analogous to `AudioSource` in `bevy_audio`.

use {
    crate::dsp_graph::DspGraph,
    bevy::{asset::Asset, reflect::{TypePath}},
    fundsp::{audiounit::AudioUnit, wave::Wave},
    std::{cell::RefCell, sync::Arc},
};

/// A DSP source similar to `AudioSource` in `bevy_audio`.
///
/// These can be played directly when the [`SourceType`] is dynamic,
/// otherwise, the DSP source must be played with a given duration.
#[derive(Clone, TypePath, Asset)]
// #[uuid = "107a9069-d37d-46a8-92f2-23ec23b73bf6"]
pub struct DspSource {
    pub(crate) dsp_graph: Arc<dyn DspGraph>,
    pub(crate) sample_rate: f32,
    pub(crate) source_type: SourceType,
}

/// The type of the [`DspSource`].
#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    /// Indicates that the DSP source is static.
    /// This means that the playing sound is simply a collection of bytes.
    /// Therefore, the audio is of definite length,
    /// and the sound last for the given duration.
    ///
    /// See [`Wave`](fundsp::wave::Wave) on how this is converted.
    Static {
        /// The duration of the source in seconds.
        duration: f32,
    },
    /// Indicates that the DSP source is dynamic.
    /// This means that the playing sound last forever.
    /// Internally, each frame is computed manually,
    /// and not referenced from an internal collection of bytes.
    ///
    /// See [`Iter`].
    Dynamic,
}

impl DspSource {
    pub(crate) fn new<D: DspGraph>(
        dsp_graph: D,
        sample_rate: f32,
        source_type: SourceType,
    ) -> Self {
        Self {
            dsp_graph: Arc::new(dsp_graph),
            sample_rate,
            source_type,
        }
    }

    /// Convert the DSP source to its corresponding bytes.
    ///
    /// The source type must be static,
    /// otherwise it will panic,
    /// as it does not know how long it is.
    ///
    /// Internally, this uses [`fundsp::wave::Wave`].
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let duration = match self.source_type {
            SourceType::Static { duration } => duration,
            SourceType::Dynamic => panic!("Only static DSP sources can be converted into bytes."),
        };

        let mut node = self.dsp_graph.generate_graph();

        let wave = Wave::render(
            f64::from(self.sample_rate),
            f64::from(duration),
            node.as_mut(),
        );

        let mut buffer = Vec::new();

        wave.write_wav16(&mut buffer)
            .unwrap_or_else(|err| panic!("Cannot write wave to buffer. Error: {err:?}"));

        buffer
    }
}

impl IntoIterator for DspSource {
    type Item = [f32; 2];
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            sample_rate: self.sample_rate,
            audio_unit: RefCell::new(self.dsp_graph.generate_graph()),
        }
    }
}

/// An iterator of the DSP source
/// whose item is a stereo sample.
///
/// This is infinite, and would never return `None`.
pub struct Iter {
    pub(crate) sample_rate: f32,
    pub(crate) audio_unit: RefCell<Box<dyn AudioUnit>>,
}

pub(crate) trait Source {
    type Frame;

    fn sample_rate(&self) -> f32;
    fn sample(&self) -> Self::Frame;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn advance(&self, dt: f32) {
        for _ in 0..(self.sample_rate() * dt) as usize {
            self.sample();
        }
    }
}

impl Iter {
    /// Convert the iterator into a different iterator
    /// that returns mono samples.
    pub fn into_mono(self) -> IterMono {
        IterMono(self)
    }
}

impl Source for Iter {
    type Frame = [f32; 2];

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn sample(&self) -> Self::Frame {
        self.audio_unit.borrow_mut().get_stereo().into()
    }
}

impl Iterator for Iter {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}

/// An iterator that returns mono samples.
/// This is similar to [`Iter`].
///
/// Internally, only `bevy_audio` uses this.
pub struct IterMono(pub(crate) Iter);

impl Source for IterMono {
    type Frame = f32;

    fn sample_rate(&self) -> f32 {
        self.0.sample_rate
    }

    fn sample(&self) -> f32 {
        self.0.audio_unit.borrow_mut().get_mono()
    }
}

impl Iterator for IterMono {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]

    use {
        super::{DspSource, SourceType},
        crate::DEFAULT_SAMPLE_RATE,
        fundsp::hacker32::*,
    };

    #[test]
    fn constant_signal() {
        let source = DspSource::new(
            || constant(440.0),
            *DEFAULT_SAMPLE_RATE,
            SourceType::Dynamic,
        );

        let mut iter = source.into_iter();

        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));

        let mut iter = iter.into_mono();

        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
    }

    #[test]
    fn sine_wave_signal() {
        let sine_wave = || constant(440.0) >> sine();

        let source = DspSource::new(sine_wave, *DEFAULT_SAMPLE_RATE, SourceType::Dynamic);

        let iter = source.into_iter().into_mono();
        let mut signal = sine_wave();

        for sample in iter.take(1_000) {
            let signal_sample = signal.get_mono();
            assert!((signal_sample - sample).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn constant_controllable() {
        let frequency = shared(440.0);
        let sine_wave_frequency = frequency.clone();

        let sine_wave = move || var(&sine_wave_frequency);

        let source = DspSource::new(sine_wave, *DEFAULT_SAMPLE_RATE, SourceType::Dynamic);

        let mut iter = source.into_iter();

        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));

        frequency.set_value(880.0);

        assert_eq!(iter.next(), Some([880.0, 880.0]));
        assert_eq!(iter.next(), Some([880.0, 880.0]));
        assert_eq!(iter.next(), Some([880.0, 880.0]));

        let mut iter = iter.into_mono();

        assert_eq!(iter.next(), Some(880.0));
        assert_eq!(iter.next(), Some(880.0));
        assert_eq!(iter.next(), Some(880.0));

        frequency.set_value(440.0);

        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
    }
}
