use tracing_subscriber_config2::config::LayerConfig;
use tracing_subscriber_config2::config::TracingConfig;
use tracing_subscriber_config2::runtime::TracingBuilder;

use super::support::remove;
use super::support::temp_db;
use crate::Config;
use crate::SqliteFactoryConfig;
use crate::SqliteLayerFactory;

#[test]
fn sync_factory_builds_configured_sqlite_layer() {
    let path = temp_db("sync-factory");
    let mut builder = TracingBuilder::new();
    builder
        .register(SqliteLayerFactory)
        .expect("register SQLite factory");
    let prepared = builder
        .prepare(TracingConfig {
            enabled: true,
            layers: vec![LayerConfig {
                name: "sqlite".to_owned(),
                kind: "sqlite".to_owned(),
                filter: "info".to_owned(),
                config: serde_value::to_value(SqliteFactoryConfig {
                    path: path.clone(),
                    sqlite: Config::default(),
                })
                .expect("serialize factory config"),
            }],
        })
        .expect("prepare SQLite layer");

    assert!(table_exists(&path, "tracing_events"));
    drop(prepared);
    remove(&path);
}

fn table_exists(path: &std::path::Path, table: &str) -> bool {
    rusqlite::Connection::open(path)
        .expect("open sqlite database")
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?)",
            [table],
            |row| row.get(0),
        )
        .expect("query table")
}
