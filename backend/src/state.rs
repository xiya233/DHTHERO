use crate::{config::AppConfig, search::client::MeiliSearchClient};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CrawlerStatus {
    Disabled,
    Starting,
    Running,
    Failed,
}

impl CrawlerStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeiliStatus {
    Disabled,
    Starting,
    Syncing,
    Ready,
    Failed,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    meili_client: Option<Arc<MeiliSearchClient>>,
    meili_sync_tx: Option<mpsc::Sender<String>>,
    crawler_status: Arc<RwLock<CrawlerStatus>>,
    meili_status: Arc<RwLock<MeiliStatus>>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        config: Arc<AppConfig>,
        meili_client: Option<Arc<MeiliSearchClient>>,
        meili_sync_tx: Option<mpsc::Sender<String>>,
    ) -> Self {
        let initial_status = if config.crawler.enabled {
            CrawlerStatus::Starting
        } else {
            CrawlerStatus::Disabled
        };
        let initial_meili_status = if !config.features.meili_enabled {
            MeiliStatus::Disabled
        } else if meili_client.is_some() {
            MeiliStatus::Starting
        } else {
            MeiliStatus::Failed
        };

        Self {
            pool,
            config,
            meili_client,
            meili_sync_tx,
            crawler_status: Arc::new(RwLock::new(initial_status)),
            meili_status: Arc::new(RwLock::new(initial_meili_status)),
        }
    }

    pub async fn set_crawler_status(&self, status: CrawlerStatus) {
        let mut guard = self.crawler_status.write().await;
        *guard = status;
    }

    pub async fn crawler_status(&self) -> CrawlerStatus {
        *self.crawler_status.read().await
    }

    pub fn meili_client(&self) -> Option<Arc<MeiliSearchClient>> {
        self.meili_client.clone()
    }

    pub async fn set_meili_status(&self, status: MeiliStatus) {
        let mut guard = self.meili_status.write().await;
        *guard = status;
    }

    pub fn enqueue_meili_sync(&self, info_hash: String) -> bool {
        let Some(tx) = &self.meili_sync_tx else {
            return false;
        };

        tx.try_send(info_hash).is_ok()
    }
}
