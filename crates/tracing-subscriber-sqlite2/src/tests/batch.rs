use std::time::Duration;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::count_rows;
use super::support::remove;
use super::support::temp_db;
use super::support::wait_for_rows;
use crate::Config;
use crate::SqliteLayer;

#[test]
fn batch_size_flushes_without_explicit_flush() {
    let path = temp_db("batch");
    let layer = SqliteLayer::open_with_config(
        &path,
        Config {
            queue_capacity: 4,
            batch_size: 2,
            flush_interval: Duration::from_secs(60),
            ..Config::default()
        },
    )
    .expect("open layer");
    let subscriber = tracing_subscriber::registry().with(layer);
    let _guard = subscriber.set_default();

    tracing::info!("first");
    tracing::info!("second");
    wait_for_rows(&path, 2);

    assert_eq!(count_rows(&path), 2);
    remove(&path);
}
