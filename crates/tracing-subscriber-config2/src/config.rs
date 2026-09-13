//! Serializable configuration shared by all tracing layer factories.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_value::Value;

/// Top-level tracing configuration.
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct TracingConfig {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default)]
    pub layers: Vec<LayerConfig>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            layers: Vec::new(),
        }
    }
}

/// Generic layer configuration; `config` remains owned by the backend factory.
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct LayerConfig {
    pub name: String,
    pub kind: String,
    #[serde(default = "default_filter")]
    pub filter: String,
    #[serde(default = "empty_backend_config")]
    pub config: Value,
}

impl LayerConfig {
    pub fn deserialize<T>(&self) -> Result<T, serde_value::DeserializerError>
    where
        T: DeserializeOwned,
    {
        T::deserialize(self.config.clone())
    }
}

fn enabled_by_default() -> bool {
    true
}

fn default_filter() -> String {
    "info".to_owned()
}

fn empty_backend_config() -> Value {
    Value::Map(BTreeMap::new())
}
