use std::sync::atomic::Ordering;

use crate::config::TracingConfig;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;
use crate::tests::support::layer;

#[test]
fn installation_supports_filter_reload_and_guard_lifecycle() {
    let (factory, events, _, guard_drops) = factory();
    let mut builder = TracingBuilder::new();
    builder.register(factory).expect("register factory");
    let handle = builder
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![layer("counter", "count", "off,configurable_test=info")],
        })
        .expect("prepare tracing")
        .install()
        .expect("install tracing");

    tracing::info!(target: "configurable_test", "first");
    assert_eq!(events.load(Ordering::Relaxed), 1);
    handle
        .reload_filter("counter", "off")
        .expect("reload filter");
    tracing::info!(target: "configurable_test", "second");
    assert_eq!(events.load(Ordering::Relaxed), 1);

    drop(handle);
    assert_eq!(guard_drops.load(Ordering::Relaxed), 1);
}
