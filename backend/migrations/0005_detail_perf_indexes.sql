-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_torrent_files_hash_file_index_cover
ON torrent_files (info_hash, file_index) INCLUDE (path, size, depth);
