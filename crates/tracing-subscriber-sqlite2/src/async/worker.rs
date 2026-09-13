use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::Config;
use crate::EventRecord;
use crate::schema::TableNames;

pub(crate) enum Command {
    Event(EventRecord),
    Flush(oneshot::Sender<()>),
}

pub(crate) async fn run(
    connection: tokio_rusqlite::Connection,
    mut receiver: mpsc::Receiver<Command>,
    config: Config,
    tables: TableNames,
    write_errors: Arc<AtomicU64>,
) {
    let mut buffer = Vec::with_capacity(config.batch_size);
    loop {
        match tokio::time::timeout(config.flush_interval, receiver.recv()).await {
            Ok(Some(Command::Event(event))) => {
                buffer.push(event);
                if buffer.len() >= config.batch_size {
                    flush(&connection, &mut buffer, &tables, &write_errors).await;
                }
            }
            Ok(Some(Command::Flush(reply))) => {
                flush(&connection, &mut buffer, &tables, &write_errors).await;
                let _ = reply.send(());
            }
            Ok(None) => {
                flush(&connection, &mut buffer, &tables, &write_errors).await;
                break;
            }
            Err(_) => flush(&connection, &mut buffer, &tables, &write_errors).await,
        }
    }
}

async fn flush(
    connection: &tokio_rusqlite::Connection,
    buffer: &mut Vec<EventRecord>,
    tables: &TableNames,
    write_errors: &AtomicU64,
) {
    let events = std::mem::take(buffer);
    if events.is_empty() {
        return;
    }
    let tables = tables.clone();
    let result = connection
        .call(move |connection| crate::storage::write_batch(connection, events, &tables))
        .await;
    if result.is_err() {
        write_errors.fetch_add(1, Ordering::Relaxed);
    }
}
