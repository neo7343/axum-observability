use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
use opentelemetry::{
    global,
    runtime::Tokio,
    sdk::{
        export::metrics::aggregation::cumulative_temporality_selector,
        metrics::{controllers::BasicController, selectors},
        Resource,
    },
    Context, KeyValue,
};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use std::time::{Duration, Instant};

pub fn setup_meter(
    resource: Resource,
    export_config: ExportConfig,
) -> anyhow::Result<BasicController> {
    let controller = opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::histogram([0.1, 0.5, 1.0, 5.0, 10.0]),
            cumulative_temporality_selector(),
            Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
            // can also config it using with_* functions like the tracing part above.
        )
        .with_resource(resource)
        .with_period(Duration::from_secs(5))
        .with_timeout(Duration::from_secs(10))
        .build()?;
    controller.start(&Context::current(), Tokio)?;
    Ok(controller)
}

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

    // get a meter from a provider
    let meter = global::meter("");

    let histogram = meter.f64_histogram("http.server.duration").init();

    histogram.record(
        &Context::current(),
        latency,
        &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("status", status),
            KeyValue::new("path", path),
        ],
    );
    response
}
