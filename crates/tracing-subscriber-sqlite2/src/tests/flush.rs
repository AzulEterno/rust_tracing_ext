use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::count_rows;
use super::support::remove;
use super::support::temp_db;
use crate::SqliteLayer;

#[test]
fn flush_persists_accepted_events() {
    let path = temp_db("flush");
    let layer = SqliteLayer::open(&path).expect("open layer");
    let subscriber = tracing_subscriber::registry().with(layer.clone());
    let _guard = subscriber.set_default();

    tracing::info!(request_id = "request-1", "persisted");
    layer.flush();

    assert_eq!(count_rows(&path), 1);
    remove(&path);
}
