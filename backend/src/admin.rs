use crate::{
    config::AppConfig,
    state::{
        AdminChartRenderKind, AdminDashboardChart, AdminDashboardNow, AdminDashboardPoint,
        AdminDashboardSeries, AdminDashboardSnapshot, AdminDashboardTab, AdminDashboardWindow,
        AppState,
    },
};
use chrono::Utc;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tracing::{info, warn};

const METRIC_ANNOUNCE_PEER_BLOCKED_TOTAL: &str = "dht_announce_peer_blocked_total";
const METRIC_INFO_HASH_DISCOVERED_TOTAL: &str = "dht_info_hashes_discovered_total";
const METRIC_MESSAGES_PARSE_ERROR_TOTAL: &str = "dht_messages_parse_error_total";
const METRIC_MESSAGES_PROCESSED_TOTAL: &str = "dht_messages_processed_total";
const METRIC_METADATA_BYTES_DOWNLOADED_TOTAL: &str = "dht_metadata_bytes_downloaded_total";
const METRIC_METADATA_CONNECTION_RESULT_TOTAL: &str = "dht_metadata_connection_result_total";
const METRIC_METADATA_FETCH_ATTEMPTS_TOTAL: &str = "dht_metadata_fetch_attempts_total";
const METRIC_METADATA_FAIL_TOTAL: &str = "dht_metadata_fetch_fail_total";
const METRIC_METADATA_SUCCESS_TOTAL: &str = "dht_metadata_fetch_success_total";
const METRIC_METADATA_HANDSHAKE_RESULT_TOTAL: &str = "dht_metadata_handshake_result_total";
const METRIC_METADATA_QUEUE_SIZE: &str = "dht_metadata_queue_size";
const METRIC_METADATA_SIZE_BYTES: &str = "dht_metadata_size_bytes";
const METRIC_WORKER_PRESSURE: &str = "dht_metadata_worker_pressure";
const METRIC_NODE_QUEUE_SIZE: &str = "dht_node_queue_size";
const METRIC_NODES_DISCOVERED_TOTAL: &str = "dht_nodes_discovered_total";
const METRIC_QUERIES_TOTAL: &str = "dht_queries_total";
const METRIC_UDP_BYTES_RECEIVED_TOTAL: &str = "dht_udp_bytes_received_total";
const METRIC_UDP_BYTES_SENT_TOTAL: &str = "dht_udp_bytes_sent_total";
const METRIC_UDP_PACKETS_RECEIVED_TOTAL: &str = "dht_udp_packets_received_total";
const METRIC_UDP_PACKETS_SENT_TOTAL: &str = "dht_udp_packets_sent_total";

const METRIC_METADATA_SIZE_P50: &str = "dht_metadata_size_bytes_p50";
const METRIC_METADATA_SIZE_P95: &str = "dht_metadata_size_bytes_p95";
const METRIC_METADATA_SIZE_AVG: &str = "dht_metadata_size_bytes_avg";

const COUNTER_METRICS: &[&str] = &[
    METRIC_ANNOUNCE_PEER_BLOCKED_TOTAL,
    METRIC_INFO_HASH_DISCOVERED_TOTAL,
    METRIC_MESSAGES_PARSE_ERROR_TOTAL,
    METRIC_MESSAGES_PROCESSED_TOTAL,
    METRIC_METADATA_BYTES_DOWNLOADED_TOTAL,
    METRIC_METADATA_CONNECTION_RESULT_TOTAL,
    METRIC_METADATA_FETCH_ATTEMPTS_TOTAL,
    METRIC_METADATA_FAIL_TOTAL,
    METRIC_METADATA_SUCCESS_TOTAL,
    METRIC_METADATA_HANDSHAKE_RESULT_TOTAL,
    METRIC_NODES_DISCOVERED_TOTAL,
    METRIC_QUERIES_TOTAL,
    METRIC_UDP_BYTES_RECEIVED_TOTAL,
    METRIC_UDP_BYTES_SENT_TOTAL,
    METRIC_UDP_PACKETS_RECEIVED_TOTAL,
    METRIC_UDP_PACKETS_SENT_TOTAL,
];

