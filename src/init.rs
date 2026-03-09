use std::env;
#[cfg(feature = "datadog")]
use tracing_appender::non_blocking::NonBlocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

#[cfg(feature = "datadog")]
use crate::formatter::DatadogFormatter;
#[cfg(feature = "datadog")]
use crate::shutdown::TracerShutdown;
#[cfg(feature = "datadog")]
use crate::tracer::build_tracer_provider;
#[cfg(feature = "datadog")]
use opentelemetry::trace::TracerProvider;
#[cfg(feature = "datadog")]
use opentelemetry_sdk::trace::TraceError;
#[cfg(feature = "datadog")]
use tracing::Subscriber;
#[cfg(feature = "datadog")]
use tracing_subscriber::registry::LookupSpan;
#[cfg(feature = "datadog")]
use tracing_subscriber::Layer;

fn loglevel_filter_layer(#[allow(unused)] dd_enabled: bool) -> EnvFilter {
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    #[cfg(feature = "datadog")]
    {
        let axum_tracing_log_level = env::var("AXUM_TRACING_LOG_LEVEL").unwrap_or_else(|_| {
            if dd_enabled {
                "trace".to_string()
            } else {
                "off".to_string()
            }
        });
        let otel_log_level = env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
        unsafe {
            env::set_var(
                "RUST_LOG",
                format!("{log_level},otel::tracing={axum_tracing_log_level},otel={otel_log_level}"),
            );
        }
    }

    #[cfg(not(feature = "datadog"))]
    {
        unsafe {
            env::set_var("RUST_LOG", &log_level);
        }
    }

    EnvFilter::from_default_env()
}

#[cfg(feature = "datadog")]
fn log_layer<S>(
    dd_enabled: bool,
    non_blocking: NonBlocking,
) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if dd_enabled {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .event_format(DatadogFormatter)
                .with_writer(non_blocking),
        )
    } else {
        Box::new(tracing_subscriber::fmt::layer().with_writer(non_blocking))
    }
}

/// Initialize tracing subscriber.
///
/// With the `datadog` feature enabled, checks `DD_ENABLED` env var and optionally
/// sets up OpenTelemetry export to Datadog agent.
///
/// Without the `datadog` feature, sets up a plain human-readable fmt subscriber
/// with `RUST_LOG` env filter.
#[cfg(feature = "datadog")]
pub fn init() -> Result<(WorkerGuard, TracerShutdown), TraceError> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
    let dd_enabled = env::var("DD_ENABLED").map(|s| s == "true").unwrap_or(false);

    if dd_enabled {
        let provider = build_tracer_provider()?;
        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(provider.tracer("DataDogTelemetry"));
        Registry::default()
            .with(loglevel_filter_layer(dd_enabled))
            .with(log_layer(dd_enabled, non_blocking))
            .with(telemetry_layer)
            .init();
        Ok((
            guard,
            TracerShutdown {
                tracer_provider: Some(provider),
            },
        ))
    } else {
        Registry::default()
            .with(loglevel_filter_layer(dd_enabled))
            .with(log_layer(dd_enabled, non_blocking))
            .init();
        Ok((
            guard,
            TracerShutdown {
                tracer_provider: None,
            },
        ))
    }
}

#[cfg(not(feature = "datadog"))]
pub fn init() -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
    Registry::default()
        .with(loglevel_filter_layer(false))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
    Ok(guard)
}
