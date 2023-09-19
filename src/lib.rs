//! Utilities to integrate Rust services with Datadog using [`opentelemetry`],
//! [`tracing`], and other open source libraries.
//!
//! This is an opinionated crate providing the building blocks for a setup that
//! works with Datadog. It has been tested with services using [`axum`] hosted
//! on AWS ECS, with propagation working when requests are made to other services
//! using [`reqwest`].
//!
//! [`axum`]: https://github.com/tokio-rs/axum
//! [`reqwest`]: https://docs.rs/reqwest/latest/reqwest/

#[cfg(feature = "axum")]
pub mod axum;
pub mod formatter;
pub mod tracer;
pub mod init;
pub mod shutdown;

pub use init::init_registry;
pub use opentelemetry::global::shutdown_tracer_provider;