const GAUGE_METRICS: &[&str] = &[
    METRIC_METADATA_QUEUE_SIZE,
    METRIC_WORKER_PRESSURE,
    METRIC_NODE_QUEUE_SIZE,
];

#[derive(Clone, Copy)]
struct ChartDefinition {
    id: &'static str,
    title: &'static str,
    unit: &'static str,
    render: AdminChartRenderKind,
    metrics: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct TabDefinition {
    id: &'static str,
    title: &'static str,
    charts: &'static [ChartDefinition],
}

const OVERVIEW_CHARTS: &[ChartDefinition] = &[
    ChartDefinition {
        id: "overview_discovery_rate",
        title: "InfoHash Discovery Rate",
        unit: "events/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_INFO_HASH_DISCOVERED_TOTAL],
    },
    ChartDefinition {
        id: "overview_metadata_outcome_rate",
        title: "Metadata Outcome Rate",
        unit: "events/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_METADATA_SUCCESS_TOTAL, METRIC_METADATA_FAIL_TOTAL],
    },
    ChartDefinition {
        id: "overview_queue_and_pressure",
        title: "Queue And Worker Pressure",
        unit: "value",
        render: AdminChartRenderKind::Lines,
        metrics: &[
            METRIC_METADATA_QUEUE_SIZE,
            METRIC_NODE_QUEUE_SIZE,
            METRIC_WORKER_PRESSURE,
        ],
    },
];

const METADATA_CHARTS: &[ChartDefinition] = &[
    ChartDefinition {
        id: "metadata_fetch_attempts_rate",
        title: "Metadata Fetch Attempts",
        unit: "events/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_METADATA_FETCH_ATTEMPTS_TOTAL],
    },
    ChartDefinition {
        id: "metadata_fail_reason_rate",
        title: "Metadata Fail By Reason",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_METADATA_FAIL_TOTAL],
    },
    ChartDefinition {
        id: "metadata_connection_result_rate",
        title: "Connection Result",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_METADATA_CONNECTION_RESULT_TOTAL],
    },
    ChartDefinition {
        id: "metadata_handshake_result_rate",
        title: "Handshake Result",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_METADATA_HANDSHAKE_RESULT_TOTAL],
    },
    ChartDefinition {
        id: "metadata_download_throughput",
        title: "Metadata Download Throughput",
        unit: "bytes/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_METADATA_BYTES_DOWNLOADED_TOTAL],
    },
    ChartDefinition {
        id: "metadata_size_distribution",
        title: "Metadata Size P50/P95/AVG",
        unit: "bytes",
        render: AdminChartRenderKind::Lines,
        metrics: &[
            METRIC_METADATA_SIZE_P50,
            METRIC_METADATA_SIZE_P95,
            METRIC_METADATA_SIZE_AVG,
        ],
    },
];

const NETWORK_CHARTS: &[ChartDefinition] = &[
    ChartDefinition {
        id: "network_udp_packets_received",
        title: "UDP Packets Received By Status",
        unit: "packets/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_UDP_PACKETS_RECEIVED_TOTAL],
    },
    ChartDefinition {
        id: "network_udp_packets_sent",
        title: "UDP Packets Sent By Type",
        unit: "packets/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_UDP_PACKETS_SENT_TOTAL],
    },
    ChartDefinition {
        id: "network_udp_bytes",
        title: "UDP Bytes Throughput",
        unit: "bytes/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_UDP_BYTES_RECEIVED_TOTAL, METRIC_UDP_BYTES_SENT_TOTAL],
    },
    ChartDefinition {
        id: "network_nodes_discovered",
        title: "Discovered Nodes By IP Version",
        unit: "nodes/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_NODES_DISCOVERED_TOTAL],
    },
];

