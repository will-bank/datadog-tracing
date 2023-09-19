use std::net::SocketAddr;
use std::time::Duration;

use axum::{routing::get, Router};
use ddtrace::axum::opentelemetry_tracing_layer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (guard, tracer_shutdown) = ddtrace::init()?;

    let app = Router::new()
        .route("/", get(root))
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3025));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(ddtrace::axum::shutdown_signal(tracer_shutdown))
        .await
        .unwrap();

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
