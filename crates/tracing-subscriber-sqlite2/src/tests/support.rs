use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn temp_db(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("tracing-subscriber-sqlite2-{name}-{nanos}.sqlite"))
}

pub fn count_rows(path: &std::path::Path) -> i64 {
    let connection = rusqlite::Connection::open(path).expect("open sqlite database");
    connection
        .query_row("SELECT COUNT(*) FROM tracing_events", [], |row| row.get(0))
        .expect("count rows")
}

pub fn wait_for_rows(path: &std::path::Path, expected: i64) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while count_rows(path) < expected {
        assert!(Instant::now() < deadline, "timed out waiting for events");
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub fn remove(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}