const PROTOCOL_CHARTS: &[ChartDefinition] = &[
    ChartDefinition {
        id: "protocol_queries",
        title: "DHT Queries By Type",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_QUERIES_TOTAL],
    },
    ChartDefinition {
        id: "protocol_messages",
        title: "DHT Messages Processed",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_MESSAGES_PROCESSED_TOTAL],
    },
    ChartDefinition {
        id: "protocol_parse_errors",
        title: "Message Parse Errors",
        unit: "events/s",
        render: AdminChartRenderKind::Lines,
        metrics: &[METRIC_MESSAGES_PARSE_ERROR_TOTAL],
    },
];

const ERROR_CHARTS: &[ChartDefinition] = &[
    ChartDefinition {
        id: "errors_metadata_fail",
        title: "Metadata Fail Reasons",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_METADATA_FAIL_TOTAL],
    },
    ChartDefinition {
        id: "errors_announce_peer_blocked",
        title: "Announce Peer Blocked",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_ANNOUNCE_PEER_BLOCKED_TOTAL],
    },
    ChartDefinition {
        id: "errors_udp_receive_status",
        title: "UDP Receive Status",
        unit: "events/s",
        render: AdminChartRenderKind::Stacked,
        metrics: &[METRIC_UDP_PACKETS_RECEIVED_TOTAL],
    },
];

const TAB_DEFINITIONS: &[TabDefinition] = &[
    TabDefinition {
        id: "overview",
        title: "Overview",
        charts: OVERVIEW_CHARTS,
    },
    TabDefinition {
        id: "metadata",
        title: "Metadata",
        charts: METADATA_CHARTS,
    },
    TabDefinition {
        id: "network",
        title: "Network",
        charts: NETWORK_CHARTS,
    },
    TabDefinition {
        id: "protocol",
        title: "Protocol",
        charts: PROTOCOL_CHARTS,
    },
    TabDefinition {
        id: "errors",
        title: "Errors",
        charts: ERROR_CHARTS,
    },
];

#[derive(Clone, Debug)]
struct MetricSample {
    name: String,
    labels: BTreeMap<String, String>,
    value: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct SeriesKey {
    metric: String,
    labels: Vec<(String, String)>,
}

impl SeriesKey {
    fn new(metric: impl Into<String>, labels: BTreeMap<String, String>) -> Self {
        Self {
            metric: metric.into(),
            labels: labels.into_iter().collect(),
        }
    }

    fn labels_map(&self) -> BTreeMap<String, String> {
        self.labels.iter().cloned().collect()
    }

    fn to_id(&self) -> String {
        let mut out = String::with_capacity(64);
        out.push_str(&sanitize_identifier(&self.metric));
        for (key, value) in &self.labels {
            out.push('_');
            out.push_str(&sanitize_identifier(key));
            out.push('_');
            out.push_str(&sanitize_identifier(value));
        }
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        out.push('_');
        out.push_str(&format!("{:x}", hasher.finish()));
        out
    }
}

#[derive(Clone, Copy, Debug)]
enum MetricKind {
    Counter,
    Gauge,
    HistogramBucket,
    HistogramSum,
    HistogramCount,
    Unknown,
}

#[derive(Default)]
struct HistogramBuffer {
    buckets: HashMap<String, f64>,
    sum: Option<f64>,
    count: Option<f64>,
}

#[derive(Default)]
struct HistogramState {
    prev_buckets: HashMap<String, f64>,
    prev_sum: Option<f64>,
    prev_count: Option<f64>,
}

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
        let mut prev_counters = HashMap::<SeriesKey, f64>::new();
        let mut history = HashMap::<SeriesKey, VecDeque<AdminDashboardPoint>>::new();
        let mut histogram_state = HistogramState::default();
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

            let samples = if let Some(prometheus) = &handle {
                parse_prometheus_metrics(&prometheus.render())
            } else {
                Vec::new()
            };

