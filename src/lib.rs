mod formatter;
mod propagator;
mod tracer;

pub use formatter::TraceIdFormat;
pub use propagator::set_global_propagator;
pub use tracer::{build_tracer, build_layer};
pub use opentelemetry::trace::{TraceError, TraceResult};

#[cfg(feature = "axum")]
pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
