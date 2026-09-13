//! Serializable configuration shared by all tracing layer factories.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_value::Value;

/// Top-level tracing configuration.
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct TracingConfig {
    /// Whether preparation should build layers and installation should set the global subscriber.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    /// Named layers to validate and build in declaration order.
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
    /// Unique name used to identify this layer for filter reloads.
    pub name: String,
    /// Factory kind registered in the [`crate::runtime::TracingBuilder`].
    pub kind: String,
    /// `tracing_subscriber::EnvFilter` directives applied only to this layer.
    #[serde(default = "default_filter")]
    pub filter: String,
    /// Backend-specific configuration kept opaque until the selected factory reads it.
    #[serde(default = "empty_backend_config")]
    pub config: Value,
}

impl LayerConfig {
    /// Deserializes this layer's opaque backend configuration into a factory-owned type.
    ///
    /// A factory should use this method to keep its configuration schema separate
    /// from the common layer name, kind, and filter fields.
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
