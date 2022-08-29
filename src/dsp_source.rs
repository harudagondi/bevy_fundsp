use crate::dsp_data::DspGraph;
use bevy::reflect::TypeUuid;
use fundsp::{hacker32::AudioUnit32, wave::Wave32};
use std::{cell::RefCell, sync::Arc};

#[derive(TypeUuid, Clone)]
#[uuid = "107a9069-d37d-46a8-92f2-23ec23b73bf6"]
pub struct DspSource {
    pub(crate) dsp_data: Arc<dyn DspGraph>,
    pub(crate) sample_rate: f32,
    pub(crate) source_type: SourceType,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Static { duration: f32 },
    Dynamic,
}

impl DspSource {
    pub(crate) fn new<D: DspGraph>(dsp_data: D, sample_rate: f32, source_type: SourceType) -> Self {
        Self {
            dsp_data: Arc::new(dsp_data),
            sample_rate,
            source_type,
        }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let duration = match self.source_type {
            SourceType::Static { duration } => duration,
            _ => panic!("Only static DSP sources can be converted into bytes."),
        };

        let mut node = self.dsp_data.generate_graph();

        let wave = Wave32::render(self.sample_rate as f64, duration as f64, node.as_mut());

        let mut buffer = Vec::new();

        wave.write_wav16(&mut buffer)
            .unwrap_or_else(|err| panic!("Cannot write wave to buffer. Error: {err:?}"));

        buffer
    }
}

impl IntoIterator for DspSource {
    type Item = [f32; 2];
    type IntoIter = DspSourceIter;

    fn into_iter(self) -> Self::IntoIter {
        DspSourceIter {
            sample_rate: self.sample_rate,
            audio_unit: RefCell::new(self.dsp_data.generate_graph()),
        }
    }
}

pub struct DspSourceIter {
    pub(crate) sample_rate: f32,
    audio_unit: RefCell<Box<dyn AudioUnit32>>,
}

pub(crate) trait Source {
    type Frame;

    fn sample_rate(&self) -> f32;
    fn sample(&self) -> Self::Frame;

    fn advance(&self, dt: f32) {
        for _ in 0..(self.sample_rate() * dt) as usize {
            self.sample();
        }
    }
}

impl DspSourceIter {
    pub fn into_mono(self) -> DspSourceIterMono {
        DspSourceIterMono(self)
    }
}

impl Source for DspSourceIter {
    type Frame = [f32; 2];

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn sample(&self) -> Self::Frame {
        let frame = self.audio_unit.borrow_mut().get_stereo();
        [frame.0, frame.1]
    }
}

impl Iterator for DspSourceIter {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}

pub struct DspSourceIterMono(pub(crate) DspSourceIter);

impl Source for DspSourceIterMono {
    type Frame = f32;

    fn sample_rate(&self) -> f32 {
        self.0.sample_rate
    }

    fn sample(&self) -> f32 {
        self.0.audio_unit.borrow_mut().get_mono()
    }
}

impl Iterator for DspSourceIterMono {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}
