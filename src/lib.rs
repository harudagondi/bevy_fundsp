#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]

//! This library integrates [FunDSP] into [Bevy].
//!
//! When using this library, **remember to lower your volume first**!
//!
//! Also, you may encounter the following error when using this library:
//!
//! ```text no_run
//! warning: operator precedence can trip the unwary
//!   --> examples/bevy_audio/noise.rs:22:5
//!    |
//! 22 |     white() >> split::<U2>() * 0.2
//!    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: consider parenthesizing your expression: `white() >> (split::<U2>() * 0.2)`
//!    |
//!    = note: `#[warn(clippy::precedence)]` on by default
//!    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#precedence
//!
//! warning: `bevy_fundsp` (example "noise") generated 1 warning
//! ```
//!
//! This isn't necessary when writing your DSP graphs.
//! It is more intuitive to remove the parentheses when writing these types of expressions,
//! as FunDSP is essentially a domain specific language.
//! See the [FunDSP] README for more information.
//!  
//! [FunDSP]: https://github.com/SamiPerttu/fundsp
//! [Bevy]: https://bevyengine.org/

use bevy::asset::AssetApp;
use {
    backend::{Backend, DefaultBackend},
    bevy::prelude::{App, Plugin},
    dsp_graph::DspGraph,
    dsp_manager::DspManager,
    dsp_source::{DspSource, SourceType},
    once_cell::sync::Lazy,
};

pub mod backend;
pub mod dsp_graph;
pub mod dsp_manager;
pub mod dsp_source;

/// Add support for using [FunDSP graphs] in Bevy code.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_fundsp::prelude::*;
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugin(DspPlugin::default())
///     .run()
/// ```
pub struct DspPlugin {
    sample_rate: f32,
}

impl DspPlugin {
    /// Construct the plugin given the sample rate.
    ///
    /// It is recommended to use the [`Default`]
    /// implementation to avoid problems with audio output.
    ///
    /// Internally, the default plugin gets the sample rate
    /// of the device using [`cpal`].
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fundsp::prelude::*;
    /// App::new()
    ///     .add_plugins(DefaultPlugins)
    ///     .add_plugin(DspPlugin::new(44100.0))
    ///     .run()
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }
}

impl Default for DspPlugin {
    fn default() -> Self {
        Self::new(*DEFAULT_SAMPLE_RATE)
    }
}

impl Plugin for DspPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DspManager::new(self.sample_rate))
            .init_asset::<DspSource>();

        DefaultBackend::init_app(app);
    }
}

/// Trait extension for the [`App`] struct.
pub trait DspAppExt {
    /// Register a DSP source with the given [`SourceType`].
    ///
    /// The type to be registered must implement [`DspGraph`].
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_fundsp::prelude::*;
    /// App::new()
    ///     .add_plugins(DefaultPlugins)
    ///     .add_plugin(DspPlugin::default())
    ///     .add_dsp_source(a_simple_440hz_sine_wave, SourceType::Dynamic)
    ///     .run();
    ///
    /// fn a_simple_440hz_sine_wave() -> impl AudioUnit32 {
    ///     sine_hz(440.0)
    /// }
    /// ```
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_graph: D, source_type: SourceType) -> &mut Self;
}

impl DspAppExt for App {
    fn add_dsp_source<D: DspGraph>(&mut self, dsp_graph: D, source_type: SourceType) -> &mut Self {
        let mut dsp_manager = self.world.resource_mut::<DspManager>();

        dsp_manager.add_graph(dsp_graph, source_type);

        self
    }
}

static DEFAULT_SAMPLE_RATE: Lazy<f32> = Lazy::new(default_sample_rate);

#[cfg(not(test))]
fn default_sample_rate() -> f32 {
    use cpal::traits::{DeviceTrait, HostTrait};

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

#[cfg(test)]
fn default_sample_rate() -> f32 {
    bevy::log::warn!("This is in test mode!");
    44100.0
}

/// Prelude for all `bevy_fundsp` types.
///
/// This also includes the `fundsp::hacker32` prelude.
pub mod prelude {
    pub use {
        crate::{
            backend::{Backend, DefaultBackend, DspAudioExt},
            dsp_graph::DspGraph,
            dsp_manager::DspManager,
            dsp_source::{DspSource, Iter, IterMono, SourceType},
            DspAppExt, DspPlugin,
        },
        fundsp::hacker32::*,
    };
}

#[doc = include_str!("../README.md")]
#[cfg(all(feature = "bevy_audio", doctest))]
struct DocTestsForReadMe; // Only used for testing code blocks in README.md
