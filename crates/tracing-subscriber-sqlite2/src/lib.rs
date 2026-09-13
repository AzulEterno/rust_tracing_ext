//! Bounded, batched `tracing_subscriber::Layer` implementations backed by SQLite.

#[cfg(feature = "async")]
#[path = "async/mod.rs"]
mod async_mode;
mod capture;
mod config;
#[cfg(feature = "configurable")]
mod factory_config;
mod format;
mod model;
mod schema;
mod storage;
mod sync;

#[cfg(feature = "async")]
pub use async_mode::AsyncSqliteLayer;
#[cfg(all(feature = "async", feature = "configurable"))]
pub use async_mode::AsyncSqliteLayerFactory;
pub use config::Config;
#[cfg(feature = "configurable")]
pub use factory_config::SqliteFactoryConfig;
pub use model::EventLevel;
pub use model::EventRecord;
pub use sync::SqliteLayer;
#[cfg(feature = "configurable")]
pub use sync::SqliteLayerFactory;

#[cfg(test)]
mod tests;
