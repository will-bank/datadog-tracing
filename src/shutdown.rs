//! Shutdown utilities.
//!
//! This module re-exposes the shutdown fn provided by [`opentelemetry`] project.
//!
//! [`opentelemetry::global::shutdown_trace_provider`]: https://github.com/open-telemetry/opentelemetry-rust/blob/cf46a55420458bfd74a177cd713681369f01f6eb/opentelemetry/src/global/trace.rs#L407

use opentelemetry::global::shutdown_tracer_provider;
use tokio::signal;

pub struct TracerShutdown {}

impl TracerShutdown {
    pub fn shutdown(&self) {
        shutdown_tracer_provider();
    }
}

pub async fn handle_signal(tracer_shutdown: TracerShutdown) {
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
