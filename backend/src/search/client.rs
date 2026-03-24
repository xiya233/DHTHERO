use anyhow::{Context, Result, anyhow};
use meilisearch_sdk::{client::Client, indexes::Index, search::SearchResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;

use crate::config::MeiliConfig;

#[derive(Debug, Clone, Copy)]
pub enum MeiliSearchSort {
    Relevance,
    Latest,
}

#[derive(Debug, Clone)]
pub struct MeiliSearchRequest {
    pub query: String,
    pub category: Option<i16>,
    pub sort: MeiliSearchSort,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct MeiliSearchResponse {
    pub info_hashes: Vec<String>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeiliTorrentDoc {
    pub info_hash: String,
    pub name: String,
    pub category: i16,
    pub last_seen_at: i64,
    pub total_size: i64,
    pub file_count: i32,
    pub first_seen_at: i64,
}

#[derive(Clone)]
pub struct MeiliSearchClient {
    client: Client,
    index_name: String,
    query_timeout: Duration,
}

impl MeiliSearchClient {
    pub fn new(config: &MeiliConfig) -> Result<Self> {
        let client =
            Client::new(&config.url, config.api_key.as_deref()).context("invalid meili config")?;

        Ok(Self {
            client,
            index_name: config.index.clone(),
            query_timeout: Duration::from_millis(config.query_timeout_ms.max(100)),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        match self
            .client
            .create_index(&self.index_name, Some("info_hash"))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                let err_text = err.to_string();
                if !err_text.contains("index_already_exists") {
                    return Err(anyhow!("failed to create meili index: {err}"));
                }
            }
        }

        let index = self.index();
        if let Err(err) = index.set_filterable_attributes(["category"]).await {
            warn!(error = ?err, "failed to configure meili filterable attributes");
        }
        if let Err(err) = index.set_sortable_attributes(["last_seen_at"]).await {
            warn!(error = ?err, "failed to configure meili sortable attributes");
        }

        Ok(())
    }

    pub async fn upsert_documents(&self, docs: &[MeiliTorrentDoc]) -> Result<()> {
        if docs.is_empty() {
            return Ok(());
        }

        self.index()
            .add_documents(docs, Some("info_hash"))
            .await
            .context("failed to upsert meili documents")?;

        Ok(())
    }

    pub async fn search(&self, request: &MeiliSearchRequest) -> Result<MeiliSearchResponse> {
        let index = self.index();
        let mut query = index.search();
        query.with_query(&request.query);
        query.with_limit(request.limit);
        query.with_offset(request.offset);

        let filter;
        if let Some(category) = request.category {
            filter = format!("category = {category}");
            query.with_filter(&filter);
        }

        let latest_sort;
        if matches!(request.sort, MeiliSearchSort::Latest) {
            latest_sort = ["last_seen_at:desc"];
            query.with_sort(&latest_sort);
        }

        let response = timeout(self.query_timeout, query.execute::<MeiliTorrentDoc>())
            .await
            .map_err(|_| anyhow!("meili search timed out"))?
            .context("failed to execute meili search")?;

        let info_hashes = extract_hits(response.hits);
        let total = response
            .estimated_total_hits
            .map(|value| value as i64)
            .unwrap_or(info_hashes.len() as i64);

        Ok(MeiliSearchResponse { info_hashes, total })
    }

    fn index(&self) -> Index {
        self.client.index(&self.index_name)
    }
}

fn extract_hits(hits: Vec<SearchResult<MeiliTorrentDoc>>) -> Vec<String> {
    hits.into_iter().map(|hit| hit.result.info_hash).collect()
}
