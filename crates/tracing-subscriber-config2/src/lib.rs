//! Configuration and runtime composition of `tracing_subscriber::Layer` factories.

pub mod config;
pub mod error;
pub mod factory;
mod registry;
pub mod runtime;

#[cfg(test)]
mod tests;
