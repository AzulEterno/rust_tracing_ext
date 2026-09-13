use std::path::PathBuf;

use crate::Config;

/// Backend-owned configuration consumed by the SQLite layer factories.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SqliteFactoryConfig {
    pub path: PathBuf,
    #[serde(flatten)]
    pub sqlite: Config,
}
