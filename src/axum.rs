//! Axum utilities.
//!
//! This module re-exposes the middleware layers provided by the
//! [`axum-tracing-opentelemetry`] project.
//!
//! [`axum-tracing-opentelemetry`]: https://github.com/davidB/axum-tracing-opentelemetry

pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer_grpc;
use crate::shutdown::TracerShutdown;
use tokio::signal;

pub async fn shutdown_signal(tracer_shutdown: TracerShutdown) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");

    tracer_shutdown.shutdown();
}
