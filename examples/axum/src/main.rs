use std::net::SocketAddr;
use std::time::Duration;
use datadog_tracing::axum::{OtelAxumLayer, OtelInResponseLayer};

use axum::{routing::get, Router};
use tower_http::timeout::TimeoutLayer;
use tokio::net::TcpListener;
use tracing::info;
use datadog_tracing::axum::shutdown_signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_guard, tracer_shutdown) = datadog_tracing::init()?;

    let app = Router::new()
        .route("/", get(root))
        // include trace context as header into the response
        .layer(OtelInResponseLayer)
        //start OpenTelemetry trace on incoming request
        .layer((
            OtelAxumLayer::default(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(90)),
        ))
        .route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3025));
    let listener = TcpListener::bind(addr).await?;

    info!("listening on {}", addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracer_shutdown.shutdown();

    Ok(())
}

async fn root() -> &'static str {
    do_something().await;
    "Hello, World!"
}

#[tracing::instrument]
async fn do_something() {
    tokio::time::sleep(Duration::from_millis(120)).await;
    do_something_else().await;
    tracing::info!("in the middle of doing something");
    tokio::time::sleep(Duration::from_millis(10)).await;
    do_something_else().await;
    tokio::time::sleep(Duration::from_millis(20)).await;
}

#[tracing::instrument]
async fn do_something_else() {
    tokio::time::sleep(Duration::from_millis(40)).await;
}

async fn health() -> &'static str {
    "healthy"
}
