use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use super::support::count_rows;
use super::support::remove;
use super::support::temp_db;
use crate::AsyncSqliteLayer;

#[test]
fn async_feature_persists_events() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build runtime")
        .block_on(async {
            let path = temp_db("async");
            let layer = AsyncSqliteLayer::open(&path).await.expect("open layer");
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = subscriber.set_default();

            tracing::info!("async event");
            layer.flush().await;

            assert_eq!(count_rows(&path), 1);
            remove(&path);
        });
}
