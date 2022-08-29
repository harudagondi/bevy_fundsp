#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]

//! This library integrates [FunDSP] into [Bevy].
//! 
//! When using this library, **remember to lower your volume first**!
//! 
//! [FunDSP]: https://github.com/SamiPerttu/fundsp
//! [Bevy]: https://bevyengine.org/

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

/// Add support for using [FunDSP graphs] in Bevy code.
pub struct DspPlugin<B = DefaultBackend>
where
    B: Backend,
{
    sample_rate: f32,
    _backend: PhantomData<B>,
}

impl<B: Backend> DspPlugin<B> {
    /// Construct the plugin given the sample rate.
    /// 
    /// It is recommended to use the [`Default`]
    /// implementation to avoid problems with audio output.
    /// 
    /// Internally, the default plugin gets the sample rate
    /// of the device using [`cpal`].
    #[allow(clippy::must_use_candidate)]
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

/// Trait extension for the [`App`] struct.
pub trait DspAppExt {
    /// Register a DSP source with the given [`SourceType`].
    /// 
    /// The type to be registered must implement [`DspGraph`].
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_graph: D, source_type: SourceType) -> &mut Self;
}

impl DspAppExt for App {
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_graph: D, source_type: SourceType) -> &mut Self {
        let mut dsp_manager = self.world.resource_mut::<DspManager>();

        dsp_manager.add_graph(dsp_graph, source_type);

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

    #[allow(clippy::cast_precision_loss)]
    {
        default_config.sample_rate().0 as f32
    }
}

/// Prelude for all `bevy_fundsp` types.
pub mod prelude {
    pub use crate::backend::{Backend, DspAudioExt};
    pub use crate::dsp_data::DspGraph;
    pub use crate::dsp_manager::DspManager;
    pub use crate::dsp_source::{DspSource, Iter, IterMono, SourceType};
    pub use crate::DspPlugin;
    pub use crate::DspAppExt;
}

pub use prelude::*;

#[doc = include_str!("../README.md")]
#[cfg(all(feature = "bevy_audio", doctest))]
struct DocTestsForReadMe; // Only used for testing code blocks in README.md