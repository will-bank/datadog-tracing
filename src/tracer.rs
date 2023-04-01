//! Trace and layer builders to export traces to the Datadog agent.
//!
//! This module contains a function that builds a tracer with an exporter
//! to send traces to the Datadog agent in batches over gRPC.
//!
//! It also contains a convenience function to build a layer with the tracer.
use opentelemetry::sdk::trace::{RandomIdGenerator, Sampler, Tracer};
use opentelemetry::sdk::{trace, Resource};
pub use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;
use tracing::Subscriber;
use tracing_opentelemetry::{OpenTelemetryLayer, PreSampledTracer};
use tracing_subscriber::registry::LookupSpan;

pub fn build_tracer(service_name: &str) -> TraceResult<Tracer> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_timeout(Duration::from_secs(3));

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    service_name.to_string(),
                )]))
                .with_id_generator(RandomIdGenerator::default()),
        )
        .with_exporter(exporter)
        .install_batch(opentelemetry::runtime::Tokio)
}

pub fn build_layer<S>(service_name: &str) -> TraceResult<OpenTelemetryLayer<S, Tracer>>
where
    Tracer: opentelemetry::trace::Tracer + PreSampledTracer + 'static,
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer = build_tracer(service_name)?;
    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}
