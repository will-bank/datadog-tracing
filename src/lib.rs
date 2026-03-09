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
#[cfg(feature = "datadog")]
pub mod formatter;
pub mod init;
#[cfg(feature = "datadog")]
pub mod shutdown;
#[cfg(feature = "datadog")]
pub mod tracer;

pub use init::{init, init_with_target, OutputTarget};
