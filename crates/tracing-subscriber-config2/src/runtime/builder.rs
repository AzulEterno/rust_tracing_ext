use std::collections::BTreeSet;
use std::future::Future;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;
use tracing_subscriber::reload;

use crate::config::TracingConfig;
use crate::error::InitError;
use crate::error::PrepareError;
use crate::error::RegisterError;
use crate::factory::AsyncLayerFactory;
use crate::factory::BuiltLayer;
use crate::factory::DynLayer;
use crate::factory::LayerFactory;
use crate::factory::ResourceGuard;
use crate::registry::LayerRegistry;
use crate::runtime::PreparedTracing;
use crate::runtime::TracingHandle;
use crate::runtime::global::GlobalInitClaim;

/// Collects layer factories and prepares a configured subscriber.
#[derive(Default)]
pub struct TracingBuilder {
    registry: LayerRegistry,
}

impl TracingBuilder {
    /// Creates an empty builder with no registered factories.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a synchronous factory, rejecting duplicate kinds.
    pub fn register<F>(&mut self, factory: F) -> Result<&mut Self, RegisterError>
    where
        F: LayerFactory + 'static,
    {
        self.registry.register(factory)?;
        Ok(self)
    }

    /// Registers an asynchronous factory, rejecting duplicate async kinds.
    pub fn register_async<F>(&mut self, factory: F) -> Result<&mut Self, RegisterError>
    where
        F: AsyncLayerFactory + 'static,
    {
        self.registry.register_async(factory)?;
        Ok(self)
    }

    /// Validates a configuration for synchronous initialization without building layers.
    pub fn validate(&self, config: &TracingConfig) -> Result<(), PrepareError> {
        if config.enabled {
            self.validate_layers(config, false)?;
        }
        Ok(())
    }

    /// Validates a configuration for asynchronous initialization without building layers.
    pub fn validate_async(&self, config: &TracingConfig) -> Result<(), PrepareError> {
        if config.enabled {
            self.validate_layers(config, true)?;
        }
        Ok(())
    }

    /// Validates, builds, and installs process-wide tracing exactly once.
    ///
    /// Validation runs before the process-wide claim. Factory or installation
    /// failures release the claim so the entrance program may retry. A
    /// successful disabled configuration still claims tracing for the process.
    #[track_caller]
    pub fn init_global(&self, config: TracingConfig) -> Result<TracingHandle, InitError> {
        let validated = if config.enabled {
            self.validate_layers(&config, false)?
        } else {
            Vec::new()
        };
        let claim = GlobalInitClaim::acquire(std::panic::Location::caller())?;
        let prepared = if config.enabled {
            self.build_sync(validated)?
        } else {
            PreparedTracing::disabled()
        };
        let handle = prepared.install()?;
        claim.commit();
        Ok(handle)
    }

