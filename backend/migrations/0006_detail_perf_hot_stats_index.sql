-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_hot_stats_info_hash_bucket_desc
ON torrent_hot_stats_hourly (info_hash, bucket_hour DESC);
