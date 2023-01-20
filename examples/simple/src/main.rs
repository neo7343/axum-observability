use axum::{response::IntoResponse, routing::get, Router};
use serde_json::json;
use std::net::SocketAddr;

// fn init_tracing() {
//     use axum_tracing_opentelemetry::{
//         make_resource,
//         otlp,
//         //stdio,
//     };
//     use tracing_subscriber::filter::EnvFilter;
//     use tracing_subscriber::fmt::format::FmtSpan;
//     use tracing_subscriber::layer::SubscriberExt;
//     std::env::set_var(
//         "RUST_LOG",
//         std::env::var("RUST_LOG")
//             .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
//             .unwrap_or_else(|_| "INFO".to_string()),
//     );

//     let otel_rsrc = make_resource(
//         std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
//         env!("CARGO_PKG_VERSION"),
//     );
//     let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
//     // let otel_tracer =
//     //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())
//     //         .expect("setup of Tracer");
//     let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

//     let fmt_layer = tracing_subscriber::fmt::layer()
//         .json()
//         .with_timer(tracing_subscriber::fmt::time::uptime())
//         .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

//     // Build a subscriber that combines the access log and stdout log
//     // layers.
//     let subscriber = tracing_subscriber::registry()
//         .with(fmt_layer)
//         .with(EnvFilter::from_default_env())
//         .with(otel_layer);
//     tracing::subscriber::set_global_default(subscriber).unwrap();
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = axum_observability::setup_tracer_and_meter()?;
    let app = app();
    // run it
    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/` (with trace)"); //Devskim: ignore DS137138
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/heatlh` (with NO trace)"); //Devskim: ignore DS137138
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    axum_observability::shutdown_tracer();
    axum_observability::stop_meter(controller);
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    let router = Router::new().route("/", get(index)); // request processed inside span

    axum_observability::init_router(router).route("/health", get(health))
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

async fn index() -> impl IntoResponse {
    let trace_id = axum_observability::find_current_trace_id();
    tracing::info!("my_trace_id:{:?}",trace_id);
    axum::Json(json!({ "my_trace_id": trace_id }))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
}
