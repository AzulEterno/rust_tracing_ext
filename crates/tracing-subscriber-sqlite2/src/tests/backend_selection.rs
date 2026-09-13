use super::support::remove;
use super::support::temp_db;
use crate::Config;
use crate::SqliteLayer;

#[test]
fn sync_layer_rejects_async_backend_config() {
    let path = temp_db("backend-selection");
    let result = SqliteLayer::open_with_config(
        &path,
        Config {
            async_backend: true,
            ..Config::default()
        },
    );

    assert!(result.is_err());
    remove(&path);
}
