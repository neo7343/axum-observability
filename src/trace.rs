use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};
use opentelemetry_otlp::SpanExporterBuilder;
use opentelemetry_semantic_conventions as semcov;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub fn setup() -> Result<(), anyhow::Error> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or("info".into()),
    );

    let otel_tracer = init_tracer(make_resource(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    ))
    .expect("setup of Tracer");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().event_format(super::TraceIdFormat);
    // .with_timer(tracing_subscriber::fmt::time::uptime());
    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_default_env())
        .with(ErrorLayer::default())
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}

fn init_tracer(resource: Resource) -> Result<sdktrace::Tracer, TraceError> {
    use opentelemetry_otlp::WithExportConfig;

    global::set_text_map_propagator(TraceContextPropagator::new());
    let protocol = std::env::var("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .unwrap_or_else(|_| "http/protobuf".to_string());
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .ok();

    let exporter: SpanExporterBuilder = match protocol.as_str() {
        "http/protobuf" => opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint.unwrap_or_else(|| "http://localhost:4318".to_string())) //Devskim: ignore DS137138
            .into(),
        _ => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint.unwrap_or_else(|| "http://localhost:4317".to_string())) //Devskim: ignore DS137138
            .into(),
    };

    let pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        );

    pipeline.install_batch(opentelemetry::runtime::Tokio)
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
