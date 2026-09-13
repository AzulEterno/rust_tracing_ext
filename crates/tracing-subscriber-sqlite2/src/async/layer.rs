use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::Event;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::Config;
use crate::EventRecord;
use crate::async_mode::worker::Command;
use crate::capture;
use crate::schema::TableNames;

/// Async SQLite layer backed by Tokio and `tokio-rusqlite`.
///
/// This type is available with the `async` feature. Event callbacks try to
/// enqueue records without awaiting; a full queue increments the dropped
/// counter. The worker is a Tokio task, and SQLite calls are dispatched through
/// `tokio-rusqlite`. Clones share the worker, queue, and counters. When the last
/// clone is dropped, the worker drains queued records and then stops.
pub struct AsyncSqliteLayer {
    sender: mpsc::Sender<Command>,
    dropped: Arc<AtomicU64>,
    write_errors: Arc<AtomicU64>,
}

impl Clone for AsyncSqliteLayer {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            dropped: Arc::clone(&self.dropped),
            write_errors: Arc::clone(&self.write_errors),
        }
    }
}

impl AsyncSqliteLayer {
    /// Opens a database with default settings for the async backend.
    ///
    /// The required schema is created if it does not already exist.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, tokio_rusqlite::Error> {
        Self::open_with_config(
            path,
            Config {
                async_backend: true,
                ..Config::default()
            },
        )
        .await
    }

    /// Opens an async SQLite layer with the supplied storage settings.
    ///
    /// The configuration is normalized before use: zero queue and batch sizes
    /// become one, and a zero flush interval becomes ten seconds. The
    /// configuration's `async_backend` must be `true`.
    pub async fn open_with_config(
        path: impl AsRef<Path>,
        config: Config,
    ) -> Result<Self, tokio_rusqlite::Error> {
        let config = config.normalized();
        config
            .ensure_backend(true)
            .map_err(tokio_rusqlite::Error::from)?;
        let tables = TableNames::new(&config).map_err(tokio_rusqlite::Error::from)?;
        let schema = tables.create_schema();
        let connection = tokio_rusqlite::Connection::open(path).await?;
        connection
            .call(move |connection| {
                connection.execute_batch(&schema)?;
                Ok(())
            })
            .await?;

        let (sender, receiver) = mpsc::channel(config.queue_capacity);
        let write_errors = Arc::new(AtomicU64::new(0));
        tokio::spawn(crate::async_mode::worker::run(
            connection,
            receiver,
            config,
            tables,
            Arc::clone(&write_errors),
        ));

        Ok(Self {
            sender,
            dropped: Arc::new(AtomicU64::new(0)),
            write_errors,
        })
    }

    /// Awaits the flush marker after the async worker commits earlier batches.
    ///
    /// Events submitted concurrently with this call may be ordered before or
    /// after the marker. A failed batch is counted by [`write_error_count`](Self::write_error_count)
    /// and does not make this method return an error.
    pub async fn flush(&self) {
        let (reply, wait) = oneshot::channel();
        if self.sender.send(Command::Flush(reply)).await.is_ok() {
            let _ = wait.await;
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

    fn try_send(&self, record: EventRecord) {
        if self.sender.try_send(Command::Event(record)).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl<S> Layer<S> for AsyncSqliteLayer
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
