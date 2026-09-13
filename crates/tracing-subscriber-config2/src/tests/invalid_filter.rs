use std::sync::atomic::Ordering;

use crate::config::TracingConfig;
use crate::error::PrepareError;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;
use crate::tests::support::layer;

#[test]
fn preparation_validates_filters_before_building() {
    let (factory, _, builds, _) = factory();
    let mut builder = TracingBuilder::new();
    builder.register(factory).expect("register factory");
    let error = builder
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![layer("bad-filter", "count", "[invalid")],
        })
        .err()
        .expect("invalid filter must fail");

    assert!(matches!(error, PrepareError::InvalidFilter { .. }));
    assert_eq!(builds.load(Ordering::Relaxed), 0);
}
