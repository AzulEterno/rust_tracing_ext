use std::sync::atomic::Ordering;

use crate::config::TracingConfig;
use crate::error::InitError;
use crate::error::PrepareError;
use crate::factory::BuildError;
use crate::factory::BuiltLayer;
use crate::factory::LayerFactory;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;
use crate::tests::support::layer;

#[test]
fn installation_supports_filter_reload_and_guard_lifecycle() {
    let mut failing_builder = TracingBuilder::new();
    failing_builder
        .register(FailingFactory)
        .expect("register failing factory");
    let error = match failing_builder.init_global(TracingConfig {
        enabled: true,
        layers: vec![layer("failure", "fail", "info")],
    }) {
        Ok(_) => panic!("factory failure must fail initialization"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        InitError::Prepare(PrepareError::Build { .. })
    ));

    let (factory, events, builds, guard_drops) = factory();
    let mut builder = TracingBuilder::new();
    builder.register(factory).expect("register factory");
    let handle = builder
        .init_global(TracingConfig {
            enabled: true,
            layers: vec![layer("counter", "count", "off,configurable_test=info")],
        })
        .expect("initialize tracing");

    tracing::info!(target: "configurable_test", "first");
    assert_eq!(events.load(Ordering::Relaxed), 1);
    handle
        .reload_filter("counter", "off")
        .expect("reload filter");
    tracing::info!(target: "configurable_test", "second");
    assert_eq!(events.load(Ordering::Relaxed), 1);

    let error = match builder.init_global(TracingConfig::default()) {
        Ok(_) => panic!("second initialization must fail"),
        Err(error) => error,
    };
    assert!(matches!(error, InitError::AlreadyInitialized { .. }));
    assert_eq!(builds.load(Ordering::Relaxed), 1);

    drop(handle);
    assert_eq!(guard_drops.load(Ordering::Relaxed), 1);
}

struct FailingFactory;

impl LayerFactory for FailingFactory {
    fn kind(&self) -> &'static str {
        "fail"
    }

    fn build(&self, _config: &crate::config::LayerConfig) -> Result<BuiltLayer, BuildError> {
        Err(std::io::Error::other("expected build failure").into())
    }
}
