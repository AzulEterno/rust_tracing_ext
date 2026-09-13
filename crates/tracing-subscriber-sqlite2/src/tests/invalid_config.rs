use super::support::remove;
use super::support::temp_db;
use crate::Config;
use crate::SqliteLayer;

#[test]
fn rejects_unsafe_table_identifiers() {
    let path = temp_db("invalid-config");
    let result = SqliteLayer::open_with_config(
        &path,
        Config {
            table_prefix: "events; DROP TABLE strings".to_owned(),
            ..Config::default()
        },
    );

    assert!(result.is_err());
    remove(&path);
}
