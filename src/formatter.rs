//! An event formatter to emit events in a way that Datadog can correlate them with traces.
//!
//! Datadog's trace ID and span ID format is different from the OpenTelemetry standard.
//! Using this formatter, the trace ID is converted to the correct format.
//! It also adds the trace ID to the `dd.trace_id` field and the span ID to the
//! `dd.span_id` field, which is where Datadog looks for these by default
//! (although the path to the trace ID can be overridden in Datadog).
use std::io;

use chrono::Utc;
use opentelemetry::trace::{SpanId, TraceContextExt, TraceId};
use serde::ser::{SerializeMap, Serializer as _};
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_serde::fields::AsMap;
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::{LookupSpan, SpanRef};

struct TraceInfo {
    trace_id: String,
    span_id: String,
}

fn convert_trace_id(id: TraceId) -> String {
    let bytes = &id.to_bytes()[std::mem::size_of::<u64>()..std::mem::size_of::<u128>()];
    let dd_id = u64::from_be_bytes(bytes.try_into().unwrap());

    dd_id.to_string()
}

fn convert_span_id(id: SpanId) -> String {
    let dd_id = u64::from_be_bytes(id.to_bytes());

    dd_id.to_string()
}

fn lookup_trace_info<S>(span_ref: &SpanRef<S>) -> Option<TraceInfo>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    span_ref.extensions().get::<OtelData>().map(|o| TraceInfo {
        trace_id: convert_trace_id(o.parent_cx.span().span_context().trace_id()),
        span_id: convert_span_id(o.builder.span_id.unwrap_or(SpanId::INVALID)),
    })
}

// mostly stolen from here: https://github.com/tokio-rs/tracing/issues/1531
pub struct DatadogFormatter;

impl<S, N> FormatEvent<S, N> for DatadogFormatter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let meta = event.metadata();

        let mut visit = || {
            let mut serializer = serde_json::Serializer::new(WriteAdaptor::new(&mut writer));
            let mut serializer = serializer.serialize_map(None)?;
            serializer.serialize_entry("timestamp", &Utc::now().to_rfc3339())?;
            serializer.serialize_entry("level", &meta.level().as_serde())?;
            serializer.serialize_entry("fields", &event.field_map())?;
            serializer.serialize_entry("target", meta.target())?;

            if let Some(ref span_ref) = ctx.lookup_current() {
                if let Some(trace_info) = lookup_trace_info(span_ref) {
                    serializer.serialize_entry("dd.span_id", &trace_info.span_id)?;
                    serializer.serialize_entry("dd.trace_id", &trace_info.trace_id)?;
                }
            }

            serializer.end()
        };

        visit().map_err(|_| std::fmt::Error)?;
        writeln!(writer)
    }
}

struct WriteAdaptor<'a> {
    fmt_write: &'a mut dyn std::fmt::Write,
}

impl<'a> WriteAdaptor<'a> {
    fn new(fmt_write: &'a mut dyn std::fmt::Write) -> Self {
        Self { fmt_write }
    }
}

impl<'a> io::Write for WriteAdaptor<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.fmt_write
            .write_str(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(s.as_bytes().len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
