#![warn(clippy::pedantic)]
#![warn(missing_docs)]

//! DSP plugin for the game engine Bevy.
//!
//! When using types from `fundsp`, always use the `f32` version (`AudioUnit32`, etc).
//!
//! **Always lower your volume when testing sound.**
//!
//! Also, when writing DSP graphs while using clippy, you may encounter the following warning:
//!
//! ```ignore
//! warning: operator precedence can trip the unwary
//! --> examples/kira/noise.rs:14:5
//! |
//! |     white() * 0.2 >> split::<U2>()
//! |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: consider parenthesizing your expression: `(white() * 0.2) >> split::<U2>()`
//! |
//! = note: `#[warn(clippy::precedence)]` on by default
//! = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#precedence
//! ```
//!
//! It is recommended to add `#![allow(clippy::precedence)]` at the top of your crate to stop seeing this error,
//! or applying `#[allow(clippy::precendence)]` at every DSP graph function.
//! See the [`FunDSP` README] for more information.
//!
//! [`FunDSP` README]: https://github.com/SamiPerttu/fundsp

#[cfg(all(feature = "bevy_audio", feature = "kira"))]
compile_error!("feature \"bevy_audio\" and feature \"kira\" cannot be enabled at the same time");

use std::any::{type_name, Any, TypeId};

#[cfg(feature = "bevy_audio")]
use bevy::audio::AudioSource;
#[cfg(feature = "kira")]
use bevy_kira_audio::AudioSource;
#[cfg(feature = "oddio")]
use bevy_oddio::AudioSource;

use bevy::{
    asset::{Assets, Handle},
    prelude::{App, Commands, Plugin, Res, ResMut, StageLabel, StartupStage, SystemStage},
    utils::HashMap,
};
pub use fundsp::hacker32;
use fundsp::hacker32::{AudioUnit32, Wave32};

#[cfg(feature = "kira")]
mod kira_impl;
#[cfg(feature = "bevy_audio")]
mod rodio_impl;
#[cfg(feature = "oddio")]
mod oddio_impl;


/// A source of a DSP graph.
pub struct DspSource {
    graph: Box<dyn AudioUnit32>,
    length: f64,
}

impl DspSource {
    /// Creates a new DSP data source from the given graph.
    pub fn new<X: AudioUnit32 + 'static>(graph: X, length: f64) -> Self {
        Self {
            graph: Box::new(graph),
            length,
        }
    }

    /// Creates a new DSP data source from a boxed audio unit.
    #[must_use]
    pub fn from_boxed(graph: Box<dyn AudioUnit32>, length: f64) -> Self {
        Self { graph, length }
    }

    /// Generate the raw bytes of a DSP graph given the sample rate and its length.
    ///
    /// # Panics
    ///
    /// This panics when it cannot write the DSP graph to a wave buffer.
    #[must_use]
    pub fn generate_raw_bytes(mut self, sample_rate: f64) -> Vec<u8> {
        let wave = Wave32::render(sample_rate, self.length, self.graph.as_mut());

        let mut buffer = Vec::new();

        wave.write_wav16(&mut buffer)
            .unwrap_or_else(|err| panic!("Cannot write wave to buffer. Error: {err:?}"));

        buffer
    }
}

/// A trait that is implemented for all functions
/// that accepts no input and returns an audio graph.
pub trait FnDspGraph: Send + Sync + 'static {
    /// Generate a boxed graph.
    fn generate_graph(&self) -> Box<dyn AudioUnit32>;
}

impl<X: AudioUnit32 + 'static, F> FnDspGraph for F
where
    F: Fn() -> X + Send + Sync + 'static,
{
    fn generate_graph(&self) -> Box<dyn AudioUnit32> {
        Box::new(self())
    }
}

#[cfg(any(feature = "bevy_audio", feature = "oddio"))]
type Settings = ();
#[cfg(feature = "kira")]
type Settings = kira::sound::static_sound::StaticSoundSettings;

/// A DSP graph struct used in the manager.
pub struct DspGraph {
    func: Box<dyn FnDspGraph>,
    length: f64,
    #[allow(dead_code)]
    settings: Settings,
}

impl DspGraph {
    /// Create a new graph from the graph function and its length in seconds.
    #[must_use]
    pub fn new(func: Box<dyn FnDspGraph>, length: f64) -> Self {
        Self {
            func,
            length,
            settings: Settings::default(),
        }
    }
}

/// Manages all DSP graphs.
/// This is automatically added as a resource.
pub struct DspManager {
    graphs: HashMap<TypeId, DspGraph>,
    #[allow(dead_code)] // This is only used when `kira` is enabled.
    sample_rate: f64,
}

