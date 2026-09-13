use tracing_subscriber_config2::config::LayerConfig;
use tracing_subscriber_config2::factory::BuildError;
use tracing_subscriber_config2::factory::BuiltLayer;
use tracing_subscriber_config2::factory::LayerFactory;

use crate::SqliteFactoryConfig;
use crate::SqliteLayer;

pub struct SqliteLayerFactory;

impl LayerFactory for SqliteLayerFactory {
    fn kind(&self) -> &'static str {
        "sqlite"
    }

    fn build(&self, config: &LayerConfig) -> Result<BuiltLayer, BuildError> {
        let config = config
            .deserialize::<SqliteFactoryConfig>()
            .map_err(|error| Box::new(error) as BuildError)?;
        build(config)
    }
}

pub(crate) fn build(config: SqliteFactoryConfig) -> Result<BuiltLayer, BuildError> {
    let layer = SqliteLayer::open_with_config(config.path, config.sqlite)
        .map_err(|error| Box::new(error) as BuildError)?;
    Ok(BuiltLayer::new(layer.clone()).with_guard(FlushGuard(layer)))
}

struct FlushGuard(SqliteLayer);

impl Drop for FlushGuard {
    fn drop(&mut self) {
        self.0.flush();
    }
}
