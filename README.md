[![crates-badge]](https://crates.io/crates/datadog_tracing)
[![docs-badge]](https://docs.rs/datadog_tracing)
[![Crates.io](https://img.shields.io/crates/l/datadog_tracing)](LICENSE)

Non-official datadog tracing and log correlation for Rust services.

This crate contains the necessary glue to bridge the gap between OpenTelemetry and Datadog.

# Features

`datadog_tracing` has the following features:
1. tracing: utilities for building an OpenTelemetry tracer/layer that sends traces to the Datadog agent
2. log correlation: a log formatter that converts the trace ID and span ID to the Datadog native format and injects them into the `dd.trace_id` and `dd.span_id` fields
   ([more information](https://docs.datadoghq.com/tracing/other_telemetry/connect_logs_and_traces/opentelemetry/))
3. propagation: a utility function to set the Datadog propagator as the global propagator
4. axum (enabled via the `axum` feature): re-exposing the functionality of [axum-tracing-opentelemetry](https://github.com/davidB/axum-tracing-opentelemetry)

# Configuration

The lib is configurable via environment variables as following:

| env var                | default value                               | description                                               |
|------------------------|---------------------------------------------|-----------------------------------------------------------|
| DD_ENABLED             | false                                       | Enables the datadog exporter and trace_id/span_id on logs |
| DD_SERVICE             | <required>                                  | Datadog service name                                      |
| DD_AGENT_HOST          | localhost                                   | Datadog agent host                                        |
| DD_AGENT_PORT          | 8126                                        | Datadog agent port                                        |
| RUST_LOG               | info                                        |                                                           |
| AXUM_TRACING_LOG_LEVEL | if DD_ENABLED=true, "info", otherwise "off" |                                                           |
| OTEL_LOG_LEVEL         | debug                                       |                                                           |


# Examples

- Check the [axum](examples/axum/src/main.rs) folder for a complete example using axum.
  - Please refer to the `Cargo.toml` [here](https://github.com/will-bank/datadog_tracing/tree/main/examples/axum).

# Further Context and Rationale

## Inspiration

This lib was highly inspired on [datadog_tracing](https://github.com/will-bank/datadog_tracing),
which is also a glue between tracing + opentelemtry + datadog.
The main difference is that it exportes using the opentelemetry_otlp exporter, and this one uses opentelemetry_datadog,
so there is no need to configure your datadog agent to receive traces via OTLP and the default datadog APM works as expected! 


## Propagation

Two commonly used propagation standards are `B3` (OpenZipkin's propagation style)
and Jaeger. OpenTelemetry [supports both](https://opentelemetry.io/docs/reference/specification/context/api-propagators/#propagators-distribution).

Most Datadog SDK's support both `B3` and the Datadog native propagation style.
For example, the Python `datadog_tracing` library supports `B3` but it
[needs to be explicitly enabled](https://datadog_tracing.readthedocs.io/en/stable/configuration.html#DD_TRACE_PROPAGATION_STYLE).

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
If you set the global propagator using `datadog_tracing`, it will work out of the box.

```rust
use datadog_tracing::set_global_propagator;
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

[crates-badge]: https://img.shields.io/crates/v/datadog_tracing.svg
[docs-badge]: https://docs.rs/datadog_tracing/badge.svg
