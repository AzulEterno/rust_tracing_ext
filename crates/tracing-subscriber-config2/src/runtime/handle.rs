use std::collections::BTreeMap;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::reload;

use crate::error::ReloadError;
use crate::factory::ResourceGuard;

/// Keeps backend resources alive and exposes per-layer filter reloads.
pub struct TracingHandle {
    _guards: Vec<Box<dyn ResourceGuard>>,
    filters: BTreeMap<String, reload::Handle<EnvFilter, Registry>>,
}

impl TracingHandle {
    pub(crate) fn new(
        guards: Vec<Box<dyn ResourceGuard>>,
        filters: BTreeMap<String, reload::Handle<EnvFilter, Registry>>,
    ) -> Self {
        Self {
            _guards: guards,
            filters,
        }
    }

    pub fn reload_filter(&self, layer: &str, filter: &str) -> Result<(), ReloadError> {
        let filter = EnvFilter::try_new(filter)
            .map_err(|error| ReloadError::InvalidFilter(Box::new(error)))?;
        self.filters
            .get(layer)
            .ok_or_else(|| ReloadError::UnknownLayer(layer.to_owned()))?
            .reload(filter)?;
        Ok(())
    }

    pub fn layer_names(&self) -> impl Iterator<Item = &str> {
        self.filters.keys().map(String::as_str)
    }
}
