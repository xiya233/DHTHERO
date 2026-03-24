use anyhow::Result;

use crate::{
    db::{
        models::TorrentListRow,
        repo::{self, SearchParams, SearchSort},
    },
    state::AppState,
};

use super::client::{MeiliSearchRequest, MeiliSearchSort};

pub async fn search_torrents(
    state: &AppState,
    params: &SearchParams,
) -> Result<Option<(Vec<TorrentListRow>, i64)>> {
    if !state.config.features.meili_enabled {
        return Ok(None);
    }

    let Some(client) = state.meili_client() else {
        return Ok(None);
    };

    let meili_sort = match params.sort {
        SearchSort::Relevance => MeiliSearchSort::Relevance,
        SearchSort::Latest => MeiliSearchSort::Latest,
        SearchSort::Hot | SearchSort::SizeAsc | SearchSort::SizeDesc => return Ok(None),
    };

    let request = MeiliSearchRequest {
        query: params.q.clone(),
        category: params.category,
        sort: meili_sort,
        limit: usize::try_from(params.limit.max(0)).unwrap_or(usize::MAX),
        offset: usize::try_from(params.offset.max(0)).unwrap_or(0),
    };

    let result = client.search(&request).await?;

    if result.info_hashes.is_empty() {
        return Ok(Some((Vec::new(), result.total)));
    }

    let rows = repo::fetch_torrents_by_info_hashes(&state.pool, &result.info_hashes).await?;

    Ok(Some((rows, result.total)))
}
