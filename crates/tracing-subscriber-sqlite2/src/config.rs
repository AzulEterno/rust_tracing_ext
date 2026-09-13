use std::time::Duration;

const QUEUE_CAPACITY: usize = 2_048;
const BATCH_SIZE: usize = 512;
const FLUSH_INTERVAL: Duration = Duration::from_secs(10);

/// Storage, queue, and batching settings for a SQLite layer.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Selects the Tokio-backed layer when true, or the synchronous layer when false.
    pub async_backend: bool,
    /// Prefix used for this domain's event table and indexes.
    pub table_prefix: String,
    /// Shared string-intern table name.
    pub strings_table: String,
    pub queue_capacity: usize,
    pub batch_size: usize,
    pub flush_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            async_backend: false,
            table_prefix: "tracing".to_owned(),
            strings_table: "tracing_strings".to_owned(),
            queue_capacity: QUEUE_CAPACITY,
            batch_size: BATCH_SIZE,
            flush_interval: FLUSH_INTERVAL,
        }
    }
}

impl Config {
    pub(crate) fn normalized(self) -> Self {
        Self {
            async_backend: self.async_backend,
            table_prefix: self.table_prefix,
            strings_table: self.strings_table,
            queue_capacity: self.queue_capacity.max(1),
            batch_size: self.batch_size.max(1),
            flush_interval: if self.flush_interval.is_zero() {
                FLUSH_INTERVAL
            } else {
                self.flush_interval
            },
        }
    }

    pub(crate) fn ensure_backend(&self, async_backend: bool) -> rusqlite::Result<()> {
        if self.async_backend == async_backend {
            Ok(())
        } else {
            Err(rusqlite::Error::InvalidParameterName(format!(
                "configured async_backend={} does not match the selected layer",
                self.async_backend
            )))
        }
    }
}
