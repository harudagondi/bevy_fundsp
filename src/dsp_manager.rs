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
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..default()
        }
    }

    pub(crate) fn add_graph<D: DspGraph>(&mut self, dsp_data: D, source_type: SourceType) {
        self.collection.insert(
            dsp_data.id(),
            DspSource::new(dsp_data, self.sample_rate, source_type),
        );
    }

    pub fn get_graph<D: DspGraph>(&self, dsp_data: D) -> Option<&DspSource> {
        self.collection.get(&dsp_data.id())
    }
}