            let mut current_raw = HashMap::<SeriesKey, f64>::new();
            let mut computed = HashMap::<SeriesKey, f64>::new();
            let mut metadata_size_hist = HistogramBuffer::default();

            for sample in samples {
                match classify_metric(&sample.name) {
                    MetricKind::Counter => {
                        let key = SeriesKey::new(sample.name, sample.labels);
                        current_raw.insert(key.clone(), sample.value);
                        let rate = rate_per_second(
                            prev_counters.get(&key).copied(),
                            sample.value,
                            elapsed_secs,
                        );
                        prev_counters.insert(key.clone(), sample.value);
                        computed.insert(key, rate);
                    }
                    MetricKind::Gauge => {
                        let key = SeriesKey::new(sample.name, sample.labels);
                        current_raw.insert(key.clone(), sample.value);
                        computed.insert(key, sample.value);
                    }
                    MetricKind::HistogramBucket => {
                        if let Some(le) = sample.labels.get("le") {
                            metadata_size_hist
                                .buckets
                                .insert(le.to_string(), sample.value);
                        }
                    }
                    MetricKind::HistogramSum => {
                        metadata_size_hist.sum = Some(sample.value);
                    }
                    MetricKind::HistogramCount => {
                        metadata_size_hist.count = Some(sample.value);
                    }
                    MetricKind::Unknown => {}
                }
            }

            if let Some((p50, p95, avg)) =
                histogram_stats(&metadata_size_hist, &mut histogram_state)
            {
                computed.insert(
                    SeriesKey::new(METRIC_METADATA_SIZE_P50, BTreeMap::new()),
                    p50,
                );
                computed.insert(
                    SeriesKey::new(METRIC_METADATA_SIZE_P95, BTreeMap::new()),
                    p95,
                );
                computed.insert(
                    SeriesKey::new(METRIC_METADATA_SIZE_AVG, BTreeMap::new()),
                    avg,
                );
            }

            for (key, value) in computed {
                let points = history
                    .entry(key)
                    .or_insert_with(|| VecDeque::with_capacity(history_points));
                if points.len() >= history_points {
                    points.pop_front();
                }
                points.push_back(AdminDashboardPoint {
                    timestamp: now_ts,
                    value,
                });
            }

            let info_hash_discovered_total =
                sum_metric(&current_raw, METRIC_INFO_HASH_DISCOVERED_TOTAL);
            let metadata_fetch_success_total =
                sum_metric(&current_raw, METRIC_METADATA_SUCCESS_TOTAL);
            let metadata_fetch_fail_total = sum_metric(&current_raw, METRIC_METADATA_FAIL_TOTAL);
            let metadata_queue_size = sum_metric(&current_raw, METRIC_METADATA_QUEUE_SIZE);
            let node_queue_size = sum_metric(&current_raw, METRIC_NODE_QUEUE_SIZE);
            let metadata_worker_pressure = sum_metric(&current_raw, METRIC_WORKER_PRESSURE);
            let udp_packets_received_total =
                sum_metric(&current_raw, METRIC_UDP_PACKETS_RECEIVED_TOTAL);
            let udp_packets_sent_total = sum_metric(&current_raw, METRIC_UDP_PACKETS_SENT_TOTAL);

            let metadata_total = metadata_fetch_success_total + metadata_fetch_fail_total;
            let metadata_success_rate = if metadata_total > 0.0 {
                metadata_fetch_success_total / metadata_total
            } else {
                0.0
            };

            let udp_ok_packets = sum_metric_with_label(
                &current_raw,
                METRIC_UDP_PACKETS_RECEIVED_TOTAL,
                "status",
                "ok",
            );
            let udp_drop_packets = (udp_packets_received_total - udp_ok_packets).max(0.0);
            let udp_receive_drop_rate = if udp_packets_received_total > 0.0 {
                udp_drop_packets / udp_packets_received_total
            } else {
                0.0
            };

