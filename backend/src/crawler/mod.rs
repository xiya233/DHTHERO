mod ingest;

use crate::state::{AppState, CrawlerStatus};
use anyhow::Result;
use dht_crawler::{DHTOptions, DHTServer, NetMode, TorrentInfo};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Semaphore, mpsc};
use tracing::{error, info, warn};

pub async fn spawn(state: AppState) -> Result<()> {
    let config = state.config.crawler.clone();
    let ingest_queue_capacity = config.ingest_queue_capacity.max(1);
    let ingest_worker_count = resolve_ingest_worker_count(
        config.ingest_worker_count,
        state.config.database_max_connections,
    );

    let options = DHTOptions {
        port: config.port,
        metadata_timeout: config.metadata_timeout,
        max_metadata_queue_size: config.max_metadata_queue_size,
        max_metadata_worker_count: config.max_metadata_worker_count,
        netmode: parse_netmode(&config.netmode),
        node_queue_capacity: config.node_queue_capacity,
        hash_queue_capacity: config.hash_queue_capacity,
    };

    let server = DHTServer::new(options).await?;
    let (tx, mut rx) = mpsc::channel::<TorrentInfo>(ingest_queue_capacity);
    let dropped_counter = Arc::new(AtomicU64::new(0));
    let closed_counter = Arc::new(AtomicU64::new(0));

    server.on_torrent(move |torrent| match tx.try_send(torrent) {
        Ok(()) => {}
        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
            let dropped = dropped_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if dropped == 1 || dropped % 1_000 == 0 {
                warn!(
                    dropped,
                    "crawler ingest queue is full, dropping torrent event for backpressure"
                );
            }
        }
        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
            let closed = closed_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if closed == 1 {
                warn!("crawler ingest queue receiver dropped");
            }
        }
    });

    server.on_metadata_fetch(|_hash| async move { true });

    server.on_error(|err| {
        error!(error = ?err, "dht crawler runtime error");
    });

    let semaphore = Arc::new(Semaphore::new(ingest_worker_count.max(1)));
    let ingest_state = state.clone();
    tokio::spawn(async move {
        while let Some(torrent) = rx.recv().await {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => break,
            };
            let ingest_state = ingest_state.clone();
            tokio::spawn(async move {
                let _permit = permit;
                if let Err(err) = ingest::ingest_torrent(&ingest_state, torrent).await {
                    error!(error = ?err, "failed to ingest torrent");
                }
            });
        }
    });

    let run_state = state.clone();
    tokio::spawn(async move {
        run_state.set_crawler_status(CrawlerStatus::Running).await;
        info!(
            ingest_worker_count,
            ingest_queue_capacity, "dht crawler started"
        );

        if let Err(err) = server.start().await {
            error!(error = ?err, "dht crawler stopped with error");
            run_state.set_crawler_status(CrawlerStatus::Failed).await;
        }
    });

    Ok(())
}

fn resolve_ingest_worker_count(configured: usize, db_max_connections: u32) -> usize {
    if configured > 0 {
        return configured.max(1);
    }

    let available = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4);
    let suggested = available.saturating_mul(2);
    let upper_bound = usize::try_from(db_max_connections.saturating_sub(4).max(1)).unwrap_or(1);

    if upper_bound < 4 {
        upper_bound.max(1)
    } else {
        suggested.clamp(4, upper_bound)
    }
}

fn parse_netmode(raw: &str) -> NetMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ipv6" | "ipv6only" => NetMode::Ipv6Only,
        "dual" | "dualstack" => NetMode::DualStack,
        _ => NetMode::Ipv4Only,
    }
}
