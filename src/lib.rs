// mod metrics;
mod metrics;
mod trace;
mod trace_id_format;
use std::time::Duration;

use self::trace_id_format::TraceIdFormat;
use axum::middleware as mw;
use axum::Router;
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry::sdk::Resource;
use opentelemetry::Context;
use opentelemetry_otlp::ExportConfig;
use opentelemetry_otlp::Protocol;
use reqwest_tracing::{DefaultSpanBackend, TracingMiddleware};
mod middleware;
use metrics::*;
use opentelemetry_semantic_conventions as semcov;

pub use reqwest_middleware::ClientWithMiddleware;

pub fn build_reqwest() -> ClientWithMiddleware {
    reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::<DefaultSpanBackend>::new())
        .build()
}

pub fn init_router<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + 'static,
{
    router
        .layer(middleware::opentelemetry_tracing_layer())
        .route_layer(mw::from_fn(track_metrics))
}

pub fn setup_tracer_and_meter() -> anyhow::Result<BasicController> {
    let resource = make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    trace::setup(resource.clone(), export_config())?;
    metrics::setup_meter(resource, export_config())
}

pub fn shutdown_tracer() {
    opentelemetry::global::shutdown_tracer_provider();
}

pub fn stop_meter(controller: BasicController) {
    controller.stop(&Context::current()).unwrap();
}

fn export_config() -> ExportConfig {
    let endpoint =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or("http://localhost:4317".to_string());
    tracing::debug!("otel endpoint:{}", endpoint);
    ExportConfig {
        endpoint,
        timeout: Duration::from_secs(3),
        protocol: Protocol::Grpc,
    }
}

fn make_resource<S>(service_name: S, service_version: S) -> Resource
where
    S: Into<String>,
{
    Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(service_name.into()),
        semcov::resource::SERVICE_VERSION.string(service_version.into()),
    ])
}
