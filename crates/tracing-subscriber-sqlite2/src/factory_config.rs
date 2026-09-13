use std::path::PathBuf;

use crate::Config;

/// Backend-owned configuration consumed by the SQLite layer factories.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SqliteFactoryConfig {
    /// SQLite database path passed to the selected layer constructor.
    pub path: PathBuf,
    /// SQLite storage and queue settings flattened alongside `path` in serialized configuration.
    #[serde(flatten)]
    pub sqlite: Config,
}
