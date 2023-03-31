pub fn set_global_propagator() {
    let propagator = opentelemetry_datadog::DatadogPropagator::new();
    opentelemetry::global::set_text_map_propagator(propagator);
}
