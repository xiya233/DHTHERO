use crate::config::AppConfig;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

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

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    crawler_status: Arc<RwLock<CrawlerStatus>>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Arc<AppConfig>) -> Self {
        let initial_status = if config.crawler.enabled {
            CrawlerStatus::Starting
        } else {
            CrawlerStatus::Disabled
        };

        Self {
            pool,
            config,
            crawler_status: Arc::new(RwLock::new(initial_status)),
        }
    }

    pub async fn set_crawler_status(&self, status: CrawlerStatus) {
        let mut guard = self.crawler_status.write().await;
        *guard = status;
    }

    pub async fn crawler_status(&self) -> CrawlerStatus {
        *self.crawler_status.read().await
    }
}
