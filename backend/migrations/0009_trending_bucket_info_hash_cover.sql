-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_hot_stats_bucket_info_hash_cover
ON torrent_hot_stats_hourly (bucket_hour DESC, info_hash) INCLUDE (trend_score, observation_count);
