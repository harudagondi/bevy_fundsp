use std::marker::PhantomData;

use backend::{Backend, DefaultBackend};
use bevy::prelude::{AddAsset, App, Plugin};
use cpal::traits::{DeviceTrait, HostTrait};
use dsp_data::DspGraph;
use dsp_manager::DspManager;
use dsp_source::{DspSource, SourceType};

pub mod backend;
pub mod dsp_data;
pub mod dsp_manager;
pub mod dsp_source;

pub struct DspPlugin<B = DefaultBackend>
where
    B: Backend,
{
    sample_rate: f32,
    _backend: PhantomData<B>,
}

impl<B: Backend> DspPlugin<B> {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            _backend: PhantomData,
        }
    }
}

impl<B: Backend> Default for DspPlugin<B> {
    fn default() -> Self {
        Self {
            sample_rate: default_sample_rate(),
            _backend: PhantomData,
        }
    }
}

impl<B: Backend> Plugin for DspPlugin<B> {
    fn build(&self, app: &mut App) {
        app.insert_resource(DspManager::new(self.sample_rate))
            .add_asset::<DspSource>();

        B::init_app(app);
    }
}

pub trait DspAppExt {
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_data: D, source_type: SourceType) -> &mut Self;
}

impl DspAppExt for App {
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_data: D, source_type: SourceType) -> &mut Self {
        let mut dsp_manager = self.world.resource_mut::<DspManager>();

        dsp_manager.add_graph(dsp_data, source_type);

        self
    }
}

fn default_sample_rate() -> f32 {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .unwrap_or_else(|| panic!("No output device available."));
    let default_config = device
        .default_output_config()
        .unwrap_or_else(|err| panic!("Cannot find default stream config. Error: {err}"));

    default_config.sample_rate().0 as f32
}
