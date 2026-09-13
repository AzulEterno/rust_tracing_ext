# tracing-subscriber-config2

Serializable configuration and runtime composition for
`tracing_subscriber::Layer` factories.

`TracingBuilder` validates named layers, asks registered factories to build
them, applies an independent `EnvFilter` to each layer, and installs the
resulting subscriber. Synchronous and asynchronous factories can use the same
configuration kind; asynchronous preparation prefers its async factory and
falls back to the synchronous one when no async factory is registered.

The example uses these direct dependencies (Rust does not make transitive
dependencies available to application code):

```toml
tracing-subscriber-config2 = "0"
serde-value = "0"
```

## Example

```rust
use tracing_subscriber_config2::config::{LayerConfig, TracingConfig};
use tracing_subscriber_config2::factory::FmtLayerFactory;
use tracing_subscriber_config2::runtime::TracingBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}
```

`install` sets the process-wide tracing subscriber and can succeed only once.
Preparation validates duplicate names, factory kinds, and filters before
building layers. Keep the returned `TracingHandle` alive for filter reloads and
for backend resource guards; dropping it releases those guards but does not
remove the global subscriber.

## Factories

Implement `LayerFactory` for synchronous setup or `AsyncLayerFactory` for
asynchronous setup. `BuiltLayer::with_guard` keeps a worker or other backend
resource alive until the installed handle is dropped. `FmtLayerFactory` writes
to stdout, while `FileLayerFactory` creates parent directories and appends
formatted events to the configured path.
