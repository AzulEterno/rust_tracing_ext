use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

use crate::config::LayerConfig;
use crate::factory::{FileLayerConfig, FileLayerFactory, LayerFactory};

#[test]
fn file_factory_filters_by_server_instance_span() {
    let directory = std::env::temp_dir().join(format!(
        "tracing_subscriber_config2_file_factory_{}",
        std::process::id()
    ));
    let path = directory.join("alpha.log");
    let config = LayerConfig {
        name: "alpha".to_owned(),
        kind: "file".to_owned(),
        filter: String::new(),
        config: serde_value::to_value(FileLayerConfig { path: path.clone() })
            .expect("serialize file config"),
    };
    let built = FileLayerFactory.build(&config).expect("build file layer");
    let filter = EnvFilter::try_new("off,[server_instance{name=alpha}]=trace")
        .expect("parse instance filter");
    let subscriber = tracing_subscriber::Registry::default().with(built.layer.with_filter(filter));

    tracing::subscriber::with_default(subscriber, || {
        tracing::info_span!("server_instance", name = "alpha")
            .in_scope(|| tracing::info!("alpha event"));
        tracing::info_span!("server_instance", name = "beta")
            .in_scope(|| tracing::info!("beta event"));
    });

    let output = std::fs::read_to_string(&path).expect("read instance log");
    assert!(output.contains("alpha event"));
    assert!(!output.contains("beta event"));
    std::fs::remove_dir_all(directory).expect("remove test logs");
}
