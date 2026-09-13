use tracing::Event;
use tracing::span::Record;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::FormattedFields;
use tracing_subscriber::fmt::format::DefaultFields;
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug)]
pub(crate) struct SpanContext {
    pub(crate) name: String,
    pub(crate) fields: String,
}

pub(crate) fn format_scope<S>(
    event: &Event<'_>,
    ctx: &tracing_subscriber::layer::Context<'_, S>,
) -> Option<String>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let scope = ctx.event_scope(event)?;
    let mut output = String::new();
    for span in scope.from_root() {
        if !output.is_empty() {
            output.push(':');
        }
        if let Some(context) = span.extensions().get::<SpanContext>() {
            output.push_str(&context.name);
            if !context.fields.is_empty() {
                output.push('{');
                output.push_str(&context.fields);
                output.push('}');
            }
        } else {
            output.push_str(span.metadata().name());
        }
    }
    (!output.is_empty()).then_some(output)
}

pub(crate) fn format_fields<R>(fields: R) -> String
where
    R: RecordFields,
{
    let formatter = DefaultFields::default();
    let mut formatted = FormattedFields::<DefaultFields>::new(String::new());
    let _ = formatter.format_fields(formatted.as_writer(), fields);
    formatted.fields
}

pub(crate) fn append_fields(fields: &mut String, values: &Record<'_>) {
    let formatter = DefaultFields::default();
    let mut formatted = FormattedFields::<DefaultFields>::new(std::mem::take(fields));
    let _ = formatter.add_fields(&mut formatted, values);
    *fields = formatted.fields;
}
