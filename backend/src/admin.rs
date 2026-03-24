use crate::{
    config::AppConfig,
    state::{
        AdminDashboardNow, AdminDashboardSeriesPoint, AdminDashboardSnapshot, AdminDashboardWindow,
        AppState,
    },
};
use chrono::Utc;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::{
    collections::HashMap, collections::VecDeque, net::SocketAddr, sync::Arc, time::Duration,
};
use tracing::{info, warn};

const METRIC_INFO_HASH_DISCOVERED_TOTAL: &str = "dht_info_hashes_discovered_total";
const METRIC_METADATA_SUCCESS_TOTAL: &str = "dht_metadata_fetch_success_total";
const METRIC_METADATA_FAIL_TOTAL: &str = "dht_metadata_fetch_fail_total";
const METRIC_METADATA_QUEUE_SIZE: &str = "dht_metadata_queue_size";
const METRIC_NODE_QUEUE_SIZE: &str = "dht_node_queue_size";
const METRIC_WORKER_PRESSURE: &str = "dht_metadata_worker_pressure";
const METRIC_UDP_PACKETS_RECEIVED_TOTAL: &str = "dht_udp_packets_received_total";
const METRIC_UDP_PACKETS_SENT_TOTAL: &str = "dht_udp_packets_sent_total";

pub fn init_prometheus_exporter(config: &AppConfig) -> Option<Arc<PrometheusHandle>> {
    if !config.crawler.prometheus_enabled {
        return None;
    }

    let addr: SocketAddr = match config.crawler.prometheus_listen_addr.parse() {
        Ok(addr) => addr,
        Err(err) => {
            warn!(
                error = ?err,
                listen_addr = %config.crawler.prometheus_listen_addr,
                "invalid CRAWLER_PROMETHEUS_LISTEN_ADDR"
            );
            return None;
        }
    };

    match PrometheusBuilder::new()
        .with_http_listener(addr)
        .install_recorder()
    {
        Ok(handle) => {
            info!(listen_addr = %addr, "prometheus metrics exporter started");
            Some(Arc::new(handle))
        }
        Err(err) => {
            warn!(error = ?err, "failed to initialize prometheus metrics exporter");
            None
        }
    }
}

pub fn spawn_dashboard_sampler(state: AppState, handle: Option<Arc<PrometheusHandle>>) {
    let sample_interval_secs = state.config.admin.metrics_sample_interval_secs.max(1);
    let history_points = state.config.admin.metrics_history_points.max(1);
    let prometheus_enabled = handle.is_some() && state.config.crawler.prometheus_enabled;

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(sample_interval_secs));
        let mut series = VecDeque::<AdminDashboardSeriesPoint>::with_capacity(history_points);
        let mut last_received_total: Option<f64> = None;
        let mut last_sent_total: Option<f64> = None;
        let mut last_tick_ts = Utc::now();

        loop {
            ticker.tick().await;
            let now_ts = Utc::now();
            let elapsed_secs = now_ts
                .signed_duration_since(last_tick_ts)
                .num_milliseconds()
                .max(1) as f64
                / 1000.0;
            last_tick_ts = now_ts;

            let parsed = if let Some(prometheus) = &handle {
                parse_prometheus_metrics(&prometheus.render())
            } else {
                HashMap::new()
            };

            let received_total = metric_value(&parsed, METRIC_UDP_PACKETS_RECEIVED_TOTAL);
            let sent_total = metric_value(&parsed, METRIC_UDP_PACKETS_SENT_TOTAL);
            let received_rate = rate_per_second(last_received_total, received_total, elapsed_secs);
            let sent_rate = rate_per_second(last_sent_total, sent_total, elapsed_secs);
            last_received_total = Some(received_total);
            last_sent_total = Some(sent_total);

            let crawler_status = state.crawler_status().await.as_str().to_string();

            let point = AdminDashboardSeriesPoint {
                timestamp: now_ts,
                info_hash_discovered_total: metric_value(
                    &parsed,
                    METRIC_INFO_HASH_DISCOVERED_TOTAL,
                ),
                metadata_fetch_success_total: metric_value(&parsed, METRIC_METADATA_SUCCESS_TOTAL),
                metadata_fetch_fail_total: metric_value(&parsed, METRIC_METADATA_FAIL_TOTAL),
                metadata_queue_size: metric_value(&parsed, METRIC_METADATA_QUEUE_SIZE),
                node_queue_size: metric_value(&parsed, METRIC_NODE_QUEUE_SIZE),
                metadata_worker_pressure: metric_value(&parsed, METRIC_WORKER_PRESSURE),
                udp_packets_received_rate: received_rate,
                udp_packets_sent_rate: sent_rate,
            };

            if series.len() >= history_points {
                series.pop_front();
            }
            series.push_back(point);

            let latest = series.back().cloned();
            let now_metrics = latest.unwrap_or(AdminDashboardSeriesPoint {
                timestamp: now_ts,
                info_hash_discovered_total: 0.0,
                metadata_fetch_success_total: 0.0,
                metadata_fetch_fail_total: 0.0,
                metadata_queue_size: 0.0,
                node_queue_size: 0.0,
                metadata_worker_pressure: 0.0,
                udp_packets_received_rate: 0.0,
                udp_packets_sent_rate: 0.0,
            });

            let snapshot = AdminDashboardSnapshot {
                now: AdminDashboardNow {
                    crawler_status,
                    info_hash_discovered_total: now_metrics.info_hash_discovered_total,
                    metadata_fetch_success_total: now_metrics.metadata_fetch_success_total,
                    metadata_fetch_fail_total: now_metrics.metadata_fetch_fail_total,
                    metadata_queue_size: now_metrics.metadata_queue_size,
                    node_queue_size: now_metrics.node_queue_size,
                    metadata_worker_pressure: now_metrics.metadata_worker_pressure,
                    udp_packets_received_total: received_total,
                    udp_packets_sent_total: sent_total,
                    updated_at: now_ts,
                },
                series: series.iter().cloned().collect(),
                window: AdminDashboardWindow {
                    sample_interval_secs,
                    points: series.len(),
                    horizon_secs: sample_interval_secs.saturating_mul(series.len() as u64),
                    prometheus_enabled,
                },
            };

            state.set_admin_dashboard(snapshot).await;
        }
    });
}

fn parse_prometheus_metrics(text: &str) -> HashMap<String, f64> {
    let mut metrics = HashMap::<String, f64>::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_ascii_whitespace();
        let Some(metric_with_labels) = parts.next() else {
            continue;
        };
        let Some(raw_value) = parts.next() else {
            continue;
        };

        let Ok(value) = raw_value.parse::<f64>() else {
            continue;
        };
        if !value.is_finite() {
            continue;
        }

        let metric_name = metric_with_labels
            .split('{')
            .next()
            .unwrap_or(metric_with_labels);

        *metrics.entry(metric_name.to_string()).or_insert(0.0) += value;
    }

    metrics
}

fn metric_value(metrics: &HashMap<String, f64>, key: &str) -> f64 {
    metrics.get(key).copied().unwrap_or(0.0)
}

fn rate_per_second(previous: Option<f64>, current: f64, elapsed_secs: f64) -> f64 {
    let Some(previous) = previous else {
        return 0.0;
    };
    if current < previous || elapsed_secs <= 0.0 {
        return 0.0;
    }

    (current - previous) / elapsed_secs
}
