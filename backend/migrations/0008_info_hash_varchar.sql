ALTER TABLE torrent_files
DROP CONSTRAINT IF EXISTS torrent_files_info_hash_fkey;

ALTER TABLE torrent_hot_stats_hourly
DROP CONSTRAINT IF EXISTS torrent_hot_stats_hourly_info_hash_fkey;

ALTER TABLE dht_observations
DROP CONSTRAINT IF EXISTS dht_observations_info_hash_fkey;

ALTER TABLE torrent_hourly_unique_peers
DROP CONSTRAINT IF EXISTS torrent_hourly_unique_peers_info_hash_fkey;

ALTER TABLE torrents
ALTER COLUMN info_hash TYPE VARCHAR(40);

ALTER TABLE torrent_files
ALTER COLUMN info_hash TYPE VARCHAR(40);

ALTER TABLE torrent_hot_stats_hourly
ALTER COLUMN info_hash TYPE VARCHAR(40);

ALTER TABLE dht_observations
ALTER COLUMN info_hash TYPE VARCHAR(40);

ALTER TABLE torrent_hourly_unique_peers
ALTER COLUMN info_hash TYPE VARCHAR(40);

ALTER TABLE torrent_files
ADD CONSTRAINT torrent_files_info_hash_fkey
FOREIGN KEY (info_hash) REFERENCES torrents(info_hash) ON DELETE CASCADE;

ALTER TABLE torrent_hot_stats_hourly
ADD CONSTRAINT torrent_hot_stats_hourly_info_hash_fkey
FOREIGN KEY (info_hash) REFERENCES torrents(info_hash) ON DELETE CASCADE;

ALTER TABLE dht_observations
ADD CONSTRAINT dht_observations_info_hash_fkey
FOREIGN KEY (info_hash) REFERENCES torrents(info_hash) ON DELETE CASCADE;

ALTER TABLE torrent_hourly_unique_peers
ADD CONSTRAINT torrent_hourly_unique_peers_info_hash_fkey
FOREIGN KEY (info_hash) REFERENCES torrents(info_hash) ON DELETE CASCADE;

ANALYZE torrents;
ANALYZE torrent_files;
ANALYZE torrent_hot_stats_hourly;
