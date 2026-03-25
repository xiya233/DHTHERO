pub mod dto;
pub mod handlers;

use crate::{error::ApiError, state::AppState};
use axum::{
    Router,
    body::Body,
    http::{Method, Request},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::PUT])
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/healthz", get(handlers::healthz))
        .route("/api/v1/features", get(handlers::features))
        .route("/api/v1/site/stats", get(handlers::site_stats))
        .route("/api/v1/site/content", get(handlers::site_content))
        .route("/api/v1/categories", get(handlers::categories))
        .route("/api/v1/search", get(handlers::search))
        .route("/api/v1/admin/dashboard", get(handlers::admin_dashboard))
        .route(
            "/api/v1/admin/site-settings",
            get(handlers::admin_site_settings_get).put(handlers::admin_site_settings_put),
        )
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            private_mode_guard,
        ))
        .with_state(state)
}

async fn private_mode_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if request.method() == Method::OPTIONS
        || request.uri().path() == "/api/v1/healthz"
        || !state.config.private_mode.is_active()
    {
        return Ok(next.run(request).await);
    }

    let expected_password = state
        .config
        .private_mode
        .site_password
        .as_deref()
        .unwrap_or("");
    let provided_password = request
        .headers()
        .get("x-site-password")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::unauthorized("missing x-site-password header"))?;

    if provided_password != expected_password {
        return Err(ApiError::unauthorized("invalid site password"));
    }

    Ok(next.run(request).await)
}
