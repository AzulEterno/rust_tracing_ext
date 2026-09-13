//! Two-phase tracing preparation, installation, and runtime control.
//!
//! [`TracingBuilder`] can prepare layers without touching the process-wide
//! subscriber, or perform one guarded global initialization through
//! [`TracingBuilder::init_global`]. The returned [`TracingHandle`] controls
//! filter reloads and backend guard lifetimes.

mod builder;
mod global;
mod handle;
mod prepared;

pub use builder::TracingBuilder;
pub use handle::TracingHandle;
pub use prepared::PreparedTracing;
