//
//! `OpenTelemetry` tracing middleware. Copied from [axum-tracing-opentelemetry v0.16](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/0.16.0/axum-tracing-opentelemetry/src/middleware/trace_extractor.rs)
//!
//! This returns a [`OtelAxumLayer`] configured to use [`OpenTelemetry`'s conventional span field
//! names][otel].
//!
//! # Span fields
//!
//! Try to provide some of the field define at
//! [opentelemetry-specification/.../http.md](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md)
//! (Please report or provide fix for missing one)
//!
//! # Example
//!
//! ```
//! use axum::{Router, routing::get, http::Request};
//! use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
//! use std::net::SocketAddr;
//! use tower::ServiceBuilder;
//!
//! let app = Router::new()
//!     .route("/", get(|| async {}))
//!     .layer(OtelAxumLayer::default());
//!
//! # async {
//! let addr = &"0.0.0.0:3000".parse::<SocketAddr>().unwrap();
//! let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//! axum::serve(listener, app.into_make_service())
//!     .await
//!     .expect("server failed");
//! # };
//! ```
//!

use axum::extract::MatchedPath;
use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::field::Empty;
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http as otel_http;

use crate::axum::http_server;

#[deprecated(
    since = "0.12.0",
    note = "keep for transition, replaced by OtelAxumLayer"
)]
#[must_use]
pub fn opentelemetry_tracing_layer() -> OtelAxumLayer {
    OtelAxumLayer::default()
}

pub type Filter = fn(&str) -> bool;

/// layer/middleware for axum:
///
/// - propagate `OpenTelemetry` context (`trace_id`,...) to server
/// - create a Span for `OpenTelemetry` (and tracing) on call
///
/// `OpenTelemetry` context are extracted from tracing's span.
#[derive(Default, Debug, Clone)]
pub struct OtelAxumLayer {
    filter: Option<Filter>,
}

// add a builder like api
impl OtelAxumLayer {
    #[must_use]
    pub fn filter(self, filter: Filter) -> Self {
        OtelAxumLayer {
            filter: Some(filter),
        }
    }
}

impl<S> Layer<S> for OtelAxumLayer {
    /// The wrapped service
    type Service = OtelAxumService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelAxumService {
            inner,
            filter: self.filter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OtelAxumService<S> {
    inner: S,
    filter: Option<Filter>,
}

impl<S, B, B2> Service<Request<B>> for OtelAxumService<S>
where
    S: Service<Request<B>, Response = Response<B2>> + Clone + Send + 'static,
    S::Error: Error + 'static, //fmt::Display + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // #[allow(clippy::type_complexity)]
    // type Future = futures_core::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        let req = req;
        let span = if self.filter.map_or(true, |f| f(req.uri().path())) {
            let span = http_server::make_span_from_request(&req);

            let route = http_route(&req);
            let method = otel_http::http_method(req.method());

            span.record("http.route", route);
            span.record("otel.name", format!("{method} {route}").trim());

            span.set_parent(otel_http::extract_context(req.headers()));
            span
        } else {
            tracing::Span::none()
        };
        let future = {
            let _ = span.enter();
            self.inner.call(req)
        };
        ResponseFuture {
            inner: future,
            span,
        }
    }
}

pin_project! {
    /// Response future for [`Trace`].
    ///
    /// [`Trace`]: super::Trace
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        // pub(crate) start: Instant,
    }
}

impl<Fut, ResBody, E> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, E>>,
    E: std::error::Error + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures_util::ready!(this.inner.poll(cx));
        http_server::update_span_from_response_or_error(this.span, &result);

        Poll::Ready(result)
    }
}

#[inline]
fn http_route<B>(req: &Request<B>) -> &str {
    req.extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "", |mp| mp.as_str())
}
