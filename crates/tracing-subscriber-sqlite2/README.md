# tracing-subscriber-sqlite2

Bounded, batched `tracing_subscriber::Layer` implementations backed by SQLite.
Both synchronous and Tokio-backed layers preserve event fields and span context
while interning repeated strings in a shared table.

## Features

| Feature | Meaning |
|---|---|
| `bundled` | Build bundled SQLite; enabled by default. |
| `async` | Enable `AsyncSqliteLayer` through Tokio and `tokio-rusqlite`. |
| `serde` | Serialize and deserialize SQLite layer configuration. |
| `configurable` | Register SQLite factories with `tracing-subscriber-config2`; enables `serde`. |
| `limits` | Forward Rusqlite's runtime SQLite limit API feature. |

Use system SQLite by disabling default features:

```toml
tracing-subscriber-sqlite2 = {
    version = "0",
    default-features = false,
    features = ["async", "configurable"],
}
```

## Synchronous layer

```rust
use tracing_subscriber::prelude::*;
use tracing_subscriber_sqlite2::SqliteLayer;

let layer = SqliteLayer::open("tracing.sqlite")?;
tracing_subscriber::registry().with(layer).init();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Configuration

`Config` controls the backend mode, table prefix, shared string table, queue
capacity, batch size, and flush interval. Zero queue or batch sizes are raised
to one, and a zero flush interval uses the built-in default.

Call `flush` before shutdown when the application needs all accepted events to
be durable. Queue overflow is non-blocking and observable through
`dropped_count`; worker failures are observable through `write_error_count`.
