use std::collections::BTreeMap;

use crate::error::RegisterError;
use crate::factory::AsyncLayerFactory;
use crate::factory::LayerFactory;

#[derive(Default)]
pub(crate) struct LayerRegistry {
    factories: BTreeMap<&'static str, Box<dyn LayerFactory>>,
    async_factories: BTreeMap<&'static str, Box<dyn AsyncLayerFactory>>,
}

impl LayerRegistry {
    pub(crate) fn register<F>(&mut self, factory: F) -> Result<(), RegisterError>
    where
        F: LayerFactory + 'static,
    {
        let kind = factory.kind();
        if self.factories.contains_key(kind) {
            return Err(RegisterError::DuplicateKind(kind));
        }
        self.factories.insert(kind, Box::new(factory));
        Ok(())
    }

    pub(crate) fn get(&self, kind: &str) -> Option<&dyn LayerFactory> {
        self.factories.get(kind).map(Box::as_ref)
    }

    pub(crate) fn register_async<F>(&mut self, factory: F) -> Result<(), RegisterError>
    where
        F: AsyncLayerFactory + 'static,
    {
        let kind = factory.kind();
        if self.async_factories.contains_key(kind) {
            return Err(RegisterError::DuplicateAsyncKind(kind));
        }
        self.async_factories.insert(kind, Box::new(factory));
        Ok(())
    }

    pub(crate) fn get_async(&self, kind: &str) -> Option<&dyn AsyncLayerFactory> {
        self.async_factories.get(kind).map(Box::as_ref)
    }
}
