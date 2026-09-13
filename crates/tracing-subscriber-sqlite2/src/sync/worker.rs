use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;

use crate::Config;
use crate::EventRecord;
use crate::schema::TableNames;

pub(crate) enum Command {
    Event(EventRecord),
    Flush(mpsc::Sender<()>),
}

pub(crate) fn run(
    connection: &mut rusqlite::Connection,
    receiver: Receiver<Command>,
    config: Config,
    tables: TableNames,
    write_errors: Arc<AtomicU64>,
) {
    let mut buffer = Vec::with_capacity(config.batch_size);
    loop {
        match receiver.recv_timeout(config.flush_interval) {
            Ok(Command::Event(event)) => {
                buffer.push(event);
                if buffer.len() >= config.batch_size {
                    flush(connection, &mut buffer, &tables, &write_errors);
                }
            }
            Ok(Command::Flush(reply)) => {
                flush(connection, &mut buffer, &tables, &write_errors);
                let _ = reply.send(());
            }
            Err(RecvTimeoutError::Timeout) => {
                flush(connection, &mut buffer, &tables, &write_errors);
            }
            Err(RecvTimeoutError::Disconnected) => {
                flush(connection, &mut buffer, &tables, &write_errors);
                break;
            }
        }
    }
}

fn flush(
    connection: &mut rusqlite::Connection,
    buffer: &mut Vec<EventRecord>,
    tables: &TableNames,
    write_errors: &AtomicU64,
) {
    let events = std::mem::take(buffer);
    if crate::storage::write_batch(connection, events, tables).is_err() {
        write_errors.fetch_add(1, Ordering::Relaxed);
    }
}
