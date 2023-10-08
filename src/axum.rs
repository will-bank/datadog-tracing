//! Axum utilities.
//!
//! This module re-exposes the middleware layers provided by the
//! [`axum-tracing-opentelemetry`] project.
//!
//! [`axum-tracing-opentelemetry`]: https://github.com/davidB/axum-tracing-opentelemetry

use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use crate::shutdown::TracerShutdown;
use tokio::signal;

pub fn opentelemetry_tracing_layer() -> OtelAxumLayer {
    OtelAxumLayer::default()
}

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