            let tabs = build_tabs(&history);
            let points = history.values().map(VecDeque::len).max().unwrap_or(0);
            let crawler_status = state.crawler_status().await.as_str().to_string();

            let snapshot = AdminDashboardSnapshot {
                now: AdminDashboardNow {
                    crawler_status,
                    info_hash_discovered_total,
                    metadata_fetch_success_total,
                    metadata_fetch_fail_total,
                    metadata_success_rate,
                    metadata_queue_size,
                    node_queue_size,
                    metadata_worker_pressure,
                    udp_packets_received_total,
                    udp_packets_sent_total,
                    udp_receive_drop_rate,
                    updated_at: now_ts,
                },
                window: AdminDashboardWindow {
                    sample_interval_secs,
                    points,
                    horizon_secs: sample_interval_secs.saturating_mul(points as u64),
                    prometheus_enabled,
                },
                tabs,
            };

            state.set_admin_dashboard(snapshot).await;
        }
    });
}

fn build_tabs(
    history: &HashMap<SeriesKey, VecDeque<AdminDashboardPoint>>,
) -> Vec<AdminDashboardTab> {
    TAB_DEFINITIONS
        .iter()
        .map(|tab| {
            let charts = tab
                .charts
                .iter()
                .map(|chart| build_chart(*chart, history))
                .collect();

            AdminDashboardTab {
                id: tab.id.to_string(),
                title: tab.title.to_string(),
                charts,
            }
        })
        .collect()
}

fn build_chart(
    chart: ChartDefinition,
    history: &HashMap<SeriesKey, VecDeque<AdminDashboardPoint>>,
) -> AdminDashboardChart {
    let mut matched_keys = history
        .iter()
        .filter_map(|(key, points)| {
            if points.is_empty() || !chart.metrics.contains(&key.metric.as_str()) {
                return None;
            }
            Some(key.clone())
        })
        .collect::<Vec<_>>();

    matched_keys.sort_by(|a, b| {
        a.metric
            .cmp(&b.metric)
            .then_with(|| a.labels.cmp(&b.labels))
    });

    let multi_metric = chart.metrics.len() > 1;
    let series = matched_keys
        .into_iter()
        .filter_map(|key| {
            let points = history.get(&key)?;
            Some(AdminDashboardSeries {
                id: key.to_id(),
                label: series_label(&key, multi_metric),
                labels: key.labels_map(),
                points: points.iter().cloned().collect(),
            })
        })
        .collect();

    AdminDashboardChart {
        id: chart.id.to_string(),
        title: chart.title.to_string(),
        unit: chart.unit.to_string(),
        render: chart.render,
        series,
    }
}

fn series_label(key: &SeriesKey, multi_metric: bool) -> String {
    if key.labels.is_empty() {
        return humanize_metric_name(&key.metric);
    }

    let labels = key
        .labels
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(", ");

    if multi_metric {
        format!("{} ({labels})", humanize_metric_name(&key.metric))
    } else {
        labels
    }
}

fn humanize_metric_name(metric: &str) -> String {
    let trimmed = metric
        .trim_start_matches("dht_")
        .trim_end_matches("_total")
        .replace('_', " ");

    trimmed
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut out = String::new();
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_identifier(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in raw.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }

    if out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "series".to_string()
    } else {
        out
    }
}

fn sum_metric(history: &HashMap<SeriesKey, f64>, metric: &str) -> f64 {
    history
        .iter()
        .filter(|(key, _)| key.metric == metric)
        .map(|(_, value)| *value)
        .sum()
}

fn sum_metric_with_label(
    history: &HashMap<SeriesKey, f64>,
    metric: &str,
    label_key: &str,
    label_value: &str,
) -> f64 {
    history
        .iter()
        .filter(|(key, _)| {
            key.metric == metric
                && key
                    .labels
                    .iter()
                    .any(|(k, v)| k == label_key && v == label_value)
        })
        .map(|(_, value)| *value)
        .sum()
}

