use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc,
    time::{MissedTickBehavior, interval, sleep},
};
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    db::{models::MeiliDocRow, repo},
    state::{AppState, MeiliStatus},
};

use super::client::{MeiliSearchClient, MeiliTorrentDoc};

pub fn spawn_incremental_worker(
    pool: sqlx::PgPool,
    config: Arc<AppConfig>,
    client: Option<Arc<MeiliSearchClient>>,
    mut rx: mpsc::Receiver<String>,
) {
    if !config.features.meili_enabled {
        return;
    }

    let Some(client) = client else {
        return;
    };

    tokio::spawn(async move {
        let max_batch = config.meili.sync_batch_size.max(1);
        let flush_every = Duration::from_millis(config.meili.sync_flush_interval_ms.max(50));
        let mut ticker = interval(flush_every);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut pending = Vec::<String>::with_capacity(max_batch);
        let mut dedup = HashSet::<String>::with_capacity(max_batch.saturating_mul(2));
        let mut dropped = 0u64;
        let mut last_report = Instant::now();
        let report_every = Duration::from_secs(10);

        loop {
            tokio::select! {
                maybe_hash = rx.recv() => {
                    let Some(info_hash) = maybe_hash else {
                        break;
                    };
                    if dedup.insert(info_hash.clone()) {
                        pending.push(info_hash);
                    }

                    if pending.len() >= max_batch {
                        flush_pending(&pool, &client, &mut pending, &mut dedup).await;
                    }
                }
                _ = ticker.tick() => {
                    if !pending.is_empty() {
                        flush_pending(&pool, &client, &mut pending, &mut dedup).await;
                    }

                    if dropped > 0 && last_report.elapsed() >= report_every {
                        warn!(dropped, "meili sync queue had dropped items (backpressure)");
                        dropped = 0;
                        last_report = Instant::now();
                    }
                }
            }
        }

        if !pending.is_empty() {
            flush_pending(&pool, &client, &mut pending, &mut dedup).await;
        }
    });
}

pub fn spawn_bootstrap_job(state: AppState) {
    if !state.config.features.meili_enabled || !state.config.meili.bootstrap_enabled {
        return;
    }

    tokio::spawn(async move {
        let Some(client) = state.meili_client() else {
            state.set_meili_status(MeiliStatus::Failed).await;
            return;
        };

        state.set_meili_status(MeiliStatus::Syncing).await;

        let batch_size = i64::from(state.config.meili.bootstrap_batch_size.max(1));
        let mut offset = 0i64;
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(30);

        loop {
            let rows = match repo::fetch_meili_docs_batch(&state.pool, batch_size, offset).await {
                Ok(rows) => rows,
                Err(err) => {
                    warn!(error = ?err, offset, "failed to fetch meili bootstrap batch from postgres");
                    sleep(backoff).await;
                    backoff = next_backoff(backoff, max_backoff);
                    continue;
                }
            };

            if rows.is_empty() {
                break;
            }

            let docs: Vec<MeiliTorrentDoc> = rows.into_iter().map(to_meili_doc).collect();
            match client.upsert_documents(&docs).await {
                Ok(_) => {
                    offset += docs.len() as i64;
                    backoff = Duration::from_secs(1);
                    if offset % (batch_size * 10) == 0 {
                        info!(indexed = offset, "meili bootstrap progress");
                    }
                }
                Err(err) => {
                    warn!(error = ?err, offset, "failed to upsert meili bootstrap batch");
                    sleep(backoff).await;
                    backoff = next_backoff(backoff, max_backoff);
                }
            }
        }

        state.set_meili_status(MeiliStatus::Ready).await;
        info!(indexed = offset, "meili bootstrap completed");
    });
}

pub fn try_enqueue_incremental(state: &AppState, info_hash: String) {
    if !state.config.features.meili_enabled {
        return;
    }

    if state.enqueue_meili_sync(info_hash.clone()) {
        return;
    }

    warn!(
        info_hash = %info_hash,
        "meili sync queue is full or unavailable, skipping incremental sync"
    );
}

pub fn to_meili_doc(row: MeiliDocRow) -> MeiliTorrentDoc {
    MeiliTorrentDoc {
        info_hash: row.info_hash,
        name: row.name,
        category: row.category,
        last_seen_at: row.last_seen_at.timestamp_millis(),
        total_size: row.total_size,
        file_count: row.file_count,
        first_seen_at: row.first_seen_at.timestamp_millis(),
    }
}

fn next_backoff(current: Duration, max_backoff: Duration) -> Duration {
    let doubled_secs = current.as_secs().saturating_mul(2).max(1);
    Duration::from_secs(doubled_secs.min(max_backoff.as_secs()))
}

async fn flush_pending(
    pool: &sqlx::PgPool,
    client: &MeiliSearchClient,
    pending: &mut Vec<String>,
    dedup: &mut HashSet<String>,
) {
    if pending.is_empty() {
        return;
    }

    let current_batch = std::mem::take(pending);
    dedup.clear();

    let mut backoff = Duration::from_millis(200);
    let max_backoff = Duration::from_secs(5);

    loop {
        match repo::fetch_meili_docs_by_info_hashes(pool, &current_batch).await {
            Ok(rows) => {
                let docs: Vec<MeiliTorrentDoc> = rows.into_iter().map(to_meili_doc).collect();
                if docs.is_empty() {
                    return;
                }

                match client.upsert_documents(&docs).await {
                    Ok(_) => return,
                    Err(err) => {
                        warn!(
                            error = ?err,
                            count = docs.len(),
                            "failed to upsert incremental meili batch, retrying"
                        );
                    }
                }
            }
            Err(err) => {
                warn!(
                    error = ?err,
                    count = current_batch.len(),
                    "failed to fetch incremental meili batch from postgres, retrying"
                );
            }
        }

        sleep(backoff).await;
        backoff = next_backoff(backoff, max_backoff);
    }
}
