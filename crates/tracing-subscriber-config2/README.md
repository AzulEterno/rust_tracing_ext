# tracing-subscriber-config2

Serializable configuration and runtime composition for
`tracing_subscriber::Layer` factories.

The crate provides:

- named tracing layers with independent filters;
- synchronous and asynchronous layer factories;
- built-in stdout and append-only file factories;
- one-time global subscriber installation;
- runtime filter reload handles;
- resource guards that keep backend workers alive.

## Example

```rust
use tracing_subscriber_config2::config::{LayerConfig, TracingConfig};
use tracing_subscriber_config2::factory::FmtLayerFactory;
use tracing_subscriber_config2::runtime::TracingBuilder;

let mut builder = TracingBuilder::new();
builder.register(FmtLayerFactory)?;

let handle = builder
    .prepare(TracingConfig {
        enabled: true,
        layers: vec![LayerConfig {
            name: "console".into(),
            kind: "fmt".into(),
            filter: "info,my_app=debug".into(),
            config: serde_value::Value::Unit,
        }],
    })?
    .install()?;

handle.reload_filter("console", "warn")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Install the global tracing subscriber only once. Keep the returned handle alive
for as long as backend resource guards and filter reloads are needed.
