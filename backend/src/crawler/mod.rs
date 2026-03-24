mod ingest;

use crate::state::{AppState, CrawlerStatus};
use anyhow::Result;
use dht_crawler::{DHTOptions, DHTServer, NetMode, TorrentInfo};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub async fn spawn(state: AppState) -> Result<()> {
    let config = state.config.crawler.clone();

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
    let (tx, mut rx) = mpsc::unbounded_channel::<TorrentInfo>();

    server.on_torrent(move |torrent| {
        if tx.send(torrent).is_err() {
            warn!("crawler ingestion queue receiver dropped");
        }
    });

    server.on_metadata_fetch(|_hash| async move { true });

    server.on_error(|err| {
        error!(error = ?err, "dht crawler runtime error");
    });

    let ingest_state = state.clone();
    tokio::spawn(async move {
        while let Some(torrent) = rx.recv().await {
            if let Err(err) = ingest::ingest_torrent(&ingest_state, torrent).await {
                error!(error = ?err, "failed to ingest torrent");
            }
        }
    });

    let run_state = state.clone();
    tokio::spawn(async move {
        run_state.set_crawler_status(CrawlerStatus::Running).await;
        info!("dht crawler started");

        if let Err(err) = server.start().await {
            error!(error = ?err, "dht crawler stopped with error");
            run_state.set_crawler_status(CrawlerStatus::Failed).await;
        }
    });

    Ok(())
}

fn parse_netmode(raw: &str) -> NetMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ipv6" | "ipv6only" => NetMode::Ipv6Only,
        "dual" | "dualstack" => NetMode::DualStack,
        _ => NetMode::Ipv4Only,
    }
}
