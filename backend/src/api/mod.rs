pub mod dto;
pub mod handlers;

use crate::state::AppState;
use axum::{Router, http::Method, routing::get};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/healthz", get(handlers::healthz))
        .route("/api/v1/features", get(handlers::features))
        .route("/api/v1/site/stats", get(handlers::site_stats))
        .route("/api/v1/categories", get(handlers::categories))
        .route("/api/v1/search", get(handlers::search))
        .route("/api/v1/admin/dashboard", get(handlers::admin_dashboard))
        .route("/api/v1/latest", get(handlers::latest))
        .route("/api/v1/trending", get(handlers::trending))
        .route(
            "/api/v1/torrents/{info_hash}",
            get(handlers::torrent_detail),
        )
        .route(
            "/api/v1/torrents/{info_hash}/files",
            get(handlers::torrent_files),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
