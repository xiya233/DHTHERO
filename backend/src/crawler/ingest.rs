use crate::domain::category::Category;
use anyhow::Result;
use chrono::{DateTime, Utc};
use dht_crawler::TorrentInfo;
use sqlx::PgPool;
use std::net::SocketAddr;

pub async fn ingest_torrent(pool: &PgPool, torrent: TorrentInfo) -> Result<()> {
    if !is_valid_info_hash(&torrent.info_hash) {
        return Ok(());
    }

    let info_hash = torrent.info_hash.to_ascii_lowercase();
    let observed_at =
        DateTime::<Utc>::from_timestamp(torrent.timestamp as i64, 0).unwrap_or_else(Utc::now);

    let category = Category::classify(&torrent.name, &torrent.files);
    let file_count = torrent.files.len() as i32;

    let peer = torrent
        .peers
        .first()
        .and_then(|value| value.parse::<SocketAddr>().ok());
    let peer_ip: Option<String> = peer.map(|value| value.ip().to_string());
    let peer_port: Option<i32> = peer.map(|value| i32::from(value.port()));

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO torrents (
            info_hash,
            magnet_link,
            name,
            category,
            total_size,
            piece_length,
            file_count,
            first_seen_at,
            last_seen_at,
            last_peer_ip,
            last_peer_port,
            fetch_success_count
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$8,$9::inet,$10,1
        )
        ON CONFLICT (info_hash) DO UPDATE
        SET
            magnet_link = EXCLUDED.magnet_link,
            name = EXCLUDED.name,
            category = EXCLUDED.category,
            total_size = EXCLUDED.total_size,
            piece_length = EXCLUDED.piece_length,
            file_count = EXCLUDED.file_count,
            first_seen_at = LEAST(torrents.first_seen_at, EXCLUDED.first_seen_at),
            last_seen_at = GREATEST(torrents.last_seen_at, EXCLUDED.last_seen_at),
            last_peer_ip = COALESCE(EXCLUDED.last_peer_ip, torrents.last_peer_ip),
            last_peer_port = COALESCE(EXCLUDED.last_peer_port, torrents.last_peer_port),
            fetch_success_count = torrents.fetch_success_count + 1,
            updated_at = NOW()
        "#,
    )
    .bind(&info_hash)
    .bind(&torrent.magnet_link)
    .bind(&torrent.name)
    .bind(category.code())
    .bind(i64::try_from(torrent.total_size).unwrap_or(i64::MAX))
    .bind(i64::try_from(torrent.piece_length).unwrap_or(i64::MAX))
    .bind(file_count)
    .bind(observed_at)
    .bind(peer_ip)
    .bind(peer_port)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM torrent_files WHERE info_hash = $1")
        .bind(&info_hash)
        .execute(&mut *tx)
        .await?;

    for (index, file) in torrent.files.iter().enumerate() {
        let file_size = i64::try_from(file.size).unwrap_or(i64::MAX);
        let depth = path_depth(&file.path);
        let ext = file_ext(&file.path);

        sqlx::query(
            r#"
            INSERT INTO torrent_files (info_hash, file_index, path, size, depth, ext)
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(&info_hash)
        .bind(index as i32)
        .bind(&file.path)
        .bind(file_size)
        .bind(depth)
        .bind(ext)
        .execute(&mut *tx)
        .await?;
    }

    if let Some(peer_addr) = peer {
        sqlx::query(
            r#"
            INSERT INTO dht_observations (info_hash, peer_ip, peer_port, observed_at, bucket_hour)
            VALUES ($1, $2::inet, $3, $4, date_trunc('hour', $4))
            "#,
        )
        .bind(&info_hash)
        .bind(peer_addr.ip().to_string())
        .bind(i32::from(peer_addr.port()))
        .bind(observed_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO torrent_hot_stats_hourly (
                bucket_hour,
                info_hash,
                observation_count,
                unique_peer_count,
                trend_score
            )
            SELECT
                date_trunc('hour', $1::timestamptz) AS bucket_hour,
                $2,
                COUNT(*)::int AS observation_count,
                COUNT(DISTINCT (peer_ip, peer_port))::int AS unique_peer_count,
                (COUNT(*) + COUNT(DISTINCT (peer_ip, peer_port)) * 1.5)::double precision AS trend_score
            FROM dht_observations
            WHERE info_hash = $2
              AND bucket_hour = date_trunc('hour', $1::timestamptz)
            GROUP BY bucket_hour
            ON CONFLICT (bucket_hour, info_hash) DO UPDATE
            SET
                observation_count = EXCLUDED.observation_count,
                unique_peer_count = EXCLUDED.unique_peer_count,
                trend_score = EXCLUDED.trend_score
            "#,
        )
        .bind(observed_at)
        .bind(&info_hash)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

fn is_valid_info_hash(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn path_depth(path: &str) -> i16 {
    let segments = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    segments.saturating_sub(1) as i16
}

fn file_ext(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .and_then(|ext| if ext.is_empty() { None } else { Some(ext) })
}
