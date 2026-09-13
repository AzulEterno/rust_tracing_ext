use serde::Deserialize;

use crate::tests::support::layer;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct BackendConfig {
    batch_size: usize,
}

#[test]
fn layer_config_deserializes_backend_schema() {
    let mut layer = layer("backend", "test", "info");
    layer.config = serde_value::to_value(serde_value::Value::Map(
        [(
            serde_value::Value::String("batch_size".to_owned()),
            serde_value::Value::U64(64),
        )]
        .into_iter()
        .collect(),
    ))
    .expect("serialize backend config");

    assert_eq!(
        layer
            .deserialize::<BackendConfig>()
            .expect("deserialize config"),
        BackendConfig { batch_size: 64 }
    );
}
