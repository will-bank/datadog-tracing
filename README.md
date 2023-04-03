[![crates-badge]](https://crates.io/crates/ddtrace)
[![docs-badge]](https://docs.rs/ddtrace)
[![Crates.io](https://img.shields.io/crates/l/ddtrace)](LICENSE)

Datadog tracing and log correlation for Rust services.

Datadog has official support for Python, which includes various SDKs and
other utilities (such as the Python `ddtrace` library)
for tracing and logging in Python applications.

They don't have similar support for Rust. However, they do support the
[OpenTelemetry](https://opentelemetry.io/) format for both logs and traces.
This crate contains the necessary glue to bridge the gap between OpenTelemetry
and Datadog.

# Features

`ddtrace` has the following features:
1. tracing: utilities for building an OpenTelemetry tracer/layer that sends traces to the Datadog agent
2. log correlation: a log formatter that converts the trace ID and span ID to the Datadog native format and injects them into the `dd.trace_id` and `dd.span_id` fields
   ([more information](https://docs.datadoghq.com/tracing/other_telemetry/connect_logs_and_traces/opentelemetry/))
3. propagation: a utility function to set the Datadog propagator as the global propagator
4. axum (enabled via the `axum` feature): re-exposing the functionality of [axum-tracing-opentelemetry](https://github.com/davidB/axum-tracing-opentelemetry)

# A Complete Example

The following is an example for using `ddtrace` with the `axum` feature enabled
to set up an `axum` service with traces and logs sent to Datadog.

```rust
use std::net::SocketAddr;
use std::time::Duration;

use axum::{routing::get, Router};
use ddtrace::axum::opentelemetry_tracing_layer;
use ddtrace::formatter::DatadogFormatter;
use ddtrace::set_global_propagator;
use ddtrace::tracer::{build_layer, TraceResult};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> TraceResult<()> {
    let service_name = std::env::var("DD_SERVICE").unwrap_or("my-service".to_string());
    let tracing_layer = build_layer(&service_name)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .event_format(DatadogFormatter),
        )
        .with(tracing_layer)
        .init();
    set_global_propagator();

    let app = Router::new()
        .route("/", get(root))
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3025));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(ddtrace::axum::shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn root() -> &'static str {
    do_something().await;
    "Hello, World!"
}

#[tracing::instrument]
async fn do_something() {
    tokio::time::sleep(Duration::from_millis(120)).await;
    do_something_else().await;
    tracing::info!("in the middle of doing something");
    tokio::time::sleep(Duration::from_millis(10)).await;
    do_something_else().await;
    tokio::time::sleep(Duration::from_millis(20)).await;
}

#[tracing::instrument]
async fn do_something_else() {
    tokio::time::sleep(Duration::from_millis(40)).await;
}

async fn health() -> &'static str {
    "healthy"
}
```

Please refer to the complete project with the `Cargo.toml`
[here](https://github.com/Validus-Risk-Management/ddtrace/tree/main/examples/axum).

# Datadog Agent Setup

The Datadog agent needs to be configured to receive OTel traces over gRPC.
Please [refer to the Datadog documentation](https://docs.datadoghq.com/opentelemetry/otlp_ingest_in_the_agent/?tab=docker)
to set up the agent.

# Further Context and Rationale

## Exporting Traces
For traces, the official Datadog agent
[can ingest OTel trace data](https://docs.datadoghq.com/opentelemetry/)
with the correct environment variable settings. The traces can be sent 
via either HTTP or gRPC. More information on this can be found
[here](https://docs.datadoghq.com/opentelemetry/otlp_ingest_in_the_agent/?tab=docker).

OpenTelemetry has an official Rust crate with extensions for major 
formats/providers. This includes a Datadog exporter. We have found
this exporter to be less reliable than the standard OTel exporter
sending data to the OTel endpoint of the Datadog agent, though.
This crate builds on the OTel exporter.

## Propagation

Two commonly used propagation standards are `B3` (OpenZipkin's propagation style)
and Jaeger. OpenTelemetry [supports both](https://opentelemetry.io/docs/reference/specification/context/api-propagators/#propagators-distribution).

Most Datadog SDK's support both `B3` and the Datadog native propagation style.
For example, the Python `ddtrace` library supports `B3` but it
[needs to be explicitly enabled](https://ddtrace.readthedocs.io/en/stable/configuration.html#DD_TRACE_PROPAGATION_STYLE).

For ease of integration with services written in other languages that use the official Datadog SDK,
we opted for sticking with Datadog-style propagation over `B3`. This is set via the
`set_global_propagator` function.


# Reqwest Propagation
The Python library takes care of propagation of the trace context automatically.
Unfortunately, we need to do this manually in Rust.

Arguably, propagation in HTTP requests is the most common need.
This crate does not provide any additional support, but we recommend using
the [reqwest-middleware](https://crates.io/crates/reqwest-middleware) crate
to inject the necessary headers when using `reqwest`.
If you set the global propagator using `ddtrace`, it will work out of the box.

```rust
use ddtrace::set_global_propagator;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;

#[tokio::main]
async fn main() {
    set_global_propagator();
    client = get_http_client();
    
    // configure tracing, setup your app and inject the client
}

fn get_http_client() -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .build()
}
```

[crates-badge]: https://img.shields.io/crates/v/ddtrace.svg
[docs-badge]: https://docs.rs/ddtrace/badge.svg
