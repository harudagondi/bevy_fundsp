//! Implementation to integrate `bevy_fundsp` into `bevy_oddio`.

use {
    super::{Backend, DspAudioExt},
    crate::dsp_source::{DspSource, Iter, IterMono, Source, SourceType},
    bevy::prelude::{App, Assets, Handle},
    bevy_oddio::{
        oddio::{Frames, Sample, Signal},
        output::AudioSink,
        Audio, AudioApp, AudioSource, ToSignal,
    },
    std::{cell::RefCell, rc::Rc},
};

/// The backend for `bevy_oddio`.
#[allow(clippy::module_name_repetitions)]
pub struct OddioBackend;

impl Backend for OddioBackend {
    type StaticAudioSource = AudioSource<[f32; 2]>;

    fn init_app(app: &mut App) {
        app.add_audio_source::<_, DspSource>();
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
    ) -> ExactSizeIter<impl Iterator<Item = [f32; 2]> + ExactSizeIterator> {
        let audio_unit = Rc::new(RefCell::new(self.dsp_graph.generate_graph()));
        let duration = match self.source_type {
            SourceType::Static { duration } => duration,
            SourceType::Dynamic => {
                panic!("Cannot convert dynamic DSP source into exact size iterator")
            }
        };

        let number_of_frames = (self.sample_rate * duration).round() as usize;

        let collection =
            (0..number_of_frames).map(|_| audio_unit.clone().borrow_mut().get_stereo().into());

        ExactSizeIter {
            sample_rate: self.sample_rate,
            collection: RefCell::new(collection.collect::<Vec<_>>().into_iter()),
        }
    }
}

pub(crate) struct ExactSizeIter<I>
where
    I: Iterator<Item = [f32; 2]> + ExactSizeIterator,
{
    sample_rate: f32,
    collection: RefCell<I>,
}

impl<I> Source for ExactSizeIter<I>
where
    I: Iterator<Item = [f32; 2]> + ExactSizeIterator,
{
    type Frame = [f32; 2];

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn sample(&self) -> Self::Frame {
        self.collection.borrow_mut().next().unwrap_or_default()
    }
}

impl<I> Iterator for ExactSizeIter<I>
where
    I: Iterator<Item = [f32; 2]> + ExactSizeIterator,
{
    type Item = [f32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        self.collection.borrow_mut().next()
    }
}

impl<I> ExactSizeIterator for ExactSizeIter<I>
where
    I: Iterator<Item = [f32; 2]> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.collection.borrow().len()
    }
}

impl Signal for Iter {
    type Frame = [f32; 2];

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        self.advance(interval);
        for out_frame in out {
            *out_frame = Source::sample(self);
        }
    }
}

impl Signal for IterMono {
    //  Frame must be f32 to be compatible with oddio spatial audio.
    type Frame = Sample;

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        self.advance(interval);
        for out_frame in out {
            let frame = Source::sample(self);
            *out_frame = frame;
        }
    }
}

impl DspAudioExt for Audio<[f32; 2], AudioSource<[f32; 2]>> {
    type Assets = Assets<AudioSource<[f32; 2]>>;
    type Settings = <AudioSource<[f32; 2]> as ToSignal>::Settings;
    type Sink = Handle<AudioSink<AudioSource<[f32; 2]>>>;

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

impl DspAudioExt for Audio<[f32; 2], DspSource> {
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
