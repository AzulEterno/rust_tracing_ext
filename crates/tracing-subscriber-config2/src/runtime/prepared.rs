use std::collections::BTreeMap;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;

use crate::error::InstallError;
use crate::factory::DynLayer;
use crate::factory::ResourceGuard;
use crate::runtime::TracingHandle;

/// Fully validated layers and resources ready for one global installation.
///
/// This value owns the constructed layers until [`install`](Self::install) is
/// called. An enabled value can be installed only once because installation
/// consumes it and the underlying tracing global is process-wide.
pub struct PreparedTracing {
    enabled: bool,
    layers: Vec<DynLayer>,
    guards: Vec<Box<dyn ResourceGuard>>,
    filters: BTreeMap<String, reload::Handle<EnvFilter, Registry>>,
}

impl PreparedTracing {
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            layers: Vec::new(),
            guards: Vec::new(),
            filters: BTreeMap::new(),
        }
    }

    pub(crate) fn new(
        layers: Vec<DynLayer>,
        guards: Vec<Box<dyn ResourceGuard>>,
        filters: BTreeMap<String, reload::Handle<EnvFilter, Registry>>,
    ) -> Self {
        Self {
            enabled: true,
            layers,
            guards,
            filters,
        }
    }

    /// Installs the prepared layers as the process-wide global subscriber.
    ///
    /// Disabled configurations return a handle without registering a global
    /// subscriber. For enabled configurations, this fails if another global
    /// subscriber has already been installed.
    pub fn install(self) -> Result<TracingHandle, InstallError> {
        if self.enabled {
            tracing::subscriber::set_global_default(Registry::default().with(self.layers))?;
        }
        Ok(TracingHandle::new(self.guards, self.filters))
    }
}
