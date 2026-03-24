use crate::{config::AppConfig, search::client::MeiliSearchClient};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::{collections::BTreeMap, sync::Arc};
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

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardNow {
    pub crawler_status: String,
    pub info_hash_discovered_total: f64,
    pub metadata_fetch_success_total: f64,
    pub metadata_fetch_fail_total: f64,
    pub metadata_success_rate: f64,
    pub metadata_queue_size: f64,
    pub node_queue_size: f64,
    pub metadata_worker_pressure: f64,
    pub udp_packets_received_total: f64,
    pub udp_packets_sent_total: f64,
    pub udp_receive_drop_rate: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardSeries {
    pub id: String,
    pub label: String,
    pub labels: BTreeMap<String, String>,
    pub points: Vec<AdminDashboardPoint>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminChartRenderKind {
    Lines,
    Stacked,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardChart {
    pub id: String,
    pub title: String,
    pub unit: String,
    pub render: AdminChartRenderKind,
    pub series: Vec<AdminDashboardSeries>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardTab {
    pub id: String,
    pub title: String,
    pub charts: Vec<AdminDashboardChart>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardWindow {
    pub sample_interval_secs: u64,
    pub points: usize,
    pub horizon_secs: u64,
    pub prometheus_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardSnapshot {
    pub now: AdminDashboardNow,
    pub window: AdminDashboardWindow,
    pub tabs: Vec<AdminDashboardTab>,
}

impl AdminDashboardSnapshot {
    pub fn empty(
        crawler_status: &str,
        sample_interval_secs: u64,
        history_points: usize,
        prometheus_enabled: bool,
    ) -> Self {
        Self {
            now: AdminDashboardNow {
                crawler_status: crawler_status.to_string(),
                info_hash_discovered_total: 0.0,
                metadata_fetch_success_total: 0.0,
                metadata_fetch_fail_total: 0.0,
                metadata_success_rate: 0.0,
                metadata_queue_size: 0.0,
                node_queue_size: 0.0,
                metadata_worker_pressure: 0.0,
                udp_packets_received_total: 0.0,
                udp_packets_sent_total: 0.0,
                udp_receive_drop_rate: 0.0,
                updated_at: Utc::now(),
            },
            window: AdminDashboardWindow {
                sample_interval_secs,
                points: 0,
                horizon_secs: sample_interval_secs.saturating_mul(history_points as u64),
                prometheus_enabled,
            },
            tabs: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    meili_client: Option<Arc<MeiliSearchClient>>,
    meili_sync_tx: Option<mpsc::Sender<String>>,
    crawler_status: Arc<RwLock<CrawlerStatus>>,
    meili_status: Arc<RwLock<MeiliStatus>>,
    admin_dashboard: Arc<RwLock<AdminDashboardSnapshot>>,
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
        let metrics_sample_interval_secs = config.admin.metrics_sample_interval_secs.max(1);
        let metrics_history_points = config.admin.metrics_history_points.max(1);
        let initial_dashboard = AdminDashboardSnapshot::empty(
            initial_status.as_str(),
            metrics_sample_interval_secs,
            metrics_history_points,
            false,
        );

        Self {
            pool,
            config,
            meili_client,
            meili_sync_tx,
            crawler_status: Arc::new(RwLock::new(initial_status)),
            meili_status: Arc::new(RwLock::new(initial_meili_status)),
            admin_dashboard: Arc::new(RwLock::new(initial_dashboard)),
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

    pub async fn set_admin_dashboard(&self, snapshot: AdminDashboardSnapshot) {
        let mut guard = self.admin_dashboard.write().await;
        *guard = snapshot;
    }

    pub async fn admin_dashboard(&self) -> AdminDashboardSnapshot {
        self.admin_dashboard.read().await.clone()
    }
}
