//! Module for the [`DspGraph`] trait.

use {fundsp::prelude::AudioUnit32, uuid::Uuid};

/// Trait for generating DSP sources.
///
/// This is the public interface for registering custom DSP graphs.
///
/// Currently, this is only implemented for functions
/// that implement `Fn() -> impl AudioUnit32`.
///
/// If parameterless functions isn't enough for your usecase,
/// you can implement your own custom type.
pub trait DspGraph: Send + Sync + 'static {
    /// The ID of the given graph.
    ///
    /// Different graphs must return different IDs,
    /// even if they return the same type.
    ///
    /// This is used internally in [`DspManager`].
    ///
    /// [`DspManager`]: crate::dsp_manager::DspManager
    fn id(&self) -> Uuid;

    /// Generate a DSP graph.
    fn generate_graph(&self) -> Box<dyn AudioUnit32>;
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
