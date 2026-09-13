/// Stored event severity, ordered from least to most severe.
#[repr(i64)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventLevel {
    /// Trace-level event, stored as `1`.
    Trace = 1,
    /// Debug-level event, stored as `2`.
    Debug = 2,
    /// Info-level event, stored as `3`.
    Info = 3,
    /// Warn-level event, stored as `4`.
    Warn = 4,
    /// Error-level event, stored as `5`.
    Error = 5,
}

impl From<&tracing::Level> for EventLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::TRACE => Self::Trace,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::INFO => Self::Info,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::ERROR => Self::Error,
        }
    }
}

/// A captured [`tracing::Event`] and its metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    /// Event timestamp in nanoseconds since the Unix epoch.
    pub timestamp_unix_nanos: i64,
    /// Tracing target associated with the event.
    pub target: String,
    /// Stored severity level.
    pub level: EventLevel,
    /// Event fields formatted by `tracing-subscriber`'s default field formatter.
    pub fields: String,
    /// Root-to-leaf span scope, including recorded span fields when available.
    pub span_scope: Option<String>,
    /// Source module path supplied by tracing metadata, when available.
    pub module_path: Option<String>,
    /// Source file supplied by tracing metadata, when available.
    pub file: Option<String>,
    /// Source line supplied by tracing metadata, when available.
    pub line: Option<i64>,
}
