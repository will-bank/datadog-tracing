use opentelemetry::global::shutdown_tracer_provider;

struct TracerShutdown {}

impl TracerShutdown {
    pub fn shutdown() {
        shutdown_tracer_provider();
    }
}