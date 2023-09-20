//! Shutdown utilities.
//!
//! This module re-exposes the shutdown fn provided by [`opentelemetry`] project.
//!
//! [`opentelemetry::global::shutdown_trace_provider`]: https://github.com/open-telemetry/opentelemetry-rust/blob/cf46a55420458bfd74a177cd713681369f01f6eb/opentelemetry/src/global/trace.rs#L407

pub struct TracerShutdown {}

impl TracerShutdown {
    pub fn shutdown(&self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
