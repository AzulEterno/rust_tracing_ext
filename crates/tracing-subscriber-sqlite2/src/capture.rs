use tracing::Event;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::registry::LookupSpan;

use crate::EventLevel;
use crate::EventRecord;
use crate::format::SpanContext;
use crate::format::append_fields;
use crate::format::format_fields;
use crate::format::format_scope;

pub(crate) fn on_new_span<S>(
    attrs: &Attributes<'_>,
    id: &Id,
    ctx: tracing_subscriber::layer::Context<'_, S>,
) where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    if let Some(span) = ctx.span(id) {
        span.extensions_mut().insert(SpanContext {
            name: span.metadata().name().to_owned(),
            fields: format_fields(attrs),
        });
    }
}

pub(crate) fn on_record<S>(
    id: &Id,
    values: &Record<'_>,
    ctx: tracing_subscriber::layer::Context<'_, S>,
) where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    if let Some(span) = ctx.span(id) {
        let mut extensions = span.extensions_mut();
        if let Some(context) = extensions.get_mut::<SpanContext>() {
            append_fields(&mut context.fields, values);
        } else {
            extensions.insert(SpanContext {
                name: span.metadata().name().to_owned(),
                fields: format_fields(values),
            });
        }
    }
}

pub(crate) fn event_record<S>(
    event: &Event<'_>,
    ctx: tracing_subscriber::layer::Context<'_, S>,
) -> EventRecord
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let metadata = event.metadata();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    EventRecord {
        timestamp_unix_nanos: i64::try_from(now.as_nanos()).unwrap_or(i64::MAX),
        target: metadata.target().to_owned(),
        level: EventLevel::from(metadata.level()),
        fields: format_fields(event),
        span_scope: format_scope(event, &ctx),
        module_path: metadata.module_path().map(str::to_owned),
        file: metadata.file().map(str::to_owned),
        line: metadata.line().map(i64::from),
    }
}
