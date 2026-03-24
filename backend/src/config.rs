use anyhow::{Context, Result, bail};
use std::env;

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub search_enabled: bool,
    pub latest_enabled: bool,
    pub trending_enabled: bool,
    pub file_tree_enabled: bool,
    pub audit_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub enabled: bool,
    pub port: u16,
    pub netmode: String,
    pub metadata_timeout: u64,
    pub max_metadata_queue_size: usize,
    pub max_metadata_worker_count: usize,
    pub node_queue_capacity: usize,
    pub hash_queue_capacity: usize,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub search_max_page_size: u32,
    pub audit_log_retention_days: i64,
    pub features: FeatureFlags,
    pub crawler: CrawlerConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let database_url = required_env("DATABASE_URL")?;

        Ok(Self {
            server_host: env_or("SERVER_HOST", "0.0.0.0"),
            server_port: parse_env("SERVER_PORT", 8080u16)?,
            database_url,
            database_max_connections: parse_env("DATABASE_MAX_CONNECTIONS", 20u32)?,
            search_max_page_size: parse_env("SEARCH_MAX_PAGE_SIZE", 100u32)?,
            audit_log_retention_days: parse_env("AUDIT_LOG_RETENTION_DAYS", 180i64)?,
            features: FeatureFlags {
                search_enabled: parse_env_bool("FEATURE_SEARCH_ENABLED", true)?,
                latest_enabled: parse_env_bool("FEATURE_LATEST_ENABLED", true)?,
                trending_enabled: parse_env_bool("FEATURE_TRENDING_ENABLED", true)?,
                file_tree_enabled: parse_env_bool("FEATURE_FILE_TREE_ENABLED", true)?,
                audit_enabled: parse_env_bool("FEATURE_SEARCH_AUDIT_ENABLED", true)?,
            },
            crawler: CrawlerConfig {
                enabled: parse_env_bool("CRAWLER_ENABLED", true)?,
                port: parse_env("CRAWLER_PORT", 6881u16)?,
                netmode: env_or("CRAWLER_NETMODE", "ipv4"),
                metadata_timeout: parse_env("CRAWLER_METADATA_TIMEOUT", 3u64)?,
                max_metadata_queue_size: parse_env(
                    "CRAWLER_MAX_METADATA_QUEUE_SIZE",
                    100_000usize,
                )?,
                max_metadata_worker_count: parse_env(
                    "CRAWLER_MAX_METADATA_WORKER_COUNT",
                    1_000usize,
                )?,
                node_queue_capacity: parse_env("CRAWLER_NODE_QUEUE_CAPACITY", 100_000usize)?,
                hash_queue_capacity: parse_env("CRAWLER_HASH_QUEUE_CAPACITY", 10_000usize)?,
            },
        })
    }
}

fn required_env(key: &str) -> Result<String> {
    let value = env::var(key).with_context(|| format!("missing required env var: {key}"))?;
    if value.trim().is_empty() {
        bail!("required env var {key} is empty");
    }
    Ok(value)
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn parse_env<T>(key: &str, default: T) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(raw) => raw
            .parse::<T>()
            .map_err(|err| anyhow::anyhow!("invalid {key} value '{raw}': {err}")),
        Err(_) => Ok(default),
    }
}

fn parse_env_bool(key: &str, default: bool) -> Result<bool> {
    match env::var(key) {
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(anyhow::anyhow!(
                "invalid {key} value '{raw}', expected true/false"
            )),
        },
        Err(_) => Ok(default),
    }
}
