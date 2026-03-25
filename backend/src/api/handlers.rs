use crate::{
    api::dto::{
        ApiListResponse, CategoriesResponse, CategoryItem, FeaturesResponse, HealthResponse,
        ListQuery, Pagination, SearchQuery, SiteContentResponse, SiteSettingsUpdateRequest,
        SiteStatsResponse, TorrentDetailResponse, TorrentFileEntry, TorrentFilesQuery,
        TorrentFilesResponse, TorrentListItem, TrendingQuery,
    },
    db::{
        models::{SiteSettingsRow, TorrentDetailRow, TorrentFileRow, TorrentListRow},
        repo::{self, LatestParams, SearchAuditLogInput, SearchParams, TrendingParams},
    },
    domain::category::{Category, all_category_meta},
    error::ApiError,
    search,
    state::AppState,
};
use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
};
use chrono::Utc;
use std::{
    collections::BTreeSet, collections::HashMap, net::IpAddr, net::SocketAddr, time::Instant,
};
use tracing::{info, warn};

const SITE_TITLE_MAX_LEN: usize = 120;
const SITE_DESCRIPTION_MAX_LEN: usize = 300;
const HOME_HERO_MARKDOWN_MAX_LEN: usize = 2_000;

pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        time: Utc::now(),
    })
}

pub async fn admin_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::state::AdminDashboardSnapshot>, ApiError> {
    ensure_admin_authorized(&state, &headers)?;
    Ok(Json(state.admin_dashboard().await))
}

pub async fn features(State(state): State<AppState>) -> Json<FeaturesResponse> {
    Json(FeaturesResponse {
        search_enabled: state.config.features.search_enabled,
        latest_enabled: state.config.features.latest_enabled,
        trending_enabled: state.config.features.trending_enabled,
        file_tree_enabled: state.config.features.file_tree_enabled,
        audit_enabled: state.config.features.audit_enabled,
    })
}

pub async fn site_stats(
    State(state): State<AppState>,
) -> Result<Json<SiteStatsResponse>, ApiError> {
    let stats = repo::fetch_site_stats(&state.pool).await?;
    let crawler_status = state.crawler_status().await.as_str().to_string();

    Ok(Json(SiteStatsResponse {
        total_torrents_indexed: stats.total_torrents_indexed,
        total_size_bytes: stats.total_size_bytes,
        last_crawl_at: stats.last_crawl_at,
        crawler_status,
    }))
}

pub async fn site_content(
    State(state): State<AppState>,
) -> Result<Json<SiteContentResponse>, ApiError> {
    let settings = repo::fetch_site_settings(&state.pool).await?;
    Ok(Json(map_site_content(settings)))
}

pub async fn admin_site_settings_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SiteContentResponse>, ApiError> {
    ensure_admin_authorized(&state, &headers)?;
    let settings = repo::fetch_site_settings(&state.pool).await?;
    Ok(Json(map_site_content(settings)))
}

pub async fn admin_site_settings_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SiteSettingsUpdateRequest>,
) -> Result<Json<SiteContentResponse>, ApiError> {
    ensure_admin_authorized(&state, &headers)?;

    let site_title =
        normalize_site_setting_field(&payload.site_title, "site_title", SITE_TITLE_MAX_LEN)?;
    let site_description = normalize_site_setting_field(
        &payload.site_description,
        "site_description",
        SITE_DESCRIPTION_MAX_LEN,
    )?;
    let home_hero_markdown = normalize_site_setting_field(
        &payload.home_hero_markdown,
        "home_hero_markdown",
        HOME_HERO_MARKDOWN_MAX_LEN,
    )?;

    let updated = repo::upsert_site_settings(
        &state.pool,
        &site_title,
        &site_description,
        &home_hero_markdown,
    )
    .await?;

    Ok(Json(map_site_content(updated)))
}

