use std::error::Error;
use std::fs::OpenOptions;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Mutex;

use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::LayerConfig;

pub type BuildError = Box<dyn Error + Send + Sync + 'static>;
pub type BuildFuture<'a> =
    Pin<Box<dyn Future<Output = Result<BuiltLayer, BuildError>> + Send + 'a>>;
pub type DynLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;

/// Keeps backend-owned workers or providers alive for the installed subscriber.
pub trait ResourceGuard: Send + Sync {}

impl<T> ResourceGuard for T where T: Send + Sync {}

pub struct BuiltLayer {
    pub layer: DynLayer,
    pub guards: Vec<Box<dyn ResourceGuard>>,
}

/// Standard stdout formatter layer factory.
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FileLayerConfig {
    pub path: PathBuf,
}

/// Append-only file formatter layer factory.
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
    pub fn new<L>(layer: L) -> Self
    where
        L: Layer<Registry> + Send + Sync + 'static,
    {
        Self {
            layer: Box::new(layer),
            guards: Vec::new(),
        }
    }

    pub fn with_guard<G>(mut self, guard: G) -> Self
    where
        G: ResourceGuard + 'static,
    {
        self.guards.push(Box::new(guard));
        self
    }
}

pub trait LayerFactory: Send + Sync {
    fn kind(&self) -> &'static str;

    fn build(&self, config: &LayerConfig) -> Result<BuiltLayer, BuildError>;
}

pub trait AsyncLayerFactory: Send + Sync {
    fn kind(&self) -> &'static str;

    fn build<'a>(&'a self, config: &'a LayerConfig) -> BuildFuture<'a>;
}
