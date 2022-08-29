//! Module for [`DspManager`].

use bevy::{
    prelude::{default, Resource},
    utils::HashMap,
};
use uuid::Uuid;

use crate::{
    default_sample_rate,
    dsp_data::DspGraph,
    dsp_source::{DspSource, SourceType},
};

/// Manages the registered DSP sources.
///
/// This is a public facing interface
/// for the user to access the stored DSP sources.
#[derive(Resource)]
pub struct DspManager {
    collection: HashMap<Uuid, DspSource>,
    sample_rate: f32,
}

impl Default for DspManager {
    fn default() -> Self {
        Self {
            collection: default(),
            sample_rate: default_sample_rate(),
        }
    }
}

impl DspManager {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..default()
        }
    }

    pub(crate) fn add_graph<D: DspGraph>(&mut self, dsp_graph: D, source_type: SourceType) {
        self.collection.insert(
            dsp_graph.id(),
            DspSource::new(dsp_graph, self.sample_rate, source_type),
        );
    }

    /// Get the DSP source given a DSP graph.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_graph<D: DspGraph>(&self, dsp_graph: D) -> Option<&DspSource> {
        self.collection.get(&dsp_graph.id())
    }
}
