This repository is home to a Rust crate `ddtrace` with various Datadog
utilities for tracing and logging in Rust.

# Background

Datadog has official support for Python, which includes various SDKs and
other utilities (such as the Python `ddtrace` library)
for tracing and logging in Python applications.

They don't have similar support for Rust. However, they do support the
[OpenTelemetry](https://opentelemetry.io/) format for both logs and traces.
This crate contains the necessary glue to bridge the gap between OpenTelemetry
and Datadog.

# Features
## Tracing
For traces, the official Datadog agent
[can ingest OTel trace data](https://docs.datadoghq.com/opentelemetry/)
with the correct environment variable settings. The traces can be sent 
via either HTTP or gRPC. More information on this can be found here:
https://docs.datadoghq.com/opentelemetry/otlp_ingest_in_the_agent/?tab=docker

OpenTelemetry has an official Rust crate with extensions for major 
formats/providers. This includes a datadog exporter. We have found
this exporter to be less reliable than the standard OTel exporter
sending data to the OTel endpoint of the Datadog agent, though.

This library provides utilities to set up the Rust `tracing` crate
for sending data to the agent in the correct way.

## Logging
Datadog can ingest OpenTelemetry logs with two caveats - 
it expects the `dd.trace_id` and `dd.span_id` attributes
to be set, and it expects a slightly different format for
trace ID.

This crate contains a JSON formatter layer that also correctly
transform the trace ID to the Datadog native format.

## Propagation
The Python library takes care of propagation of the trace context automatically.
Unfortunately, we need to do this manually in Rust. There are many protocols and
corresponding libraries we could support, but the main one is HTTP requests.

In Rust, `reqwest` is the most commonly used HTTP client crate. We provide a 
`reqwest` middleware that injects the necessary headers using the Datadog native
propagation standard (common alternatives would be Jaeger and B3, more on this:
https://opentelemetry.io/docs/reference/specification/context/api-propagators/#propagators-distribution).

## Axum Support
The trace context propagated from other services needs to be extracted and injected
into the propagator. For `axum`, our choice of HTTP API framework, a third-party crate
exists that supports this: https://github.com/davidB/axum-tracing-opentelemetry.

We re-expose this library for convenience.

# Full Example

TODO