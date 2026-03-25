use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct SiteStatsRow {
    pub total_torrents_indexed: i64,
    pub total_size_bytes: i64,
    pub last_crawl_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SiteSettingsRow {
    pub site_title: String,
    pub site_description: String,
    pub home_hero_markdown: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TorrentListRow {
    pub info_hash: String,
    pub name: String,
    pub magnet_link: String,
    pub category: i16,
    pub total_size: i64,
    pub file_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub trend_score: f64,
    pub observations: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TorrentDetailRow {
    pub info_hash: String,
    pub magnet_link: String,
    pub name: String,
    pub category: i16,
    pub total_size: i64,
    pub piece_length: i64,
    pub file_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub hot_score: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TorrentFileRow {
    pub path: String,
    pub size: i64,
    pub depth: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct CategoryCountRow {
    pub category: i16,
    pub count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct MeiliDocRow {
    pub info_hash: String,
    pub name: String,
    pub category: i16,
    pub total_size: i64,
    pub file_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}