    /// Asynchronously validates, builds, and installs process-wide tracing exactly once.
    #[track_caller]
    pub fn init_global_async(
        &self,
        config: TracingConfig,
    ) -> impl Future<Output = Result<TracingHandle, InitError>> + '_ {
        let site = std::panic::Location::caller();
        async move {
            let validated = if config.enabled {
                self.validate_layers(&config, true)?
            } else {
                Vec::new()
            };
            let claim = GlobalInitClaim::acquire(site)?;
            let prepared = if config.enabled {
                self.build_async(validated).await?
            } else {
                PreparedTracing::disabled()
            };
            let handle = prepared.install()?;
            claim.commit();
            Ok(handle)
        }
    }

    /// Validates and synchronously builds all enabled configured layers.
    ///
    /// Validation checks duplicate names, registered kinds, and filters before
    /// any factory is built. Use [`Self::init_global`] when the result should
    /// become the process-wide subscriber.
    pub fn prepare(&self, config: TracingConfig) -> Result<PreparedTracing, PrepareError> {
        if !config.enabled {
            return Ok(PreparedTracing::disabled());
        }

        let validated = self.validate_layers(&config, false)?;
        self.build_sync(validated)
    }

    fn build_sync(
        &self,
        validated: Vec<(&crate::config::LayerConfig, EnvFilter)>,
    ) -> Result<PreparedTracing, PrepareError> {
        let mut layers = Vec::with_capacity(validated.len());
        let mut guards = Vec::new();
        let mut filters = std::collections::BTreeMap::new();
        for (config, filter) in validated {
            let factory = self.registry.get(&config.kind).expect("validated factory");
            let built = factory
                .build(config)
                .map_err(|source| PrepareError::Build {
                    name: config.name.clone(),
                    source,
                })?;
            push_built(
                config,
                filter,
                built,
                &mut layers,
                &mut guards,
                &mut filters,
            );
        }

        Ok(PreparedTracing::new(layers, guards, filters))
    }

    /// Asynchronously validates and builds all enabled configured layers.
    ///
    /// For a kind with both factory types registered, the async factory is used.
    /// A synchronous factory is used when no async factory exists for that kind.
    pub async fn prepare_async(
        &self,
        config: TracingConfig,
    ) -> Result<PreparedTracing, PrepareError> {
        if !config.enabled {
            return Ok(PreparedTracing::disabled());
        }

        let validated = self.validate_layers(&config, true)?;
        self.build_async(validated).await
    }

    async fn build_async(
        &self,
        validated: Vec<(&crate::config::LayerConfig, EnvFilter)>,
    ) -> Result<PreparedTracing, PrepareError> {
        let mut layers = Vec::with_capacity(validated.len());
        let mut guards = Vec::new();
        let mut filters = std::collections::BTreeMap::new();
        for (config, filter) in validated {
            let built = if let Some(factory) = self.registry.get_async(&config.kind) {
                factory.build(config).await
            } else {
                self.registry
                    .get(&config.kind)
                    .expect("validated factory")
                    .build(config)
            }
            .map_err(|source| PrepareError::Build {
                name: config.name.clone(),
                source,
            })?;
            push_built(
                config,
                filter,
                built,
                &mut layers,
                &mut guards,
                &mut filters,
            );
        }

        Ok(PreparedTracing::new(layers, guards, filters))
    }

    fn validate_layers<'a>(
        &'a self,
        config: &'a TracingConfig,
        allow_async: bool,
    ) -> Result<Vec<(&'a crate::config::LayerConfig, EnvFilter)>, PrepareError> {
        let mut names = BTreeSet::new();
        let mut validated = Vec::with_capacity(config.layers.len());
        for layer in &config.layers {
            if !names.insert(layer.name.as_str()) {
                return Err(PrepareError::DuplicateName(layer.name.clone()));
            }
            let known = self.registry.get(&layer.kind).is_some()
                || (allow_async && self.registry.get_async(&layer.kind).is_some());
            if !known {
                return Err(PrepareError::UnknownKind {
                    name: layer.name.clone(),
                    kind: layer.kind.clone(),
                });
            }
            let filter = EnvFilter::try_new(&layer.filter).map_err(|source| {
                PrepareError::InvalidFilter {
                    name: layer.name.clone(),
                    source: Box::new(source),
                }
            })?;
            validated.push((layer, filter));
        }
        Ok(validated)
    }
}

fn push_built(
    config: &crate::config::LayerConfig,
    filter: EnvFilter,
    mut built: BuiltLayer,
    layers: &mut Vec<DynLayer>,
    guards: &mut Vec<Box<dyn ResourceGuard>>,
    filters: &mut std::collections::BTreeMap<String, reload::Handle<EnvFilter, Registry>>,
) {
    let (filter, handle) = reload::Layer::<EnvFilter, Registry>::new(filter);
    layers.push(Box::new(built.layer.with_filter(filter)));
    guards.append(&mut built.guards);
    filters.insert(config.name.clone(), handle);
}