pub async fn categories(
    State(state): State<AppState>,
) -> Result<Json<CategoriesResponse>, ApiError> {
    let rows = repo::fetch_category_counts(&state.pool).await?;
    let mut by_code = HashMap::<i16, i64>::new();

    for row in rows {
        by_code.insert(row.category, row.count);
    }

    let total: i64 = by_code.values().sum();
    let mut categories = Vec::new();

    for meta in all_category_meta() {
        let count = if meta.code == 0 {
            total
        } else {
            *by_code.get(&meta.code).unwrap_or(&0)
        };

        categories.push(CategoryItem {
            key: meta.key.to_string(),
            label: meta.label.to_string(),
            count,
        });
    }

    Ok(Json(CategoriesResponse { categories }))
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
    connect_info: ConnectInfo<SocketAddr>,
) -> Result<Json<ApiListResponse<TorrentListItem>>, ApiError> {
    ensure_feature(state.config.features.search_enabled, "search")?;

    let start = Instant::now();
    let pagination = Pagination::new(
        query.page,
        query.page_size,
        state.config.search_max_page_size,
    );

    let trimmed_query = query.q.trim().to_string();
    let mut status_code = 200;
    let mut result_count = 0i64;

    let parsed_sort = match repo::SearchSort::parse(query.sort.as_deref()) {
        Ok(value) => value,
        Err(msg) => {
            status_code = 400;
            write_search_audit(
                &state,
                &headers,
                Some(&connect_info),
                &trimmed_query,
                None,
                repo::SearchSort::Relevance,
                pagination,
                result_count,
                start.elapsed().as_millis() as i64,
                status_code,
            )
            .await;
            return Err(ApiError::bad_request(msg));
        }
    };
    let parsed_category = match parse_category(query.category.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            status_code = 400;
            write_search_audit(
                &state,
                &headers,
                Some(&connect_info),
                &trimmed_query,
                None,
                parsed_sort,
                pagination,
                result_count,
                start.elapsed().as_millis() as i64,
                status_code,
            )
            .await;
            return Err(err);
        }
    };

    if trimmed_query.is_empty() {
        status_code = 400;
        write_search_audit(
            &state,
            &headers,
            Some(&connect_info),
            &trimmed_query,
            parsed_category,
            parsed_sort,
            pagination,
            result_count,
            start.elapsed().as_millis() as i64,
            status_code,
        )
        .await;
        return Err(ApiError::bad_request("query parameter 'q' cannot be empty"));
    }

    let params = SearchParams {
        q: trimmed_query.clone(),
        category: parsed_category,
        sort: parsed_sort,
        limit: pagination.limit,
        offset: pagination.offset,
    };

    let (rows, total) = match search::query::search_torrents(&state, &params).await {
        Ok(Some(value)) => value,
        Ok(None) => match repo::search_torrents(&state.pool, &params).await {
            Ok(value) => value,
            Err(err) => {
                status_code = 500;
                write_search_audit(
                    &state,
                    &headers,
                    Some(&connect_info),
                    &trimmed_query,
                    parsed_category,
                    parsed_sort,
                    pagination,
                    result_count,
                    start.elapsed().as_millis() as i64,
                    status_code,
                )
                .await;
                return Err(ApiError::from(err));
            }
        },
        Err(err) => {
            warn!(
                error = ?err,
                sort = parsed_sort.as_str(),
                "meili search failed, falling back to postgres"
            );

            match repo::search_torrents(&state.pool, &params).await {
                Ok(value) => value,
                Err(err) => {
                    status_code = 500;
                    write_search_audit(
                        &state,
                        &headers,
                        Some(&connect_info),
                        &trimmed_query,
                        parsed_category,
                        parsed_sort,
                        pagination,
                        result_count,
                        start.elapsed().as_millis() as i64,
                        status_code,
                    )
                    .await;
                    return Err(ApiError::from(err));
                }
            }
        }
    };

    result_count = rows.len() as i64;

    write_search_audit(
        &state,
        &headers,
        Some(&connect_info),
        &trimmed_query,
        parsed_category,
        parsed_sort,
        pagination,
        result_count,
        start.elapsed().as_millis() as i64,
        status_code,
    )
    .await;

    Ok(Json(ApiListResponse {
        items: rows.into_iter().map(map_torrent_item).collect(),
        total,
        page: pagination.page,
        page_size: pagination.page_size,
        took_ms: start.elapsed().as_millis() as i64,
    }))
}

