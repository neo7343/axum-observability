use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};

use opentelemetry_otlp::{ExportConfig, WithExportConfig};

use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub fn setup(resource: Resource, export_config: ExportConfig) -> Result<(), anyhow::Error> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or("info".into()),
    );

    let otel_tracer = init_tracer(resource, export_config).expect("setup of Tracer");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().event_format(super::TraceIdFormat);

    // use tracing_subscriber::fmt::format::FmtSpan;
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .json()
    //     .with_timer(tracing_subscriber::fmt::time::uptime())
    //     .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);


    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_default_env())
        .with(ErrorLayer::default())
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn init_tracer(
    resource: Resource,
    export_config: ExportConfig,
) -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}
