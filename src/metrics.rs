use axum::response::IntoResponse;
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry_sdk::metrics::MeterProvider as SdkMeterProvider;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub meter: Meter,
    pub request_counter: Arc<opentelemetry::metrics::Counter<u64>>,
    pub request_duration: Arc<opentelemetry::metrics::Histogram<f64>>,
    pub registry: Arc<prometheus::Registry>,
}

impl Metrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create a Prometheus registry
        let registry = prometheus::Registry::new();

        // Create a PrometheusExporter
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;

        // Create a meter provider
        let provider = SdkMeterProvider::builder().with_reader(exporter).build();

        // Get a meter from the provider
        let meter = provider.meter("konnektoren-vc");

        // Create metrics
        let request_counter = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .init();

        let request_duration = meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .init();

        Ok(Metrics {
            meter,
            request_counter: Arc::new(request_counter),
            request_duration: Arc::new(request_duration),
            registry: Arc::new(registry),
        })
    }

    pub fn gather_metrics(&self) -> String {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

// Simple handler that doesn't need state
pub async fn metrics_handler() -> impl IntoResponse {
    // Create metrics on-demand or use global state
    match Metrics::new() {
        Ok(metrics) => metrics.gather_metrics(),
        Err(_) => "# Metrics unavailable\n".to_string(),
    }
}
