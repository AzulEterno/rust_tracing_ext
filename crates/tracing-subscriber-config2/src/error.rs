//! Errors produced while registering, preparing, installing, or reloading layers.

use crate::factory::BuildError;

/// Errors returned when registering layer factories.
#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    /// A synchronous factory with this kind has already been registered.
    #[error("layer factory kind `{0}` is already registered")]
    DuplicateKind(&'static str),
    /// An asynchronous factory with this kind has already been registered.
    #[error("async layer factory kind `{0}` is already registered")]
    DuplicateAsyncKind(&'static str),
}

/// Errors returned while validating or building configured layers.
#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    /// Two configured layers use the same name.
    #[error("layer name `{0}` is duplicated")]
    DuplicateName(String),
    /// No registered factory provides the configured kind.
    #[error("layer `{name}` uses unknown kind `{kind}`")]
    UnknownKind {
        /// Name of the layer that refers to an unknown kind.
        name: String,
        /// Factory kind that was not registered.
        kind: String,
    },
    /// The layer's filter could not be parsed by `EnvFilter`.
    #[error("layer `{name}` has an invalid filter: {source}")]
    InvalidFilter {
        /// Name of the layer whose filter failed.
        name: String,
        /// The underlying filter parser error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The selected factory could not build the layer.
    #[error("failed to build layer `{name}`: {source}")]
    Build {
        /// Name of the layer whose factory failed.
        name: String,
        /// The factory's build error.
        #[source]
        source: BuildError,
    },
}

/// Error returned when installing the process-wide subscriber fails.
#[derive(Debug, thiserror::Error)]
#[error("failed to install global tracing subscriber: {0}")]
pub struct InstallError(
    /// The error returned when the process already has a global subscriber.
    #[from]
    pub tracing::subscriber::SetGlobalDefaultError,
);

/// Errors returned while replacing a layer's runtime filter.
#[derive(Debug, thiserror::Error)]
pub enum ReloadError {
    /// No configured layer has the requested name.
    #[error("unknown tracing layer `{0}`")]
    UnknownLayer(String),
    /// The replacement filter could not be parsed by `EnvFilter`.
    #[error("invalid tracing filter: {0}")]
    InvalidFilter(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The subscriber rejected the filter replacement.
    #[error("failed to reload tracing filter: {0}")]
    Reload(#[from] tracing_subscriber::reload::Error),
}
