use axum::response::IntoResponse;
use axum::{http::StatusCode, routing::get, Router};

#[cfg(feature = "metrics")]
use axum_otel_metrics::HttpMetricsLayerBuilder;

use crate::AppState;

// import API endpoints for delta tree if building for delta-tree
// #[cfg(feature="delta-tree")]
// use crate::handlers::delta_tree::{};

// Router configuring all accessible API endpoints
pub fn app_router() -> Router<AppState> {
    let mut router = Router::new();

    // Add default endpoints
    router = router.route("/ping", get(ping));

    router = router.nest("/api/trie", crate::handlers::trie::routes::trie_routes());

    // add 404 error handler
    router = router.fallback(handler_404);

    // add metrics
    #[cfg(feature = "metrics")]
    {
        let metrics = HttpMetricsLayerBuilder::new().build();
        router = router.merge(metrics.routes::<AppState>());
        router = router.layer(metrics);
    }

    router
}

// Ping endpoint for debugging - TODO return DateTime of API server
async fn ping() -> &'static str {
    "Pong!"
}

// 404 - TODO return response in JSON
async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource could not be found.",
    )
}
