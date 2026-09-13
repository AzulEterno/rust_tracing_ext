use crate::config::TracingConfig;
use crate::error::PrepareError;
use crate::runtime::TracingBuilder;
use crate::tests::support::layer;

#[test]
fn preparation_rejects_unknown_layer_kind() {
    let error = TracingBuilder::new()
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![layer("missing", "unknown", "info")],
        })
        .err()
        .expect("unknown kind must fail");

    assert!(matches!(error, PrepareError::UnknownKind { kind, .. } if kind == "unknown"));
}
