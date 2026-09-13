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

    pub async fn flush(&self) {
        let (reply, wait) = oneshot::channel();
        if self.sender.send(Command::Flush(reply)).await.is_ok() {
            let _ = wait.await;
        }
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

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