fn classify_metric(name: &str) -> MetricKind {
    if GAUGE_METRICS.contains(&name) {
        return MetricKind::Gauge;
    }

    if COUNTER_METRICS.contains(&name) {
        return MetricKind::Counter;
    }

    if let Some(base) = name.strip_suffix("_bucket")
        && base == METRIC_METADATA_SIZE_BYTES
    {
        return MetricKind::HistogramBucket;
    }

    if let Some(base) = name.strip_suffix("_sum")
        && base == METRIC_METADATA_SIZE_BYTES
    {
        return MetricKind::HistogramSum;
    }

    if let Some(base) = name.strip_suffix("_count")
        && base == METRIC_METADATA_SIZE_BYTES
    {
        return MetricKind::HistogramCount;
    }

    MetricKind::Unknown
}

fn parse_prometheus_metrics(text: &str) -> Vec<MetricSample> {
    let mut out = Vec::new();

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

        let (name, labels) = parse_metric_and_labels(metric_with_labels);
        out.push(MetricSample {
            name,
            labels,
            value,
        });
    }

    out
}

fn parse_metric_and_labels(input: &str) -> (String, BTreeMap<String, String>) {
    if let Some(start) = input.find('{')
        && input.ends_with('}')
    {
        let metric = input[..start].to_string();
        let labels_raw = &input[start + 1..input.len() - 1];
        return (metric, parse_labels(labels_raw));
    }

    (input.to_string(), BTreeMap::new())
}

fn parse_labels(input: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let bytes = input.as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() {
        while idx < bytes.len() && (bytes[idx].is_ascii_whitespace() || bytes[idx] == b',') {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }

        let key_start = idx;
        while idx < bytes.len() && bytes[idx] != b'=' {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }

        let key = input[key_start..idx].trim();
        idx += 1;
        if idx >= bytes.len() || bytes[idx] != b'"' {
            break;
        }
        idx += 1;

        let mut value = String::new();
        while idx < bytes.len() {
            let ch = bytes[idx] as char;
            if ch == '\\' {
                idx += 1;
                if idx < bytes.len() {
                    value.push(bytes[idx] as char);
                    idx += 1;
                }
                continue;
            }
            if ch == '"' {
                idx += 1;
                break;
            }

            value.push(ch);
            idx += 1;
        }

        if !key.is_empty() {
            out.insert(key.to_string(), value);
        }

        while idx < bytes.len() && bytes[idx] != b',' {
            idx += 1;
        }
        if idx < bytes.len() && bytes[idx] == b',' {
            idx += 1;
        }
    }

    out
}

fn rate_per_second(previous: Option<f64>, current: f64, elapsed_secs: f64) -> f64 {
    let Some(previous) = previous else {
        return 0.0;
    };

    if elapsed_secs <= 0.0 {
        return 0.0;
    }

    if current >= previous {
        (current - previous) / elapsed_secs
    } else {
        // counter reset
        current / elapsed_secs
    }
}

fn histogram_stats(
    current: &HistogramBuffer,
    state: &mut HistogramState,
) -> Option<(f64, f64, f64)> {
    let current_count = current.count.unwrap_or(0.0);
    let delta_count = counter_delta(state.prev_count, current_count);
    state.prev_count = Some(current_count);

    let current_sum = current.sum.unwrap_or(0.0);
    let delta_sum = counter_delta(state.prev_sum, current_sum);
    state.prev_sum = Some(current_sum);

    let mut deltas = Vec::<(f64, f64)>::new();
    for (le, value) in &current.buckets {
        let prev = state.prev_buckets.get(le).copied();
        let delta = counter_delta(prev, *value);
        state.prev_buckets.insert(le.clone(), *value);

        let bound = if le == "+Inf" {
            f64::INFINITY
        } else {
            match le.parse::<f64>() {
                Ok(bound) => bound,
                Err(_) => continue,
            }
        };

        deltas.push((bound, delta));
    }

    if delta_count <= 0.0 {
        return None;
    }

    deltas.sort_by(|a, b| a.0.total_cmp(&b.0));

    let p50 = histogram_quantile(&deltas, delta_count, 0.50)?;
    let p95 = histogram_quantile(&deltas, delta_count, 0.95)?;
    let avg = if delta_count > 0.0 {
        delta_sum / delta_count
    } else {
        0.0
    };

    Some((p50, p95, avg))
}

