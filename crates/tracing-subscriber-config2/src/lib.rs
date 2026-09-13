#![warn(missing_docs)]

//! Configuration and runtime composition of `tracing_subscriber::Layer` factories.
//!
//! A [`runtime::TracingBuilder`] collects named synchronous and asynchronous
//! factories, validates a [`config::TracingConfig`], and prepares a subscriber
//! for one global installation. Each configured layer gets its own
//! [`tracing_subscriber::EnvFilter`], which can later be replaced through the
//! returned [`runtime::TracingHandle`].
//!
//! # Installation
//!
//! [`runtime::PreparedTracing::install`] calls
//! [`tracing::subscriber::set_global_default`]. Global subscriber installation
//! is process-wide and can succeed only once, so prepare and validate the
//! configuration before calling `install`.
//!
//! ```
//! use tracing_subscriber_config2::config::{LayerConfig, TracingConfig};
//! use tracing_subscriber_config2::factory::FmtLayerFactory;
//! use tracing_subscriber_config2::runtime::TracingBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = TracingBuilder::new();
//! builder.register(FmtLayerFactory)?;
//! let handle = builder
//!     .prepare(TracingConfig {
//!         enabled: true,
//!         layers: vec![LayerConfig {
//!             name: "console".into(),
//!             kind: "fmt".into(),
//!             filter: "info,my_app=debug".into(),
//!             config: serde_value::Value::Unit,
//!         }],
//!     })?
//!     .install()?;
//! handle.reload_filter("console", "warn")?;
//! # Ok(())
//! # }
//! ```
//!
//! Keep the returned handle alive while backend resource guards and filter
//! reloads are needed. Dropping it releases those guards; it does not undo the
//! process-wide subscriber installation.
//!
//! # Factories
//!
//! Register a [`factory::LayerFactory`] with
//! [`runtime::TracingBuilder::register`] for synchronous preparation. Register
//! an [`factory::AsyncLayerFactory`] with
//! [`runtime::TracingBuilder::register_async`] for asynchronous preparation;
//! asynchronous preparation prefers the async factory for a kind and falls
//! back to the synchronous factory when no async factory is registered.

pub mod config;
pub mod error;
pub mod factory;
mod registry;
pub mod runtime;

#[cfg(test)]
mod tests;
