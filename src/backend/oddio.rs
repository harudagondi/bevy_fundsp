//! Implementation to integrate `bevy_fundsp` into `bevy_oddio`.

use std::{cell::RefCell, rc::Rc};

use bevy::prelude::{App, Assets, Handle};
use bevy_oddio::{
    frames::{FromFrame, Stereo},
    oddio::{Controlled, Frame, Frames, Signal},
    output::AudioSink,
    Audio, AudioApp, AudioSource, ToSignal,
};
use fundsp::prelude::{AudioUnit32, Tag};

use crate::dsp_source::{DspSource, Iter, IterMono, Source, SourceType};

use super::{Backend, DspAudioExt};

/// The backend for `bevy_oddio`.
#[allow(clippy::module_name_repetitions)]
pub struct OddioBackend;

impl Backend for OddioBackend {
    type StaticAudioSource = AudioSource<Stereo>;

    fn init_app(app: &mut App) {
        app.add_audio_source::<2, _, DspSource>();
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn convert_to_audio_source(dsp_source: DspSource) -> Self::StaticAudioSource {
        let sample_rate = dsp_source.sample_rate;

        let frames = dsp_source.into_exact_size_iter();

        let frames = Frames::from_iter(sample_rate as u32, frames);

        AudioSource { frames }
    }
}

impl ToSignal for DspSource {
    type Settings = ();
    type Signal = Iter;

    fn to_signal(&self, _settings: Self::Settings) -> Self::Signal {
        self.clone().into_iter()
    }
}

impl DspSource {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub(crate) fn into_exact_size_iter(
        self,
    ) -> ExactSizeIter<impl Iterator<Item = Stereo> + ExactSizeIterator> {
        let audio_unit = Rc::new(RefCell::new(self.dsp_graph.generate_graph()));
        let duration = match self.source_type {
            SourceType::Static { duration } => duration,
            SourceType::Dynamic => {
                panic!("Cannot convert dynamic DSP source into exact size iterator")
            }
        };

        let number_of_frames = (self.sample_rate * duration).round() as usize;

        let collection = (0..number_of_frames)
            .map(|_| audio_unit.clone().borrow_mut().get_stereo())
            .map(|frame| [frame.0, frame.1])
            .map(Stereo::from);

        ExactSizeIter {
            sample_rate: self.sample_rate,
            collection: RefCell::new(collection.collect::<Vec<_>>().into_iter()),
        }
    }
}

pub(crate) struct ExactSizeIter<I>
where
    I: Iterator<Item = Stereo> + ExactSizeIterator,
{
    sample_rate: f32,
    collection: RefCell<I>,
}

impl<I> Source for ExactSizeIter<I>
where
    I: Iterator<Item = Stereo> + ExactSizeIterator,
{
    type Frame = Stereo;

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn sample(&self) -> Self::Frame {
        match self.collection.borrow_mut().next() {
            Some(frame) => frame,
            None => Stereo::ZERO,
        }
    }
}

impl<I> Iterator for ExactSizeIter<I>
where
    I: Iterator<Item = Stereo> + ExactSizeIterator,
{
    type Item = Stereo;

    fn next(&mut self) -> Option<Self::Item> {
        self.collection.borrow_mut().next()
    }
}

impl<I> ExactSizeIterator for ExactSizeIter<I>
where
    I: Iterator<Item = Stereo> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.collection.borrow().len()
    }
}

impl Signal for Iter {
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

impl DspAudioExt for Audio<Stereo, AudioSource<Stereo>> {
    type Assets = Assets<AudioSource<Stereo>>;
    type Settings = <AudioSource<Stereo> as ToSignal>::Settings;
    type Sink = Handle<AudioSink<AudioSource<Stereo>>>;

    fn play_dsp_with_settings(
        &mut self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink {
        let audio_source = OddioBackend::convert_to_audio_source(source.clone());
        let source_handle = assets.add(audio_source);
        self.play(source_handle, settings)
    }
}

impl DspAudioExt for Audio<Stereo, DspSource> {
    type Assets = Assets<DspSource>;
    type Settings = <DspSource as ToSignal>::Settings;
    type Sink = Handle<AudioSink<DspSource>>;

    fn play_dsp_with_settings(
        &mut self,
        assets: &mut Self::Assets,
        source: &DspSource,
        settings: Self::Settings,
    ) -> Self::Sink {
        let source_handle = assets.add(source.clone());
        self.play(source_handle, settings)
    }
}

/// Handle for controlling playing DSP sources.
///
/// Generally, this is used to get or set the tags of a FunDSP graph.
pub struct DspControl<'source> {
    audio_unit: &'source RefCell<Box<dyn AudioUnit32>>,
}

impl<'source> DspControl<'source> {
    pub(crate) fn new(audio_unit: &'source RefCell<Box<dyn AudioUnit32>>) -> Self {
        Self { audio_unit }
    }

    /// Query the parameter value.
    ///
    /// See more documentation in [AudioUnit32::get].
    ///
    /// [AudioUnit32::get]: fundsp::audiounit::AudioUnit32::get
    #[must_use]
    pub fn get(&self, tag: Tag) -> Option<f64> {
        self.audio_unit.borrow().get(tag)
    }

    /// Set the tag to the given value.
    ///
    /// See more documentation in [AudioUnit32::set].
    ///
    /// [AudioUnit32::set]: fundsp::audiounit::AudioUnit32::set
    pub fn set(&self, tag: Tag, value: f64) {
        self.audio_unit.borrow_mut().set(tag, value);
    }
}

pub(crate) trait Controllable<'source> {
    type Control;

    fn control(&'source self) -> Self::Control;
}

impl<'source> Controllable<'source> for Iter {
    type Control = DspControl<'source>;

    fn control(&'source self) -> Self::Control {
        DspControl::new(&self.audio_unit)
    }
}

impl<'source> Controllable<'source> for IterMono {
    type Control = DspControl<'source>;

    fn control(&'source self) -> Self::Control {
        DspControl::new(&self.0.audio_unit)
    }
}

unsafe impl<'source> Controlled<'source> for Iter {
    type Control = DspControl<'source>;

    unsafe fn make_control(signal: &'source Self) -> Self::Control {
        DspControl::new(&signal.audio_unit)
    }
}

#[cfg(test)]
mod tests {
    use fundsp::{hacker32::tag, prelude::Tag};

    use crate::{
        backend::oddio::Controllable,
        dsp_source::{DspSource, SourceType},
        DEFAULT_SAMPLE_RATE,
    };

    #[test]
    fn constant_controllable() {
        const FREQ_ID: Tag = 0;

        let sine_wave = || tag(FREQ_ID, 440.0);

        let source = DspSource::new(sine_wave, *DEFAULT_SAMPLE_RATE, SourceType::Dynamic);

        let mut iter = source.into_iter();

        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));
        assert_eq!(iter.next(), Some([440.0, 440.0]));

        iter.control().set(FREQ_ID, 880.0);

        assert_eq!(iter.next(), Some([880.0, 880.0]));
        assert_eq!(iter.next(), Some([880.0, 880.0]));
        assert_eq!(iter.next(), Some([880.0, 880.0]));

        let mut iter = iter.into_mono();

        assert_eq!(iter.next(), Some(880.0));
        assert_eq!(iter.next(), Some(880.0));
        assert_eq!(iter.next(), Some(880.0));

        iter.control().set(FREQ_ID, 440.0);

        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
        assert_eq!(iter.next(), Some(440.0));
    }
}
