use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::{
    db::{models::MeiliDocRow, repo},
    state::{AppState, MeiliStatus},
};

use super::client::MeiliTorrentDoc;

pub async fn sync_single_document(state: &AppState, info_hash: &str) -> Result<()> {
    if !state.config.features.meili_enabled {
        return Ok(());
    }

    let Some(client) = state.meili_client() else {
        return Ok(());
    };

    let Some(row) = repo::fetch_meili_doc_by_info_hash(&state.pool, info_hash).await? else {
        return Ok(());
    };

    client.upsert_documents(&[to_meili_doc(row)]).await
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
