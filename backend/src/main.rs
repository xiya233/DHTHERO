mod api;
mod config;
mod crawler;
mod db;
mod domain;
mod error;
mod search;
mod state;

use anyhow::Context;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

use crate::{
    config::AppConfig,
    db::repo,
    search::client::MeiliSearchClient,
    state::{AppState, CrawlerStatus, MeiliStatus},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .context("failed to connect to postgresql")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run sql migrations")?;

    let meili_client = init_meili_client(&config).await;
    let meili_sync_tx = if config.features.meili_enabled && meili_client.is_some() {
        let queue_capacity = config.meili.sync_queue_capacity.max(1);
        let (tx, rx) = mpsc::channel::<String>(queue_capacity);
        search::sync::spawn_incremental_worker(
            pool.clone(),
            Arc::new(config.clone()),
            meili_client.clone(),
            rx,
        );
        Some(tx)
    } else {
        None
    };

    let state = AppState::new(
        pool.clone(),
        Arc::new(config.clone()),
        meili_client,
        meili_sync_tx,
    );

    if config.features.meili_enabled {
        if state.meili_client().is_some() {
            if config.meili.bootstrap_enabled {
                search::sync::spawn_bootstrap_job(state.clone());
            } else {
                state.set_meili_status(MeiliStatus::Ready).await;
            }
        } else {
            state.set_meili_status(MeiliStatus::Failed).await;
            warn!(
                "meili feature enabled but client initialization failed; search will fallback to postgres"
            );
        }
    }

    if config.crawler.enabled {
        if let Err(err) = crawler::spawn(state.clone()).await {
            state.set_crawler_status(CrawlerStatus::Failed).await;
            return Err(err).context("crawler bootstrap failed");
        }
    } else {
        state.set_crawler_status(CrawlerStatus::Disabled).await;
        info!("crawler is disabled by CRAWLER_ENABLED=false");
    }

    spawn_audit_cleanup_job(state.clone());

    let app: Router = api::build_router(state);
    let bind_addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind {bind_addr}"))?;

    let local_addr = listener
        .local_addr()
        .context("failed to resolve listener local addr")?;
    info!("backend listening on http://{}", local_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("axum server failed")?;

    Ok(())
}

async fn init_meili_client(config: &AppConfig) -> Option<Arc<MeiliSearchClient>> {
    if !config.features.meili_enabled {
        return None;
    }

    let client = match MeiliSearchClient::new(&config.meili) {
        Ok(client) => Arc::new(client),
        Err(err) => {
            warn!(error = ?err, "failed to build meilisearch client");
            return None;
        }
    };
    match client.initialize().await {
        Ok(()) => Some(client),
        Err(err) => {
            warn!(error = ?err, "failed to initialize meilisearch client");
            None
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,backend=info,sqlx=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .compact()
        .init();
}

fn spawn_audit_cleanup_job(state: AppState) {
    if !state.config.features.audit_enabled {
        return;
    }

    let pool = state.pool.clone();
    let retention_days = state.config.audit_log_retention_days;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(6 * 60 * 60));
        loop {
            interval.tick().await;
            match repo::purge_old_audit_logs(&pool, retention_days).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(deleted, retention_days, "purged old search audit logs");
                    }
                }
                Err(err) => {
                    error!(error = ?err, "failed to purge old search audit logs");
                }
            }
        }
    });
}
