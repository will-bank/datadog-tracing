//! Axum utilities.
//!
//! This module re-exposes the middleware layers provided by the
//! [`axum-tracing-opentelemetry`] project.
//!
//! [`axum-tracing-opentelemetry`]: https://github.com/davidB/axum-tracing-opentelemetry

pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer_grpc;
