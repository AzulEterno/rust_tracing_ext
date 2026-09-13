use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;

use crate::EventLevel;
use crate::EventRecord;
use crate::SqliteLayer;
use crate::sync::worker::Command;

#[test]
fn full_queue_drops_new_events() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let layer = SqliteLayer {
        sender,
        dropped: Arc::new(AtomicU64::new(0)),
        write_errors: Arc::new(AtomicU64::new(0)),
    };
    let event = EventRecord {
        timestamp_unix_nanos: 0,
        target: "test".to_owned(),
        level: EventLevel::Info,
        fields: "message=queued".to_owned(),
        span_scope: None,
        module_path: None,
        file: None,
        line: None,
    };

    layer.try_send(event.clone());
    layer.try_send(event);

    assert_eq!(layer.dropped_count(), 1);
    assert!(matches!(receiver.try_recv(), Ok(Command::Event(_))));
}
