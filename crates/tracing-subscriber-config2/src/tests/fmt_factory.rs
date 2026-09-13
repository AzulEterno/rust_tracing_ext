use std::collections::BTreeMap;

use crate::config::{LayerConfig, TracingConfig};
use crate::factory::FmtLayerFactory;
use crate::runtime::TracingBuilder;

#[test]
fn fmt_factory_builds_a_configured_layer() {
    TracingBuilder::new()
        .register(FmtLayerFactory)
        .expect("register fmt factory")
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![LayerConfig {
                name: "stdout".to_owned(),
                kind: "fmt".to_owned(),
                filter: "info".to_owned(),
                config: serde_value::Value::Map(BTreeMap::new()),
            }],
        })
        .expect("prepare fmt layer");
}
