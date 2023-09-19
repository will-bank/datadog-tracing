//! Trace and layer builders to export traces to the Datadog agent.
//!
//! This module contains a function that builds a tracer with an exporter
//! to send traces to the Datadog agent in batches over gRPC.
//!
//! It also contains a convenience function to build a layer with the tracer.
use std::env;
use std::sync::Arc;
use opentelemetry::sdk::trace::{RandomIdGenerator, Sampler, Tracer};
use opentelemetry::sdk::trace;
pub use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry::global;
use std::time::Duration;
use opentelemetry_datadog::{ApiVersion, DatadogPropagator};
use tracing::Subscriber;
use tracing_opentelemetry::{OpenTelemetryLayer, PreSampledTracer};
use tracing_subscriber::registry::LookupSpan;

pub fn build_tracer() -> TraceResult<Tracer> {
    let service_name = env::var("DD_SERVICE").expect("missing DD_SERVICE");
    let dd_host = env::var("DD_AGENT_HOST").unwrap_or("localhost".to_string());

    // disabling connection reuse with dd-agent to avoid "connection closed from server" errors
    let dd_http_client = reqwest::ClientBuilder::new()
        .pool_idle_timeout(Duration::from_millis(1))
        .build()
        .expect("Could not init datadog http_client");

    let tracer = opentelemetry_datadog::new_pipeline()
        .with_http_client::<reqwest::Client>(Arc::new(dd_http_client))
        .with_service_name(service_name)
        .with_version(ApiVersion::Version05)
        .with_agent_endpoint(format!("http://{dd_host}:8126"))
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(opentelemetry::runtime::Tokio);

    global::set_text_map_propagator(DatadogPropagator::default());

    tracer
}

pub fn build_layer<S>() -> TraceResult<OpenTelemetryLayer<S, Tracer>>
where
    Tracer: opentelemetry::trace::Tracer + PreSampledTracer + 'static,
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer = build_tracer()?;
    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}
