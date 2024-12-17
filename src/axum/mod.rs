//! Axum utilities.
//!
//! Re-exposes the middleware layer OtelInResponseLayer provided by the [`axum-tracing-opentelemetry`] project
//! (https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk).
//!
//! Also, exposes OtelAxumLayer from the same project, but hacked to support datadog.
//!
//! Additionally, a shutdown helper function named `shutdown_signal` is also exposed

mod shutdown;
pub use shutdown::*;

// Exposes OtelAxumLayer and opentelemetry_tracing_layer
mod middleware;
pub use middleware::*;

pub use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;

mod http_server;