pub async fn latest(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiListResponse<TorrentListItem>>, ApiError> {
    ensure_feature(state.config.features.latest_enabled, "latest")?;

    let start = Instant::now();
    let pagination = Pagination::new(
        query.page,
        query.page_size,
        state.config.search_max_page_size,
    );

    let params = LatestParams {
        category: parse_category(query.category.as_deref())?,
        limit: pagination.limit,
        offset: pagination.offset,
    };

    let (rows, total) = repo::latest_torrents(&state.pool, &params).await?;

    Ok(Json(ApiListResponse {
        items: rows.into_iter().map(map_torrent_item).collect(),
        total,
        page: pagination.page,
        page_size: pagination.page_size,
        took_ms: start.elapsed().as_millis() as i64,
    }))
}

pub async fn trending(
    State(state): State<AppState>,
    Query(query): Query<TrendingQuery>,
) -> Result<Json<ApiListResponse<TorrentListItem>>, ApiError> {
    ensure_feature(state.config.features.trending_enabled, "trending")?;

    let start = Instant::now();
    let pagination = Pagination::new(
        query.page,
        query.page_size,
        state.config.search_max_page_size,
    );

    let window = repo::TrendingWindow::parse(query.window.as_deref())
        .map_err(|msg| ApiError::bad_request(msg.to_string()))?;

    let params = TrendingParams {
        category: parse_category(query.category.as_deref())?,
        window,
        limit: pagination.limit,
        offset: pagination.offset,
    };

    let (rows, total) = repo::trending_torrents(&state.pool, &params).await?;

    Ok(Json(ApiListResponse {
        items: rows.into_iter().map(map_torrent_item).collect(),
        total,
        page: pagination.page,
        page_size: pagination.page_size,
        took_ms: start.elapsed().as_millis() as i64,
    }))
}

pub async fn torrent_detail(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> Result<Json<TorrentDetailResponse>, ApiError> {
    let api_start = Instant::now();
    let info_hash = normalize_info_hash(&info_hash)?;

    let detail_db_start = Instant::now();
    let row = repo::fetch_torrent_detail(&state.pool, &info_hash)
        .await?
        .ok_or_else(|| ApiError::not_found("torrent not found"))?;
    let detail_db_ms = detail_db_start.elapsed().as_millis() as i64;

    let preview_db_start = Instant::now();
    let files_preview = if state.config.features.file_tree_enabled {
        Vec::new()
    } else {
        let preview_rows = repo::fetch_torrent_files_preview(&state.pool, &info_hash, 20).await?;
        preview_rows
            .into_iter()
            .map(|entry| TorrentFileEntry {
                path: entry.path,
                size: entry.size,
                depth: entry.depth,
                is_dir: false,
            })
            .collect()
    };
    let preview_db_ms = preview_db_start.elapsed().as_millis() as i64;

    info!(
        info_hash = %info_hash,
        detail_db_ms,
        preview_db_ms,
        file_tree_enabled = state.config.features.file_tree_enabled,
        total_ms = api_start.elapsed().as_millis() as i64,
        "torrent_detail timing"
    );

    Ok(Json(map_torrent_detail(row, files_preview)))
}

pub async fn torrent_files(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
    Query(query): Query<TorrentFilesQuery>,
) -> Result<Json<TorrentFilesResponse>, ApiError> {
    ensure_feature(state.config.features.file_tree_enabled, "file_tree")?;

    let api_start = Instant::now();
    let info_hash = normalize_info_hash(&info_hash)?;

    let exists_db_start = Instant::now();
    let exists = repo::torrent_exists(&state.pool, &info_hash).await?;
    let exists_db_ms = exists_db_start.elapsed().as_millis() as i64;
    if !exists {
        return Err(ApiError::not_found("torrent not found"));
    }

    let files_db_start = Instant::now();
    let files = repo::fetch_torrent_files(&state.pool, &info_hash).await?;
    let files_db_ms = files_db_start.elapsed().as_millis() as i64;
    let flat = query.flat.unwrap_or(false);

    let tree_build_start = Instant::now();
    let result_files = if flat {
        files
            .into_iter()
            .map(|entry| TorrentFileEntry {
                path: entry.path,
                size: entry.size,
                depth: entry.depth,
                is_dir: false,
            })
            .collect()
    } else {
        build_file_tree(files)
    };
    let tree_build_ms = tree_build_start.elapsed().as_millis() as i64;

    info!(
        info_hash = %info_hash,
        flat,
        exists_db_ms,
        files_db_ms,
        tree_build_ms,
        file_count = result_files.len(),
        total_ms = api_start.elapsed().as_millis() as i64,
        "torrent_files timing"
    );

    Ok(Json(TorrentFilesResponse {
        files: result_files,
    }))
}

fn ensure_feature(enabled: bool, key: &'static str) -> Result<(), ApiError> {
    if enabled {
        Ok(())
    } else {
        Err(ApiError::feature_disabled(key))
    }
}

fn ensure_admin_authorized(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(expected_password) = state.config.admin.dashboard_password.as_deref() else {
        return Err(ApiError::feature_disabled("admin_dashboard"));
    };

    let provided = header_value(headers, "x-admin-password")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::unauthorized("missing x-admin-password header"))?;

    if provided != expected_password {
        return Err(ApiError::unauthorized("invalid admin password"));
    }

    Ok(())
}

fn parse_category(value: Option<&str>) -> Result<Option<i16>, ApiError> {
    Category::parse_filter(value)
        .map_err(ApiError::bad_request)
        .map(|category| category.map(Category::code))
}

fn normalize_site_setting_field(
    value: &str,
    field_name: &str,
    max_len: usize,
) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request(format!(
            "field '{field_name}' cannot be empty"
        )));
    }

    if trimmed.chars().count() > max_len {
        return Err(ApiError::bad_request(format!(
            "field '{field_name}' exceeds max length {max_len}"
        )));
    }

    Ok(trimmed.to_string())
}

fn normalize_info_hash(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() != 40 || !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ApiError::bad_request("invalid info_hash format"));
    }

    Ok(normalized)
}

