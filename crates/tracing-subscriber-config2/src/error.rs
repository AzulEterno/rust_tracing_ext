//! Errors produced while registering, preparing, installing, or reloading layers.

use crate::factory::BuildError;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("layer factory kind `{0}` is already registered")]
    DuplicateKind(&'static str),
    #[error("async layer factory kind `{0}` is already registered")]
    DuplicateAsyncKind(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error("layer name `{0}` is duplicated")]
    DuplicateName(String),
    #[error("layer `{name}` uses unknown kind `{kind}`")]
    UnknownKind { name: String, kind: String },
    #[error("layer `{name}` has an invalid filter: {source}")]
    InvalidFilter {
        name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("failed to build layer `{name}`: {source}")]
    Build {
        name: String,
        #[source]
        source: BuildError,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("failed to install global tracing subscriber: {0}")]
pub struct InstallError(#[from] pub tracing::subscriber::SetGlobalDefaultError);

#[derive(Debug, thiserror::Error)]
pub enum ReloadError {
    #[error("unknown tracing layer `{0}`")]
    UnknownLayer(String),
    #[error("invalid tracing filter: {0}")]
    InvalidFilter(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to reload tracing filter: {0}")]
    Reload(#[from] tracing_subscriber::reload::Error),
}
