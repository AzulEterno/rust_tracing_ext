# tracing-subscriber-sqlite2

Bounded, batched `tracing_subscriber::Layer` implementations backed by SQLite.
Both synchronous and Tokio-backed layers preserve event fields, span context,
and tracing source metadata while interning repeated strings in a shared table.

## Features

| Feature | Meaning |
|---|---|
| `bundled` | Build bundled SQLite; enabled by default. |
| `async` | Enable `AsyncSqliteLayer` through Tokio and `tokio-rusqlite`. |
| `serde` | Serialize and deserialize SQLite layer configuration. |
| `configurable` | Register SQLite factories with `tracing-subscriber-config2`; enables `serde`. |
| `limits` | Forward Rusqlite's runtime SQLite limit API feature. |

`bundled` is enabled by default. Disable default features to link against the
system SQLite installation. `AsyncSqliteLayerFactory` requires both `async`
and `configurable`; `SqliteLayerFactory` requires `configurable` only.

Use system SQLite by disabling default features:

```toml
tracing-subscriber-sqlite2 = {
    version = "0",
    default-features = false,
    features = ["async", "configurable"],
}
```

Applications using the examples also declare their runtime and subscriber
dependencies directly:

```toml
tracing-subscriber-sqlite2 = { version = "0", features = ["async", "configurable"] }
tracing = "0"
tracing-subscriber = "0"
tokio = { version = "1", features = ["rt", "macros", "time"] }
# Required by the configurable section:
tracing-subscriber-config2 = "0"
serde-value = "0"
```

## Synchronous layer

```rust
use tracing_subscriber::prelude::*;
use tracing_subscriber_sqlite2::SqliteLayer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = SqliteLayer::open("tracing.sqlite")?;
    tracing_subscriber::registry().with(layer.clone()).init();
    tracing::info!("persisted");
    layer.flush();
    Ok(())
}
```

The synchronous layer uses one standard-library worker thread. Its event
callback uses a bounded non-blocking send, so a full queue drops the event and
increments `dropped_count`. `flush` waits for a worker flush marker after
earlier queued batches have been committed; it does not report a failed batch,
so inspect `write_error_count` as well.

## Asynchronous layer

With the `async` feature, `AsyncSqliteLayer` uses a Tokio task and
`tokio-rusqlite`. The event callback remains non-blocking; `flush` is async and
awaits the same worker completion point.

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber_sqlite2::AsyncSqliteLayer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let layer = AsyncSqliteLayer::open(":memory:").await?;
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = subscriber.set_default();
            tracing::info!(request_id = "request-1", "persisted");
            layer.flush().await;
            Ok::<(), Box<dyn std::error::Error>>(())
        })?;
    Ok(())
}
```

This example requires the `async` feature.

## Configuration

`Config` controls the backend mode, table prefix, shared string table, queue
capacity, batch size, and flush interval. Zero queue or batch sizes are raised
to one, and a zero flush interval uses the built-in ten-second default. The
`async_backend` flag must match the selected layer. The default schema uses
`tracing_events`, `tracing_strings`, and timestamp/target indexes derived from
the `tracing` table prefix; identifiers must start with `_` or an ASCII letter
and then contain only ASCII letters, digits, and underscores.

Call `flush` before shutdown when the application needs all accepted events to
reach the worker. Queue overflow is non-blocking and observable through
`dropped_count`; worker failures are observable through `write_error_count`.

## Configurable factory integration

With `configurable`, serialize `SqliteFactoryConfig` into a
`tracing-subscriber-config2::config::LayerConfig`. Register
`SqliteLayerFactory` for synchronous preparation, or register
`AsyncSqliteLayerFactory` with `TracingBuilder::register_async` when using
`prepare_async`. The factory configuration flattens the `Config` fields next
to `path`, so it has one canonical schema for both direct and configurable
construction.