fn map_torrent_item(row: TorrentListRow) -> TorrentListItem {
    TorrentListItem {
        info_hash: row.info_hash,
        name: row.name,
        magnet_link: row.magnet_link,
        category: Category::from_code(row.category).key().to_string(),
        total_size: row.total_size,
        file_count: row.file_count,
        first_seen_at: row.first_seen_at,
        last_seen_at: row.last_seen_at,
        trend_score: row.trend_score,
        observations: row.observations,
    }
}

fn map_site_content(row: SiteSettingsRow) -> SiteContentResponse {
    SiteContentResponse {
        site_title: row.site_title,
        site_description: row.site_description,
        home_hero_markdown: row.home_hero_markdown,
        updated_at: row.updated_at,
    }
}

fn map_torrent_detail(
    row: TorrentDetailRow,
    files_preview: Vec<TorrentFileEntry>,
) -> TorrentDetailResponse {
    TorrentDetailResponse {
        info_hash: row.info_hash,
        name: row.name,
        magnet_link: row.magnet_link,
        category: Category::from_code(row.category).key().to_string(),
        total_size: row.total_size,
        piece_length: row.piece_length,
        file_count: row.file_count,
        first_seen_at: row.first_seen_at,
        last_seen_at: row.last_seen_at,
        hot_score: row.hot_score,
        files_preview,
    }
}

fn build_file_tree(files: Vec<TorrentFileRow>) -> Vec<TorrentFileEntry> {
    let mut dirs = BTreeSet::<String>::new();
    let mut out = Vec::<TorrentFileEntry>::new();

    for file in files {
        let parts: Vec<&str> = file
            .path
            .split('/')
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() > 1 {
            let mut current = String::new();
            for part in parts.iter().take(parts.len() - 1) {
                if !current.is_empty() {
                    current.push('/');
                }
                current.push_str(part);
                dirs.insert(current.clone());
            }
        }

        out.push(TorrentFileEntry {
            path: file.path,
            size: file.size,
            depth: file.depth,
            is_dir: false,
        });
    }

    let mut dir_entries = dirs
        .into_iter()
        .map(|path| TorrentFileEntry {
            depth: path
                .split('/')
                .filter(|part| !part.is_empty())
                .count()
                .saturating_sub(1) as i16,
            path,
            size: 0,
            is_dir: true,
        })
        .collect::<Vec<_>>();

    dir_entries.append(&mut out);
    dir_entries.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| b.is_dir.cmp(&a.is_dir)));

    dir_entries
}

async fn write_search_audit(
    state: &AppState,
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
    query_text: &str,
    category: Option<i16>,
    sort: repo::SearchSort,
    pagination: Pagination,
    result_count: i64,
    latency_ms: i64,
    status_code: i32,
) {
    if !state.config.features.audit_enabled {
        return;
    }

    let entry = SearchAuditLogInput {
        query_text,
        category,
        sort: sort.as_str(),
        page: i64::from(pagination.page),
        page_size: i64::from(pagination.page_size),
        client_ip: extract_client_ip(headers, connect_info),
        user_agent: header_value(headers, "user-agent"),
        referer: header_value(headers, "referer"),
        result_count,
        latency_ms,
        status_code,
    };

    if let Err(err) = repo::insert_search_audit_log(&state.pool, &entry).await {
        warn!(error = ?err, "failed to insert search audit log");
    }
}

fn header_value<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    headers.get(key).and_then(|value| value.to_str().ok())
}

fn extract_client_ip(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> Option<String> {
    if let Some(forwarded) = header_value(headers, "x-forwarded-for") {
        if let Some(first) = forwarded.split(',').next()
            && let Ok(ip) = first.trim().parse::<IpAddr>()
        {
            return Some(ip.to_string());
        }
    }

    if let Some(real_ip) = header_value(headers, "x-real-ip")
        && let Ok(ip) = real_ip.trim().parse::<IpAddr>()
    {
        return Some(ip.to_string());
    }

    connect_info.map(|info| info.0.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::build_file_tree;
    use crate::db::models::TorrentFileRow;

    #[test]
    fn build_tree_contains_dirs_and_files() {
        let files = vec![
            TorrentFileRow {
                path: "movie/part1.mkv".to_string(),
                size: 10,
                depth: 1,
            },
            TorrentFileRow {
                path: "movie/subs/en.srt".to_string(),
                size: 2,
                depth: 2,
            },
        ];

        let tree = build_file_tree(files);
        assert!(
            tree.iter()
                .any(|entry| entry.is_dir && entry.path == "movie")
        );
        assert!(
            tree.iter()
                .any(|entry| entry.is_dir && entry.path == "movie/subs")
        );
        assert!(
            tree.iter()
                .any(|entry| !entry.is_dir && entry.path == "movie/part1.mkv")
        );
    }
}
