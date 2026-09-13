use std::collections::HashMap;

use rusqlite::Transaction;
use rusqlite::params;

use crate::EventRecord;
use crate::schema::TableNames;

pub(crate) fn write_batch(
    connection: &mut rusqlite::Connection,
    events: Vec<EventRecord>,
    tables: &TableNames,
) -> rusqlite::Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let insert_event_sql = tables.insert_event_sql();
    let insert_string_sql = tables.insert_string_sql();
    let select_string_sql = tables.select_string_sql();
    let transaction = connection.transaction()?;
    let mut strings = HashMap::new();
    {
        let mut statement = transaction.prepare(&insert_event_sql)?;
        for event in events {
            let target_id = intern(
                &transaction,
                &mut strings,
                &insert_string_sql,
                &select_string_sql,
                event.target,
            )?;
            let module_path_id = intern_optional(
                &transaction,
                &mut strings,
                &insert_string_sql,
                &select_string_sql,
                event.module_path,
            )?;
            let file_id = intern_optional(
                &transaction,
                &mut strings,
                &insert_string_sql,
                &select_string_sql,
                event.file,
            )?;
            statement.execute(params![
                event.timestamp_unix_nanos,
                target_id,
                event.level as i64,
                event.fields,
                event.span_scope,
                module_path_id,
                file_id,
                event.line,
            ])?;
        }
    }
    transaction.commit()
}

fn intern(
    transaction: &Transaction<'_>,
    cache: &mut HashMap<String, i64>,
    insert_sql: &str,
    select_sql: &str,
    value: String,
) -> rusqlite::Result<i64> {
    if let Some(id) = cache.get(&value) {
        return Ok(*id);
    }
    transaction.execute(insert_sql, [&value])?;
    let id = transaction.query_row(select_sql, [&value], |row| row.get(0))?;
    cache.insert(value, id);
    Ok(id)
}

fn intern_optional(
    transaction: &Transaction<'_>,
    cache: &mut HashMap<String, i64>,
    insert_sql: &str,
    select_sql: &str,
    value: Option<String>,
) -> rusqlite::Result<Option<i64>> {
    value
        .map(|value| intern(transaction, cache, insert_sql, select_sql, value))
        .transpose()
}
