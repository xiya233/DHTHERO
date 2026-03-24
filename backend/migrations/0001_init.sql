CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS torrents (
    info_hash CHAR(40) PRIMARY KEY,
    magnet_link TEXT NOT NULL,
    name TEXT NOT NULL,
    name_tsv TSVECTOR GENERATED ALWAYS AS (to_tsvector('simple', COALESCE(name, ''))) STORED,
    category SMALLINT NOT NULL DEFAULT 5 CHECK (category BETWEEN 1 AND 5),
    total_size BIGINT NOT NULL DEFAULT 0,
    piece_length BIGINT NOT NULL DEFAULT 0,
    file_count INT NOT NULL DEFAULT 0,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_peer_ip INET,
    last_peer_port INT CHECK (last_peer_port BETWEEN 0 AND 65535),
    fetch_success_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (info_hash ~ '^[0-9a-fA-F]{40}$')
);

CREATE TABLE IF NOT EXISTS torrent_files (
    id BIGSERIAL PRIMARY KEY,
    info_hash CHAR(40) NOT NULL REFERENCES torrents(info_hash) ON DELETE CASCADE,
    file_index INT NOT NULL,
    path TEXT NOT NULL,
    size BIGINT NOT NULL DEFAULT 0,
    depth SMALLINT NOT NULL DEFAULT 0,
    ext TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (info_hash, file_index)
);

CREATE TABLE IF NOT EXISTS torrent_hot_stats_hourly (
    bucket_hour TIMESTAMPTZ NOT NULL,
    info_hash CHAR(40) NOT NULL REFERENCES torrents(info_hash) ON DELETE CASCADE,
    observation_count INT NOT NULL DEFAULT 0,
    unique_peer_count INT NOT NULL DEFAULT 0,
    trend_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    PRIMARY KEY (bucket_hour, info_hash)
);

CREATE TABLE IF NOT EXISTS dht_observations (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    info_hash CHAR(40) NOT NULL REFERENCES torrents(info_hash) ON DELETE CASCADE,
    peer_ip INET NOT NULL,
    peer_port INT NOT NULL CHECK (peer_port BETWEEN 0 AND 65535),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    bucket_hour TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id, observed_at),
    CHECK (bucket_hour = date_trunc('hour', observed_at))
) PARTITION BY RANGE (observed_at);

CREATE TABLE IF NOT EXISTS search_audit_logs (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    query_text TEXT NOT NULL,
    category SMALLINT,
    sort TEXT NOT NULL,
    page INT NOT NULL,
    page_size INT NOT NULL,
    client_ip INET,
    user_agent TEXT,
    referer TEXT,
    result_count INT NOT NULL DEFAULT 0,
    latency_ms INT NOT NULL DEFAULT 0,
    status_code INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_torrents_set_updated_at ON torrents;
CREATE TRIGGER trg_torrents_set_updated_at
BEFORE UPDATE ON torrents
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX IF NOT EXISTS idx_torrents_name_tsv ON torrents USING GIN (name_tsv);
CREATE INDEX IF NOT EXISTS idx_torrents_last_seen_desc ON torrents (last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_torrents_category_last_seen ON torrents (category, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_torrent_files_hash ON torrent_files (info_hash);
CREATE INDEX IF NOT EXISTS idx_torrent_files_ext ON torrent_files (ext);
CREATE INDEX IF NOT EXISTS idx_hot_stats_bucket_trend ON torrent_hot_stats_hourly (bucket_hour DESC, trend_score DESC);
CREATE INDEX IF NOT EXISTS idx_dht_observations_hash_observed ON dht_observations (info_hash, observed_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_audit_logs_created ON search_audit_logs (created_at DESC);
