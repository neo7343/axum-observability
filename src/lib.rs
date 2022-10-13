// mod metrics;
mod metrics;
mod trace;
mod trace_id_format;
use self::trace_id_format::TraceIdFormat;
use axum::middleware as mw;
use axum::Router;

use reqwest_tracing::{DefaultSpanBackend, TracingMiddleware};

mod middleware;

pub use self::trace::*;
use metrics::*;
pub use reqwest_middleware::ClientWithMiddleware;
pub use tracing::{error, info, instrument, warn};

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
