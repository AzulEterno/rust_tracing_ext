use crate::Config;

#[derive(Clone, Debug)]
pub(crate) struct TableNames {
    pub(crate) events: String,
    pub(crate) strings: String,
    timestamp_index: String,
    target_index: String,
}

impl TableNames {
    pub(crate) fn new(config: &Config) -> rusqlite::Result<Self> {
        validate_identifier("table_prefix", &config.table_prefix)?;
        validate_identifier("strings_table", &config.strings_table)?;
        Ok(Self {
            events: format!("{}_events", config.table_prefix),
            strings: config.strings_table.clone(),
            timestamp_index: format!("{}_events_timestamp", config.table_prefix),
            target_index: format!("{}_events_target", config.table_prefix),
        })
    }

    pub(crate) fn create_schema(&self) -> String {
        format!(
            r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS "{strings}" (
    id INTEGER PRIMARY KEY,
    value TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS "{events}" (
    id INTEGER PRIMARY KEY,
    timestamp_unix_nanos INTEGER NOT NULL,
    target_id INTEGER NOT NULL REFERENCES "{strings}"(id),
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 5),
    fields TEXT NOT NULL,
    span_scope TEXT,
    module_path_id INTEGER REFERENCES "{strings}"(id),
    file_id INTEGER REFERENCES "{strings}"(id),
    line INTEGER
);
CREATE INDEX IF NOT EXISTS "{timestamp_index}"
    ON "{events}"(timestamp_unix_nanos DESC, id DESC);
CREATE INDEX IF NOT EXISTS "{target_index}" ON "{events}"(target_id);
"#,
            strings = self.strings,
            events = self.events,
            timestamp_index = self.timestamp_index,
            target_index = self.target_index,
        )
    }

    pub(crate) fn insert_event_sql(&self) -> String {
        format!(
            "INSERT INTO \"{}\" (timestamp_unix_nanos, target_id, level, fields, span_scope, module_path_id, file_id, line) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            self.events
        )
    }

    pub(crate) fn insert_string_sql(&self) -> String {
        format!(
            "INSERT OR IGNORE INTO \"{}\" (value) VALUES (?)",
            self.strings
        )
    }

    pub(crate) fn select_string_sql(&self) -> String {
        format!("SELECT id FROM \"{}\" WHERE value = ?", self.strings)
    }
}

fn validate_identifier(label: &str, value: &str) -> rusqlite::Result<()> {
    let mut chars = value.chars();
    let valid = chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidParameterName(format!(
            "invalid {label}: {value}"
        )))
    }
}
