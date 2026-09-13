use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::remove;
use super::support::temp_db;
use crate::Config;
use crate::SqliteLayer;

#[test]
fn isolated_event_tables_share_metadata_strings() {
    let path = temp_db("strings");
    let audit = SqliteLayer::open_with_config(
        &path,
        Config {
            table_prefix: "audit".to_owned(),
            strings_table: "global_strings".to_owned(),
            ..Config::default()
        },
    )
    .expect("open audit layer");
    let runtime = SqliteLayer::open_with_config(
        &path,
        Config {
            table_prefix: "runtime".to_owned(),
            strings_table: "global_strings".to_owned(),
            ..Config::default()
        },
    )
    .expect("open runtime layer");

    {
        let _guard = tracing_subscriber::registry()
            .with(audit.clone())
            .set_default();
        tracing::info!(target: "shared-target", "audit");
    }
    audit.flush();
    {
        let _guard = tracing_subscriber::registry()
            .with(runtime.clone())
            .set_default();
        tracing::info!(target: "shared-target", "runtime");
    }
    runtime.flush();

    let connection = rusqlite::Connection::open(&path).expect("open sqlite database");
    let string_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM global_strings WHERE value = 'shared-target'",
            [],
            |row| row.get(0),
        )
        .expect("count interned target");
    let audit_target: i64 = connection
        .query_row("SELECT target_id FROM audit_events", [], |row| row.get(0))
        .expect("read audit target");
    let runtime_target: i64 = connection
        .query_row("SELECT target_id FROM runtime_events", [], |row| row.get(0))
        .expect("read runtime target");

    assert_eq!(string_count, 1);
    assert_eq!(audit_target, runtime_target);
    remove(&path);
}
