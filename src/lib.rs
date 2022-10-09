mod metrics;
mod trace;
mod trace_id_format;
use self::metrics::*;
use self::trace_id_format::TraceIdFormat;
use axum::middleware as mw;
use axum::routing::get;
use axum::Router;
use reqwest_tracing::TracingMiddleware;
mod middleware;

pub use self::trace::*;
pub use reqwest_middleware::ClientWithMiddleware;
pub use tracing::{error, info, instrument, warn};

pub fn build_reqwest() -> ClientWithMiddleware {
    reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware)
        .build()
}

pub fn init_router<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + 'static,
{
    let recorder_handle = setup_metrics_recorder();
    router
        .layer(middleware::opentelemetry_tracing_layer())
        .route_layer(mw::from_fn(track_metrics))
        .route("/metrics", get(|| async move { recorder_handle.render() }))
}
