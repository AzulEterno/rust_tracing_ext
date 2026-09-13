use crate::config::TracingConfig;
use crate::error::PrepareError;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;
use crate::tests::support::layer;

#[test]
fn preparation_rejects_duplicate_layer_names() {
    let mut builder = TracingBuilder::new();
    builder.register(factory().0).expect("register factory");
    let error = builder
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![
                layer("same", "count", "info"),
                layer("same", "count", "debug"),
            ],
        })
        .err()
        .expect("duplicate names must fail");

    assert!(matches!(error, PrepareError::DuplicateName(name) if name == "same"));
}
