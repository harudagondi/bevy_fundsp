use fundsp::prelude::AudioUnit32;
use uuid::Uuid;

use crate::dsp_source::{DspSource, SourceType};

pub trait DspGraph: Send + Sync + 'static {
    fn id(&self) -> Uuid;
    fn generate_graph(&self) -> Box<dyn AudioUnit32>;

    fn into_dsp_source(self, sample_rate: f32, source_type: SourceType) -> DspSource
    where
        Self: Sized,
    {
        DspSource::new(self, sample_rate, source_type)
    }
}

impl<F, Au> DspGraph for F
where
    F: Send + Sync + 'static + Fn() -> Au,
    Au: AudioUnit32 + 'static,
{
    fn id(&self) -> Uuid {
        // TODO: This should be based on its `TypeId`
        Uuid::new_v5(&Uuid::NAMESPACE_OID, std::any::type_name::<F>().as_bytes())
    }

    fn generate_graph(&self) -> Box<dyn AudioUnit32> {
        Box::new(self())
    }
}
