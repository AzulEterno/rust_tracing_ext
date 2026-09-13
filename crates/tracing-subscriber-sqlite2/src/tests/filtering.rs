use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::count_rows;
use super::support::remove;
use super::support::temp_db;
use crate::SqliteLayer;

#[test]
fn standard_layer_filter_is_applied_per_layer() {
    let path = temp_db("filter");
    let layer = SqliteLayer::open(&path).expect("open layer");
    let subscriber = tracing_subscriber::registry().with(
        layer
            .clone()
            .with_filter(Targets::new().with_target("accepted", tracing::Level::INFO)),
    );
    let _guard = subscriber.set_default();

    tracing::info!(target: "accepted", "kept");
    tracing::info!(target: "rejected", "filtered");
    layer.flush();

    assert_eq!(count_rows(&path), 1);
    remove(&path);
}
