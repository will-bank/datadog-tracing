//! Axum utilities.
//!
//! This module re-exposes the middleware layers provided by the
//! [`axum-tracing-opentelemetry`] project.
//!
//! [`axum-tracing-opentelemetry`]: https://github.com/davidB/axum-tracing-opentelemetry


pub use axum_tracing_opentelemetry::middleware::*;
use tokio::signal;

pub async fn shutdown_signal() {
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
}
