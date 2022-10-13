use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
use opentelemetry::{global, Context, KeyValue};
use std::time::Instant;

pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    let cx = Context::current();

    // get a meter from a provider
    let meter = global::meter(env!("CARGO_PKG_NAME"));

    let histogram = meter.f64_histogram("http_requests_duration_seconds").init();

    histogram.record(
        &cx,
        latency,
        &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("status", status),
            KeyValue::new("path", path),
        ],
    );
    response
}
