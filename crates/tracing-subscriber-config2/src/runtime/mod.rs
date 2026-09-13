//! Two-phase tracing preparation, installation, and runtime control.
//!
//! [`TracingBuilder`] performs validation and factory setup without touching
//! the process-wide tracing subscriber. [`PreparedTracing::install`] performs
//! the one global installation, returning [`TracingHandle`] for filter reloads
//! and backend guard lifetimes.

mod builder;
mod handle;
mod prepared;

pub use builder::TracingBuilder;
pub use handle::TracingHandle;
pub use prepared::PreparedTracing;
