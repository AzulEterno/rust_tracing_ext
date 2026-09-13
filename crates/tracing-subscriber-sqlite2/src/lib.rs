#![warn(missing_docs)]

//! Bounded, batched `tracing_subscriber::Layer` implementations backed by SQLite.
//!
//! The default [`SqliteLayer`] uses a standard-library worker thread. Enable
//! the `async` feature for `AsyncSqliteLayer`, which uses Tokio and
//! `tokio-rusqlite`. Both layers capture event fields, span scope, and source
//! metadata, then write events in bounded batches.
//!
//! # Feature flags
//!
//! - `bundled` (default) builds the SQLite library bundled with `rusqlite`.
//!   Disable default features to use system SQLite instead.
//! - `async` enables `AsyncSqliteLayer` and its Tokio dependencies.
//! - `serde` enables serialization and deserialization of [`Config`].
//! - `configurable` enables `SqliteFactoryConfig` and the synchronous
//!   `SqliteLayerFactory`. It also enables `serde`.
//! - `async` plus `configurable` enables `AsyncSqliteLayerFactory`.
//! - `limits` forwards `rusqlite`'s SQLite runtime-limit API feature.
//!
//! The `async_backend` setting must match the layer being opened: synchronous
//! layers require `false`, while async layers require `true`. The convenience
//! [`SqliteLayer::open`] and `AsyncSqliteLayer::open` methods choose the
//! matching value for their backend.
//!
//! # Queue and flush behavior
//!
//! Event delivery is non-blocking. If the bounded queue is full, the event is
//! discarded and [`SqliteLayer::dropped_count`] or
//! `AsyncSqliteLayer::dropped_count` increases. Worker write failures are
//! counted separately by `write_error_count`; a count represents a failed
//! batch write, not necessarily one failed event.
//!
//! [`SqliteLayer::flush`] blocks until its flush marker reaches the synchronous
//! worker. `AsyncSqliteLayer::flush` awaits the same point for the async
//! worker. The worker commits batches before acknowledging the marker. Events
//! submitted concurrently with a flush can race with that marker.
//!
//! # Storage schema
//!
//! A configuration with table prefix `tracing` creates `tracing_events`, the
//! configured shared string table (by default `tracing_strings`), and indexes
//! named `tracing_events_timestamp` and `tracing_events_target`. Identifiers
//! must start with `_` or an ASCII letter and then contain only ASCII letters,
//! digits, and underscores so they can be safely interpolated into the schema
//! statements. Targets, module paths, and file names are interned in the
//! shared string table; each event stores IDs for those values.
//!
//! # Synchronous example
//!
//! ```
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber_sqlite2::SqliteLayer;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let layer = SqliteLayer::open(":memory:")?;
//! let subscriber = tracing_subscriber::registry().with(layer.clone());
//! let _guard = subscriber.set_default();
//! tracing::info!(request_id = "request-1", "persisted");
//! layer.flush();
//! # Ok(())
//! # }
//! ```
//!
//! # Async example
//!
//! This example is compiled only when the `async` feature is enabled.
//!
//! ```
//! # #[cfg(feature = "async")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::runtime::Builder::new_current_thread()
//! #     .enable_all()
//! #     .build()?
//! #     .block_on(async {
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber_sqlite2::AsyncSqliteLayer;
//!
//! let layer = AsyncSqliteLayer::open(":memory:").await?;
//! let subscriber = tracing_subscriber::registry().with(layer.clone());
//! let _guard = subscriber.set_default();
//! tracing::info!(request_id = "request-1", "persisted");
//! layer.flush().await;
//! # Ok::<(), tokio_rusqlite::Error>(())
//! #     })?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "async"))]
//! # fn main() {}
//! ```
//!
//! # Configurable factory example
//!
//! This example is compiled only when the `configurable` feature is enabled.
//! The factory consumes the same flattened `SqliteFactoryConfig` shape used
//! by serialized application configuration.
//!
//! ```
//! # #[cfg(feature = "configurable")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use tracing_subscriber_config2::config::{LayerConfig, TracingConfig};
//! use tracing_subscriber_config2::runtime::TracingBuilder;
//! use tracing_subscriber_sqlite2::{Config, SqliteFactoryConfig, SqliteLayerFactory};
//!
//! let mut builder = TracingBuilder::new();
//! builder.register(SqliteLayerFactory)?;
//! let config = SqliteFactoryConfig {
//!     path: ":memory:".into(),
//!     sqlite: Config::default(),
//! };
//! let prepared = builder.prepare(TracingConfig {
//!     enabled: true,
//!     layers: vec![LayerConfig {
//!         name: "sqlite".into(),
//!         kind: "sqlite".into(),
//!         filter: "info".into(),
//!         config: serde_value::to_value(config)?,
//!     }],
//! })?;
//! drop(prepared);
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "configurable"))]
//! # fn main() {}
//! ```

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
