use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use tracing::Event;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

use crate::config::LayerConfig;
use crate::factory::BuildError;
use crate::factory::BuiltLayer;
use crate::factory::LayerFactory;

pub struct CountingFactory {
    pub events: Arc<AtomicUsize>,
    pub builds: Arc<AtomicUsize>,
    pub guard_drops: Arc<AtomicUsize>,
}

impl LayerFactory for CountingFactory {
    fn kind(&self) -> &'static str {
        "count"
    }

    fn build(&self, _config: &LayerConfig) -> Result<BuiltLayer, BuildError> {
        self.builds.fetch_add(1, Ordering::Relaxed);
        Ok(BuiltLayer::new(CountLayer(Arc::clone(&self.events)))
            .with_guard(DropGuard(Arc::clone(&self.guard_drops))))
    }
}

struct CountLayer(Arc<AtomicUsize>);

impl Layer<Registry> for CountLayer {
    fn on_event(&self, _event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, Registry>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct DropGuard(Arc<AtomicUsize>);

impl Drop for DropGuard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn factory() -> (
    CountingFactory,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    let events = Arc::new(AtomicUsize::new(0));
    let builds = Arc::new(AtomicUsize::new(0));
    let guard_drops = Arc::new(AtomicUsize::new(0));
    (
        CountingFactory {
            events: Arc::clone(&events),
            builds: Arc::clone(&builds),
            guard_drops: Arc::clone(&guard_drops),
        },
        events,
        builds,
        guard_drops,
    )
}

pub fn layer(name: &str, kind: &str, filter: &str) -> LayerConfig {
    LayerConfig {
        name: name.to_owned(),
        kind: kind.to_owned(),
        filter: filter.to_owned(),
        config: serde_value::Value::Map(BTreeMap::new()),
    }
}