fn counter_delta(previous: Option<f64>, current: f64) -> f64 {
    let Some(previous) = previous else {
        return 0.0;
    };

    if current >= previous {
        current - previous
    } else {
        // counter reset
        current
    }
}

fn histogram_quantile(
    cumulative_buckets: &[(f64, f64)],
    total_count: f64,
    quantile: f64,
) -> Option<f64> {
    if cumulative_buckets.is_empty() || total_count <= 0.0 {
        return None;
    }

    let target = total_count * quantile;
    let mut last_finite = 0.0;

    for (bound, cumulative) in cumulative_buckets {
        if bound.is_finite() {
            last_finite = *bound;
        }

        if *cumulative >= target {
            if bound.is_finite() {
                return Some(*bound);
            }
            return Some(last_finite);
        }
    }

    Some(last_finite)
}

#[cfg(test)]
mod tests {
    use super::{
        HistogramBuffer, HistogramState, counter_delta, histogram_stats, parse_prometheus_metrics,
        rate_per_second,
    };

    #[test]
    fn parse_prometheus_metrics_keeps_label_dimensions() {
        let input = r#"
# HELP dht_metadata_fetch_fail_total ...
dht_metadata_fetch_fail_total{reason="timeout"} 5
dht_metadata_fetch_fail_total{reason="parse_error"} 3
"#;

        let parsed = parse_prometheus_metrics(input);
        assert_eq!(parsed.len(), 2);
        assert_ne!(parsed[0].labels, parsed[1].labels);
        assert_eq!(parsed[0].name, "dht_metadata_fetch_fail_total");
    }

    #[test]
    fn counter_rate_handles_first_sample_and_reset() {
        assert_eq!(rate_per_second(None, 10.0, 2.0), 0.0);
        assert_eq!(rate_per_second(Some(10.0), 18.0, 2.0), 4.0);
        assert_eq!(rate_per_second(Some(18.0), 3.0, 2.0), 1.5);
        assert_eq!(counter_delta(None, 10.0), 0.0);
        assert_eq!(counter_delta(Some(10.0), 18.0), 8.0);
        assert_eq!(counter_delta(Some(18.0), 3.0), 3.0);
    }

    #[test]
    fn histogram_stats_computes_p50_p95_avg_from_deltas() {
        let mut state = HistogramState::default();
        state.prev_buckets.insert("10".to_string(), 100.0);
        state.prev_buckets.insert("20".to_string(), 200.0);
        state.prev_buckets.insert("+Inf".to_string(), 300.0);
        state.prev_sum = Some(3000.0);
        state.prev_count = Some(300.0);

        let mut current = HistogramBuffer::default();
        current.buckets.insert("10".to_string(), 110.0);
        current.buckets.insert("20".to_string(), 230.0);
        current.buckets.insert("+Inf".to_string(), 330.0);
        current.sum = Some(3660.0);
        current.count = Some(330.0);

        let (p50, p95, avg) = histogram_stats(&current, &mut state).expect("stats");
        assert_eq!(p50, 20.0);
        assert_eq!(p95, 20.0);
        assert!((avg - 22.0).abs() < f64::EPSILON);
    }

    #[test]
    fn histogram_stats_returns_none_without_new_samples() {
        let mut state = HistogramState::default();
        state.prev_count = Some(10.0);
        state.prev_sum = Some(100.0);

        let mut current = HistogramBuffer::default();
        current.count = Some(10.0);
        current.sum = Some(100.0);

        assert!(histogram_stats(&current, &mut state).is_none());
    }
}
