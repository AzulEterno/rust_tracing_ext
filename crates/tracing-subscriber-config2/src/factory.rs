//! Synchronous and asynchronous layer factory contracts and built-in factories.

use std::error::Error;
use std::fs::OpenOptions;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Mutex;

use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::LayerConfig;

/// Error type returned by a layer factory.
pub type BuildError = Box<dyn Error + Send + Sync + 'static>;

/// Boxed future returned by an [`AsyncLayerFactory`].
pub type BuildFuture<'a> =
    Pin<Box<dyn Future<Output = Result<BuiltLayer, BuildError>> + Send + 'a>>;

/// A dynamically dispatched layer installed by this crate.
pub type DynLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;

/// Keeps backend-owned workers or providers alive for the installed subscriber.
///
/// Guards are moved into the [`crate::runtime::TracingHandle`] returned by
/// [`crate::runtime::TracingBuilder::init_global`] and dropped with that handle.
/// A backend can use a guard to keep a worker alive or flush pending data
/// during shutdown. If installation fails, the prepared value drops the guards
/// while returning the installation error.
pub trait ResourceGuard: Send + Sync {}

impl<T> ResourceGuard for T where T: Send + Sync {}

/// A constructed layer and the resources that must outlive it.
pub struct BuiltLayer {
    /// The subscriber layer to install.
    pub layer: DynLayer,
    /// Backend resources retained until the installed tracing handle is dropped.
    pub guards: Vec<Box<dyn ResourceGuard>>,
}

/// Standard stdout formatter layer factory.
///
/// The `config` value is ignored. The resulting formatter includes event
/// targets and writes to standard output.
pub struct FmtLayerFactory;

impl LayerFactory for FmtLayerFactory {
    fn kind(&self) -> &'static str {
        "fmt"
    }

    fn build(&self, _config: &LayerConfig) -> Result<BuiltLayer, BuildError> {
        Ok(BuiltLayer::new(
            tracing_subscriber::fmt::layer().with_target(true),
        ))
    }
}

/// Configuration for [`FileLayerFactory`].
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FileLayerConfig {
    /// File opened in create-and-append mode by [`FileLayerFactory`].
    pub path: PathBuf,
}

/// Append-only file formatter layer factory.
///
/// Parent directories are created when needed. ANSI escapes are disabled for
/// the file output, and event targets are included.
pub struct FileLayerFactory;

impl LayerFactory for FileLayerFactory {
    fn kind(&self) -> &'static str {
        "file"
    }

    fn build(&self, config: &LayerConfig) -> Result<BuiltLayer, BuildError> {
        let config = config
            .deserialize::<FileLayerConfig>()
            .map_err(|error| Box::new(error) as BuildError)?;
        if let Some(parent) = config
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| Box::new(error) as BuildError)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(config.path)
            .map_err(|error| Box::new(error) as BuildError)?;
        Ok(BuiltLayer::new(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_writer(Mutex::new(file)),
        ))
    }
}

impl BuiltLayer {
    /// Boxes a subscriber layer without adding any resource guards.
    pub fn new<L>(layer: L) -> Self
    where
        L: Layer<Registry> + Send + Sync + 'static,
    {
        Self {
            layer: Box::new(layer),
            guards: Vec::new(),
        }
    }

    /// Retains a backend resource until the installed tracing handle is dropped.
    pub fn with_guard<G>(mut self, guard: G) -> Self
    where
        G: ResourceGuard + 'static,
    {
        self.guards.push(Box::new(guard));
        self
    }
}

/// Builds a synchronous subscriber layer for a registered configuration kind.
pub trait LayerFactory: Send + Sync {
    /// Returns the unique configuration kind handled by this factory.
    fn kind(&self) -> &'static str;

    /// Builds a layer from the common layer configuration.
    fn build(&self, config: &LayerConfig) -> Result<BuiltLayer, BuildError>;
}

/// Builds a subscriber layer when preparation is performed asynchronously.
///
/// An async factory may perform asynchronous setup, such as opening a database.
/// [`crate::runtime::TracingBuilder::prepare_async`] prefers it over the
/// synchronous factory registered for the same kind.
pub trait AsyncLayerFactory: Send + Sync {
    /// Returns the unique configuration kind handled by this factory.
    fn kind(&self) -> &'static str;

    /// Starts building a layer from the common layer configuration.
    fn build<'a>(&'a self, config: &'a LayerConfig) -> BuildFuture<'a>;
}
