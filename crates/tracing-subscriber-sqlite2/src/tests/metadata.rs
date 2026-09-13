use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::remove;
use super::support::temp_db;
use crate::EventLevel;
use crate::SqliteLayer;

#[test]
fn stores_tracing_metadata_fields_and_span_scope() {
    let path = temp_db("metadata");
    let layer = SqliteLayer::open(&path).expect("open layer");
    let subscriber = tracing_subscriber::registry().with(layer.clone());
    let _guard = subscriber.set_default();
    let span = tracing::info_span!("request", request_id = 7);
    let _entered = span.enter();

    tracing::warn!(target: "app::worker", answer = 42, "failed");
    layer.flush();

    let connection = rusqlite::Connection::open(&path).expect("open sqlite database");
    let (target, level, fields, scope): (String, i64, String, String) = connection
        .query_row(
            "SELECT strings.value, events.level, events.fields, events.span_scope
             FROM tracing_events AS events
             JOIN tracing_strings AS strings ON strings.id = events.target_id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("read event");

    assert_eq!(target, "app::worker");
    assert_eq!(level, EventLevel::Warn as i64);
    assert!(fields.contains("failed"));
    assert!(fields.contains("42"));
    assert!(scope.contains("request"));
    assert!(scope.contains("request_id=7"));
    remove(&path);
}
