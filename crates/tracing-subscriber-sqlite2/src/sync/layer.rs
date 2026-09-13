use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;

use tracing::Event;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::Config;
use crate::EventRecord;
use crate::capture;
use crate::schema::TableNames;
use crate::sync::worker::Command;

/// Synchronous SQLite layer backed by a standard-library worker thread.
///
/// Event callbacks only try to enqueue captured records, so a full queue does
/// not block the subscriber. Clones share the worker, queue, and counters.
/// Dropping the last clone closes the queue after the worker drains pending
/// records; call [`flush`](Self::flush) when shutdown needs an explicit
/// completion point.
pub struct SqliteLayer {
    pub(crate) sender: SyncSender<Command>,
    pub(crate) dropped: Arc<AtomicU64>,
    pub(crate) write_errors: Arc<AtomicU64>,
}

impl Clone for SqliteLayer {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            dropped: Arc::clone(&self.dropped),
            write_errors: Arc::clone(&self.write_errors),
        }
    }
}

impl SqliteLayer {
    /// Opens a database with [`Config::default`](crate::Config::default) for the synchronous backend.
    ///
    /// The required schema is created if it does not already exist.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::open_with_config(path, Config::default())
    }

    /// Opens a synchronous SQLite layer with the supplied storage settings.
    ///
    /// The configuration is normalized before use: zero queue and batch sizes
    /// become one, and a zero flush interval becomes ten seconds. The
    /// configuration's `async_backend` must be `false`.
    pub fn open_with_config(path: impl AsRef<Path>, config: Config) -> rusqlite::Result<Self> {
        let config = config.normalized();
        config.ensure_backend(false)?;
        let tables = TableNames::new(&config)?;
        let mut connection = rusqlite::Connection::open(path)?;
        connection.execute_batch(&tables.create_schema())?;

        let (sender, receiver) = mpsc::sync_channel(config.queue_capacity);
        let write_errors = Arc::new(AtomicU64::new(0));
        let worker_errors = Arc::clone(&write_errors);
        std::thread::spawn(move || {
            crate::sync::worker::run(&mut connection, receiver, config, tables, worker_errors);
        });

        Ok(Self {
            sender,
            dropped: Arc::new(AtomicU64::new(0)),
            write_errors,
        })
    }

    /// Waits until the flush marker is processed by the worker.
    ///
    /// The worker commits all non-empty batches before acknowledging the marker.
    /// Events submitted concurrently with this call may be ordered before or
    /// after the marker. A failed batch is counted by [`write_error_count`](Self::write_error_count)
    /// and does not make this method return an error.
    pub fn flush(&self) {
        let (reply, wait) = mpsc::channel();
        if self.sender.send(Command::Flush(reply)).is_ok() {
            let _ = wait.recv();
        }
    }

    /// Returns the number of event records that could not be queued.
    ///
    /// This includes queue-overflow and disconnected-worker failures. The
    /// counter is shared by all clones of this layer.
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Returns the number of failed non-empty batch writes observed by the worker.
    ///
    /// The count is batch-based, so one failed transaction can represent many
    /// events. The counter is shared by all clones of this layer.
    pub fn write_error_count(&self) -> u64 {
        self.write_errors.load(Ordering::Relaxed)
    }

    pub(crate) fn try_send(&self, record: EventRecord) {
        if self.sender.try_send(Command::Event(record)).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl<S> Layer<S> for SqliteLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &Attributes<'_>,
        id: &Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        capture::on_new_span(attrs, id, ctx);
    }

    fn on_record(
        &self,
        id: &Id,
        values: &Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        capture::on_record(id, values, ctx);
    }

    fn on_event(&self, event: &Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.try_send(capture::event_record(event, ctx));
    }
}
