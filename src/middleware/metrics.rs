use axum::{extract::MatchedPath, http::Request, middleware::Next, response::Response};
use opentelemetry::KeyValue;
use std::time::Instant;

pub async fn metrics_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    let start = Instant::now();

    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unknown")
        .to_string();

    let method = request.method().to_string();

    // Record request start
    #[cfg(feature = "metrics")]
    if let Some(metrics) = crate::server::GLOBAL_METRICS.get() {
        metrics.request_counter.add(
            1,
            &[
                KeyValue::new("path", path.clone()),
                KeyValue::new("method", method.clone()),
            ],
        );
    }

    let response = next.run(request).await;

    // Record request completion
    #[cfg(feature = "metrics")]
    if let Some(metrics) = crate::server::GLOBAL_METRICS.get() {
        let duration = start.elapsed().as_secs_f64();
        metrics.request_duration.record(
            duration,
            &[
                KeyValue::new("path", path),
                KeyValue::new("method", method),
                KeyValue::new("status", response.status().as_u16() as i64),
            ],
        );
    }

    response
}
