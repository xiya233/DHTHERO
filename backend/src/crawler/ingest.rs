use crate::{domain::category::Category, state::AppState};
use anyhow::Result;
use chrono::{DateTime, Utc};
use dht_crawler::TorrentInfo;
use sqlx::{Postgres, QueryBuilder};
use std::net::SocketAddr;

const FILE_INSERT_CHUNK_SIZE: usize = 500;

pub async fn ingest_torrent(state: &AppState, torrent: TorrentInfo) -> Result<()> {
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

    let mut tx = state.pool.begin().await?;

    let (is_new,): (bool,) = sqlx::query_as(
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
        RETURNING (xmax = 0) AS is_new
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
    .fetch_one(&mut *tx)
    .await?;

    if is_new && !torrent.files.is_empty() {
        insert_torrent_files_batch(&mut tx, &info_hash, &torrent).await?;
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

        let unique_peer_result = sqlx::query(
            r#"
            INSERT INTO torrent_hourly_unique_peers (
                bucket_hour,
                info_hash,
                peer_ip,
                peer_port,
                observed_at
            ) VALUES (
                date_trunc('hour', $1::timestamptz),
                $2,
                $3::inet,
                $4,
                $1
            )
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(observed_at)
        .bind(&info_hash)
        .bind(peer_addr.ip().to_string())
        .bind(i32::from(peer_addr.port()))
        .execute(&mut *tx)
        .await?;
        let unique_peer_increment: i32 = if unique_peer_result.rows_affected() > 0 {
            1
        } else {
            0
        };

        sqlx::query(
            r#"
            INSERT INTO torrent_hot_stats_hourly (
                bucket_hour,
                info_hash,
                observation_count,
                unique_peer_count,
                trend_score
            ) VALUES (
                date_trunc('hour', $1::timestamptz),
                $2,
                1,
                $3,
                (1 + $3::double precision * 1.5)
            )
            ON CONFLICT (bucket_hour, info_hash) DO UPDATE
            SET
                observation_count = torrent_hot_stats_hourly.observation_count + 1,
                unique_peer_count = torrent_hot_stats_hourly.unique_peer_count + EXCLUDED.unique_peer_count,
                trend_score = (
                    (torrent_hot_stats_hourly.observation_count + 1)
                    + (
                        torrent_hot_stats_hourly.unique_peer_count
                        + EXCLUDED.unique_peer_count
                    ) * 1.5
                )
            "#,
        )
        .bind(observed_at)
        .bind(&info_hash)
        .bind(unique_peer_increment)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    if state.config.features.meili_enabled {
        crate::search::sync::try_enqueue_incremental(state, info_hash.clone());
    }

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

async fn insert_torrent_files_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    info_hash: &str,
    torrent: &TorrentInfo,
) -> Result<()> {
    for (chunk_idx, chunk) in torrent.files.chunks(FILE_INSERT_CHUNK_SIZE).enumerate() {
        let mut qb = QueryBuilder::<Postgres>::new(
            "INSERT INTO torrent_files (info_hash, file_index, path, size, depth, ext) ",
        );
        let base_index = chunk_idx * FILE_INSERT_CHUNK_SIZE;

        qb.push_values(chunk.iter().enumerate(), |mut builder, (offset, file)| {
            let file_index = i32::try_from(base_index + offset).unwrap_or(i32::MAX);
            let file_size = i64::try_from(file.size).unwrap_or(i64::MAX);
            let depth = path_depth(&file.path);
            let ext = file_ext(&file.path);

            builder
                .push_bind(info_hash)
                .push_bind(file_index)
                .push_bind(&file.path)
                .push_bind(file_size)
                .push_bind(depth)
                .push_bind(ext);
        });

        qb.build().execute(&mut **tx).await?;
    }

    Ok(())
}
