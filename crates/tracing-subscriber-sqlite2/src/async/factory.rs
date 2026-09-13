use tracing_subscriber_config2::config::LayerConfig;
use tracing_subscriber_config2::factory::AsyncLayerFactory;
use tracing_subscriber_config2::factory::BuildError;
use tracing_subscriber_config2::factory::BuildFuture;
use tracing_subscriber_config2::factory::BuiltLayer;

use crate::AsyncSqliteLayer;
use crate::SqliteFactoryConfig;

/// Builds SQLite layers during asynchronous configuration preparation.
///
/// This type is available with both the `async` and `configurable` features.
/// Register it with `TracingBuilder::register_async`; its configuration kind is
/// `"sqlite"`. When `async_backend` is `false`, it delegates to the synchronous
/// SQLite builder so one async preparation path can support either backend.
pub struct AsyncSqliteLayerFactory;

impl AsyncLayerFactory for AsyncSqliteLayerFactory {
    fn kind(&self) -> &'static str {
        "sqlite"
    }

    fn build<'a>(&'a self, config: &'a LayerConfig) -> BuildFuture<'a> {
        Box::pin(async move {
            let config = config
                .deserialize::<SqliteFactoryConfig>()
                .map_err(|error| Box::new(error) as BuildError)?;
            if !config.sqlite.async_backend {
                return crate::sync::factory::build(config);
            }
            let layer = AsyncSqliteLayer::open_with_config(config.path, config.sqlite)
                .await
                .map_err(|error| Box::new(error) as BuildError)?;
            Ok(BuiltLayer::new(layer))
        })
    }
}
