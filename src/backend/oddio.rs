//! Implementation to integrate `bevy_fundsp` into `bevy_oddio`.

use std::{cell::RefCell, rc::Rc};

use bevy::prelude::{App, Assets, Handle};
use bevy_oddio::{
    frames::{FromFrame, Stereo},
    oddio::{Frame, Frames, Signal},
    output::AudioSink,
    Audio, AudioApp, AudioSource, ToSignal,
};

use crate::dsp_source::{DspSource, Iter, Source, SourceType};

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