impl DspManager {
    /// Add a new graph into the manager.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bevy_fundsp::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugin(DspPlugin)
    ///         .add_startup_system(init_graph)
    ///         .run();
    /// }
    ///
    /// fn white_noise() -> impl AudioUnit32 {
    ///     noise() * 0.2 >> split::<U2>()
    /// }
    ///
    /// fn init_graph(mut dsp_manager: NonSendMut<DspManager>) {
    ///     dsp_manager.add_graph(white_noise, 5.0);
    /// }
    /// ```
    pub fn add_graph<F: FnDspGraph>(&mut self, f: F, length: f64) -> &mut Self {
        self.graphs
            .insert(TypeId::of::<F>(), DspGraph::new(Box::new(f), length));
        self
    }

    /// Remove a graph from the manager.
    pub fn remove_graph<F: FnDspGraph>(&mut self, f: &F) -> &mut Self {
        self.graphs.remove(&Any::type_id(f));
        self
    }

    /// Get a graph from the manager.
    pub fn get_graph<F: FnDspGraph>(&self, f: &F) -> Option<DspSource> {
        self.graphs
            .get(&Any::type_id(f))
            .map(|graph| DspSource::from_boxed(graph.func.generate_graph(), graph.length))
    }

    /// Generate asset handles for all DSP graphs.
    ///
    /// # Panics
    ///
    /// This panics if the [`DspSource`] cannot be converted to a `kira` sound data.
    pub fn add_assets(&self, assets: &mut Assets<AudioSource>) -> DspAssets {
        let handles = self
            .graphs
            .iter()
            .map(|(type_id, graph)| {
                let audio_graph = graph.func.generate_graph();
                let dsp_source = DspSource::from_boxed(audio_graph, graph.length);

                #[cfg(feature = "bevy_audio")]
                let audio_source = dsp_source.into_audio_source(self.sample_rate);
                #[cfg(feature = "kira")]
                let audio_source = dsp_source.into_audio_source(self.sample_rate, graph.settings);
                #[cfg(feature = "oddio")]
                let audio_source = dsp_source.into_audio_source(self.sample_rate);

                let handle = assets.add(audio_source);
                (*type_id, handle)
            })
            .collect();
        DspAssets::new(handles)
    }
}

impl Default for DspManager {
    fn default() -> Self {
        Self {
            graphs: HashMap::default(),
            sample_rate: 44100.0,
        }
    }
}

/// Hashmap for handles of audio sources for DSP graphs.
///
/// This is only available when the `kira` feature is enabled.
///
/// The `DspAssets` resource is initialized before the post-startup stage.
pub struct DspAssets {
    handles: HashMap<TypeId, Handle<AudioSource>>,
}

impl DspAssets {
    fn new(handles: HashMap<TypeId, Handle<AudioSource>>) -> Self {
        Self { handles }
    }

    /// Get a handle to the audio source from the assets.
    pub fn get_graph<X, F>(&self, f: F) -> Option<&Handle<AudioSource>>
    where
        X: AudioUnit32 + 'static,
        F: Fn() -> X + 'static,
    {
        self.handles.get(&Any::type_id(&f))
    }

    /// Get a handle to the audio source from the assets.
    ///
    /// # Panics
    ///
    /// This panics when the given function is not found in the assets map.
    pub fn graph<F: FnDspGraph>(&self, f: &F) -> Handle<AudioSource> {
        self.handles
            .get(&Any::type_id(f))
            .unwrap_or_else(|| {
                panic!(
                    "DSP asset does not exist with the key {:?}.",
                    type_name::<F>()
                )
            })
            .clone()
    }
}

/// A Bevy plugin for adding DSP graphs.
///
/// Add this plugin to your Bevy app
/// to get access to the [`DspManager`] non-send resource.
///
/// Be careful when playing DSP graphs.
/// ⚠ **Lower your volume when testing sound**. ⚠
pub struct DspPlugin;

/// Stage where [`DspManager`] automatically adds assets.
struct AddDspAssetsStage;

impl StageLabel for AddDspAssetsStage {
    fn as_str(&self) -> &'static str {
        "add_dsp_assets_stage"
    }
}

impl Plugin for DspPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DspManager>();

        app.add_startup_stage_before(
            StartupStage::PostStartup,
            AddDspAssetsStage,
            SystemStage::parallel(),
        );

        app.add_startup_system_to_stage(AddDspAssetsStage, generate_assets);
    }
}

/// System to generate assets from [`DspManager`]
#[allow(clippy::needless_pass_by_value)]
fn generate_assets(
    mut commands: Commands,
    dsp_manager: Res<DspManager>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    let assets = dsp_manager.add_assets(&mut assets);
    commands.insert_resource(assets);
}

/// Import the most commonly used items by doing `use bevy_fundsp::prelude::*;`.
pub mod prelude {
    pub use super::*;
    pub use fundsp::hacker32::*;
}

#[doc = include_str!("../README.md")]
#[cfg(all(feature = "bevy_audio", doctest))]
struct DocTestsForReadMe; // Only used for testing code blocks in README.md
