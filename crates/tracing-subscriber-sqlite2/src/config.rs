use std::time::Duration;

const QUEUE_CAPACITY: usize = 2_048;
const BATCH_SIZE: usize = 512;
const FLUSH_INTERVAL: Duration = Duration::from_secs(10);

/// Storage, queue, and batching settings for a SQLite layer.
///
/// [`crate::SqliteLayer::open_with_config`] and
/// [`crate::AsyncSqliteLayer::open_with_config`] normalize zero queue and batch
/// sizes to one and a zero flush interval to ten seconds before opening the
/// database. The constructors also require `async_backend` to match the
/// selected worker.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Selects the Tokio-backed layer when `true`, or the synchronous layer when `false`.
    ///
    /// This value must match the layer constructor or factory being used.
    pub async_backend: bool,
    /// Prefix used for the event table and its indexes.
    ///
    /// For example, `tracing` creates `tracing_events`,
    /// `tracing_events_timestamp`, and `tracing_events_target`.
    /// The identifier must start with `_` or an ASCII letter and continue with
    /// only `_` or ASCII letters and digits.
    pub table_prefix: String,
    /// Name of the shared string-intern table.
    ///
    /// Event targets and optional metadata strings are stored here and
    /// referenced by integer IDs from the event table.
    /// It follows the same identifier rules as [`Self::table_prefix`].
    pub strings_table: String,
    /// Maximum number of events waiting for the worker.
    ///
    /// Event submission is non-blocking; a full queue increments the dropped
    /// counter. A value of zero is normalized to one.
    pub queue_capacity: usize,
    /// Maximum number of events written in one SQLite transaction.
    ///
    /// A value of zero is normalized to one.
    pub batch_size: usize,
    /// Maximum time the worker waits before flushing a non-empty batch.
    ///
    /// A zero duration is normalized to the built-in default of ten seconds.
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
