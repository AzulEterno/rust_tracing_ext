/// Stored event severity, ordered from least to most severe.
#[repr(i64)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
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
    pub timestamp_unix_nanos: i64,
    pub target: String,
    pub level: EventLevel,
    pub fields: String,
    pub span_scope: Option<String>,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
}
