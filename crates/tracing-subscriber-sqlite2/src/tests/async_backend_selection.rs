use super::support::remove;
use super::support::temp_db;
use crate::AsyncSqliteLayer;
use crate::Config;

#[test]
fn async_layer_rejects_sync_backend_config() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build runtime")
        .block_on(async {
            let path = temp_db("async-backend-selection");
            let result = AsyncSqliteLayer::open_with_config(&path, Config::default()).await;

            assert!(result.is_err());
            remove(&path);
        });
}
