use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub category: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub window: Option<String>,
    pub category: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TorrentFilesQuery {
    pub flat: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub limit: i64,
    pub offset: i64,
}

impl Pagination {
    pub fn new(page: Option<u32>, page_size: Option<u32>, max_page_size: u32) -> Self {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, max_page_size.max(1));
        let limit = i64::from(page_size);
        let offset = i64::from(page.saturating_sub(1)) * limit;

        Self {
            page,
            page_size,
            limit,
            offset,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FeaturesResponse {
    pub search_enabled: bool,
    pub latest_enabled: bool,
    pub trending_enabled: bool,
    pub file_tree_enabled: bool,
    pub audit_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SiteStatsResponse {
    pub total_torrents_indexed: i64,
    pub total_size_bytes: i64,
    pub last_crawl_at: Option<DateTime<Utc>>,
    pub crawler_status: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryItem {
    pub key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryItem>,
}

#[derive(Debug, Serialize)]
pub struct TorrentListItem {
    pub info_hash: String,
    pub name: String,
    pub magnet_link: String,
    pub category: String,
    pub total_size: i64,
    pub file_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub trend_score: f64,
    pub observations: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub took_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct TorrentFileEntry {
    pub path: String,
    pub size: i64,
    pub depth: i16,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct TorrentDetailResponse {
    pub info_hash: String,
    pub name: String,
    pub magnet_link: String,
    pub category: String,
    pub total_size: i64,
    pub piece_length: i64,
    pub file_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub hot_score: f64,
    pub files_preview: Vec<TorrentFileEntry>,
}

#[derive(Debug, Serialize)]
pub struct TorrentFilesResponse {
    pub files: Vec<TorrentFileEntry>,
}
