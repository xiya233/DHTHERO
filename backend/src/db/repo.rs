use crate::db::models::{
    CategoryCountRow, SiteStatsRow, TorrentDetailRow, TorrentFileRow, TorrentListRow,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
#[derive(Debug, Clone, Copy)]
pub enum SearchSort {
    Relevance,
    Latest,
    SizeAsc,
    SizeDesc,
    Hot,
}

impl SearchSort {
    pub fn parse(raw: Option<&str>) -> Result<Self, String> {
        match raw.map(str::trim).map(str::to_ascii_lowercase) {
            None => Ok(Self::Relevance),
            Some(value) if value.is_empty() => Ok(Self::Relevance),
            Some(value) => match value.as_str() {
                "relevance" => Ok(Self::Relevance),
                "latest" => Ok(Self::Latest),
                "size_asc" => Ok(Self::SizeAsc),
                "size_desc" => Ok(Self::SizeDesc),
                "hot" => Ok(Self::Hot),
                _ => Err(format!("unsupported sort: {value}")),
            },
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Relevance => "relevance",
            Self::Latest => "latest",
            Self::SizeAsc => "size_asc",
            Self::SizeDesc => "size_desc",
            Self::Hot => "hot",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrendingWindow {
    H24,
    H72,
    D7,
}

impl TrendingWindow {
    pub fn parse(raw: Option<&str>) -> Result<Self, String> {
        match raw.map(str::trim).map(str::to_ascii_lowercase) {
            None => Ok(Self::H24),
            Some(value) if value.is_empty() => Ok(Self::H24),
            Some(value) => match value.as_str() {
                "24h" => Ok(Self::H24),
                "72h" => Ok(Self::H72),
                "7d" => Ok(Self::D7),
                _ => Err(format!("unsupported window: {value}")),
            },
        }
    }

    pub const fn interval_literal(self) -> &'static str {
        match self {
            Self::H24 => "'24 hours'",
            Self::H72 => "'72 hours'",
            Self::D7 => "'7 days'",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub q: String,
    pub category: Option<i16>,
    pub sort: SearchSort,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct LatestParams {
    pub category: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct TrendingParams {
    pub category: Option<i16>,
    pub window: TrendingWindow,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug)]
pub struct SearchAuditLogInput<'a> {
    pub query_text: &'a str,
    pub category: Option<i16>,
    pub sort: &'a str,
    pub page: i64,
    pub page_size: i64,
    pub client_ip: Option<String>,
    pub user_agent: Option<&'a str>,
    pub referer: Option<&'a str>,
    pub result_count: i64,
    pub latency_ms: i64,
    pub status_code: i32,
}

pub async fn fetch_site_stats(pool: &PgPool) -> Result<SiteStatsRow, sqlx::Error> {
    sqlx::query_as::<_, SiteStatsRow>(
        r#"
        SELECT
            COUNT(*)::bigint AS total_torrents_indexed,
            COALESCE(SUM(total_size), 0)::bigint AS total_size_bytes,
            MAX(last_seen_at) AS last_crawl_at
        FROM torrents
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn fetch_category_counts(pool: &PgPool) -> Result<Vec<CategoryCountRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryCountRow>(
        r#"
        SELECT category, COUNT(*)::bigint AS count
        FROM torrents
        GROUP BY category
        ORDER BY category ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn search_torrents(
    pool: &PgPool,
    params: &SearchParams,
) -> Result<(Vec<TorrentListRow>, i64), sqlx::Error> {
    let mut count_qb = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*)::bigint AS total FROM torrents t WHERE t.name_tsv @@ websearch_to_tsquery('simple', ",
    );
    count_qb.push_bind(&params.q);
    count_qb.push(")");

    if let Some(category) = params.category {
        count_qb.push(" AND t.category = ");
        count_qb.push_bind(category);
    }

    let (total,): (i64,) = count_qb.build_query_as().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            t.info_hash,
            t.name,
            t.magnet_link,
            t.category,
            t.total_size,
            t.file_count,
            t.first_seen_at,
            t.last_seen_at,
            COALESCE(h.trend_score, 0)::double precision AS trend_score,
            COALESCE(h.observations, 0)::bigint AS observations
        FROM torrents t
        LEFT JOIN LATERAL (
            SELECT
                COALESCE(SUM(hs.trend_score), 0)::double precision AS trend_score,
                COALESCE(SUM(hs.observation_count), 0)::bigint AS observations
            FROM torrent_hot_stats_hourly hs
            WHERE hs.info_hash = t.info_hash
              AND hs.bucket_hour >= NOW() - INTERVAL '24 hours'
        ) h ON TRUE
        WHERE t.name_tsv @@ websearch_to_tsquery('simple',
        "#,
    );
    qb.push_bind(&params.q);
    qb.push(")");

    if let Some(category) = params.category {
        qb.push(" AND t.category = ");
        qb.push_bind(category);
    }

    qb.push(" ORDER BY ");
    match params.sort {
        SearchSort::Relevance => {
            qb.push("ts_rank_cd(t.name_tsv, websearch_to_tsquery('simple', ");
            qb.push_bind(&params.q);
            qb.push(")) DESC, t.last_seen_at DESC");
        }
        SearchSort::Latest => {
            qb.push("t.last_seen_at DESC");
        }
        SearchSort::SizeAsc => {
            qb.push("t.total_size ASC, t.last_seen_at DESC");
        }
        SearchSort::SizeDesc => {
            qb.push("t.total_size DESC, t.last_seen_at DESC");
        }
        SearchSort::Hot => {
            qb.push("h.trend_score DESC, h.observations DESC, t.last_seen_at DESC");
        }
    }

    qb.push(" LIMIT ");
    qb.push_bind(params.limit);
    qb.push(" OFFSET ");
    qb.push_bind(params.offset);

    let items = qb
        .build_query_as::<TorrentListRow>()
        .fetch_all(pool)
        .await?;

    Ok((items, total))
}

pub async fn latest_torrents(
    pool: &PgPool,
    params: &LatestParams,
) -> Result<(Vec<TorrentListRow>, i64), sqlx::Error> {
    let mut count_qb =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*)::bigint AS total FROM torrents t WHERE 1=1");

    if let Some(category) = params.category {
        count_qb.push(" AND t.category = ");
        count_qb.push_bind(category);
    }

    let (total,): (i64,) = count_qb.build_query_as().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            t.info_hash,
            t.name,
            t.magnet_link,
            t.category,
            t.total_size,
            t.file_count,
            t.first_seen_at,
            t.last_seen_at,
            COALESCE(h.trend_score, 0)::double precision AS trend_score,
            COALESCE(h.observations, 0)::bigint AS observations
        FROM torrents t
        LEFT JOIN LATERAL (
            SELECT
                COALESCE(SUM(hs.trend_score), 0)::double precision AS trend_score,
                COALESCE(SUM(hs.observation_count), 0)::bigint AS observations
            FROM torrent_hot_stats_hourly hs
            WHERE hs.info_hash = t.info_hash
              AND hs.bucket_hour >= NOW() - INTERVAL '24 hours'
        ) h ON TRUE
        WHERE 1=1
        "#,
    );

    if let Some(category) = params.category {
        qb.push(" AND t.category = ");
        qb.push_bind(category);
    }

    qb.push(" ORDER BY t.last_seen_at DESC LIMIT ");
    qb.push_bind(params.limit);
    qb.push(" OFFSET ");
    qb.push_bind(params.offset);

    let items = qb
        .build_query_as::<TorrentListRow>()
        .fetch_all(pool)
        .await?;
    Ok((items, total))
}

pub async fn trending_torrents(
    pool: &PgPool,
    params: &TrendingParams,
) -> Result<(Vec<TorrentListRow>, i64), sqlx::Error> {
    let mut count_qb = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(DISTINCT hs.info_hash)::bigint AS total FROM torrent_hot_stats_hourly hs JOIN torrents t ON t.info_hash = hs.info_hash WHERE hs.bucket_hour >= NOW() - INTERVAL ",
    );
    count_qb.push(params.window.interval_literal());

    if let Some(category) = params.category {
        count_qb.push(" AND t.category = ");
        count_qb.push_bind(category);
    }

    let (total,): (i64,) = count_qb.build_query_as().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            t.info_hash,
            t.name,
            t.magnet_link,
            t.category,
            t.total_size,
            t.file_count,
            t.first_seen_at,
            t.last_seen_at,
            COALESCE(SUM(hs.trend_score), 0)::double precision AS trend_score,
            COALESCE(SUM(hs.observation_count), 0)::bigint AS observations
        FROM torrent_hot_stats_hourly hs
        JOIN torrents t ON t.info_hash = hs.info_hash
        WHERE hs.bucket_hour >= NOW() - INTERVAL
        "#,
    );
    qb.push(params.window.interval_literal());

    if let Some(category) = params.category {
        qb.push(" AND t.category = ");
        qb.push_bind(category);
    }

    qb.push(
        " GROUP BY t.info_hash, t.name, t.magnet_link, t.category, t.total_size, t.file_count, t.first_seen_at, t.last_seen_at",
    );
    qb.push(" ORDER BY trend_score DESC, observations DESC, t.last_seen_at DESC LIMIT ");
    qb.push_bind(params.limit);
    qb.push(" OFFSET ");
    qb.push_bind(params.offset);

    let items = qb
        .build_query_as::<TorrentListRow>()
        .fetch_all(pool)
        .await?;
    Ok((items, total))
}

pub async fn fetch_torrent_detail(
    pool: &PgPool,
    info_hash: &str,
) -> Result<Option<TorrentDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, TorrentDetailRow>(
        r#"
        SELECT
            t.info_hash,
            t.magnet_link,
            t.name,
            t.category,
            t.total_size,
            t.piece_length,
            t.file_count,
            t.first_seen_at,
            t.last_seen_at,
            COALESCE(h.trend_score, 0)::double precision AS hot_score
        FROM torrents t
        LEFT JOIN LATERAL (
            SELECT COALESCE(SUM(hs.trend_score), 0)::double precision AS trend_score
            FROM torrent_hot_stats_hourly hs
            WHERE hs.info_hash = t.info_hash
              AND hs.bucket_hour >= NOW() - INTERVAL '24 hours'
        ) h ON TRUE
        WHERE t.info_hash = $1
        "#,
    )
    .bind(info_hash)
    .fetch_optional(pool)
    .await
}

pub async fn fetch_torrent_files(
    pool: &PgPool,
    info_hash: &str,
) -> Result<Vec<TorrentFileRow>, sqlx::Error> {
    sqlx::query_as::<_, TorrentFileRow>(
        r#"
        SELECT file_index, path, size, depth
        FROM torrent_files
        WHERE info_hash = $1
        ORDER BY file_index ASC
        "#,
    )
    .bind(info_hash)
    .fetch_all(pool)
    .await
}

pub async fn fetch_torrent_files_preview(
    pool: &PgPool,
    info_hash: &str,
    limit: i64,
) -> Result<Vec<TorrentFileRow>, sqlx::Error> {
    sqlx::query_as::<_, TorrentFileRow>(
        r#"
        SELECT file_index, path, size, depth
        FROM torrent_files
        WHERE info_hash = $1
        ORDER BY file_index ASC
        LIMIT $2
        "#,
    )
    .bind(info_hash)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn insert_search_audit_log(
    pool: &PgPool,
    input: &SearchAuditLogInput<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO search_audit_logs (
            query_text,
            category,
            sort,
            page,
            page_size,
            client_ip,
            user_agent,
            referer,
            result_count,
            latency_ms,
            status_code
        ) VALUES ($1,$2,$3,$4,$5,$6::inet,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(input.query_text)
    .bind(input.category)
    .bind(input.sort)
    .bind(as_i32(input.page))
    .bind(as_i32(input.page_size))
    .bind(input.client_ip.as_deref())
    .bind(input.user_agent)
    .bind(input.referer)
    .bind(as_i32(input.result_count))
    .bind(as_i32(input.latency_ms))
    .bind(input.status_code)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn purge_old_audit_logs(pool: &PgPool, retention_days: i64) -> Result<u64, sqlx::Error> {
    let retention_days = retention_days.max(1);
    let result = sqlx::query(
        "DELETE FROM search_audit_logs WHERE created_at < NOW() - make_interval(days => $1)",
    )
    .bind(as_i32(retention_days))
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

fn as_i32(value: i64) -> i32 {
    value.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

#[cfg(test)]
mod tests {
    use super::{SearchSort, TrendingWindow};

    #[test]
    fn parse_sort_default_relevance() {
        assert!(matches!(SearchSort::parse(None), Ok(SearchSort::Relevance)));
        assert!(matches!(
            SearchSort::parse(Some("")),
            Ok(SearchSort::Relevance)
        ));
    }

    #[test]
    fn parse_window_variants() {
        assert!(matches!(
            TrendingWindow::parse(Some("24h")),
            Ok(TrendingWindow::H24)
        ));
        assert!(matches!(
            TrendingWindow::parse(Some("72h")),
            Ok(TrendingWindow::H72)
        ));
        assert!(matches!(
            TrendingWindow::parse(Some("7d")),
            Ok(TrendingWindow::D7)
        ));
    }
}
