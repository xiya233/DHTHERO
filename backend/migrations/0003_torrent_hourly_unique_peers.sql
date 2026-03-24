CREATE TABLE IF NOT EXISTS torrent_hourly_unique_peers (
    bucket_hour TIMESTAMPTZ NOT NULL,
    info_hash CHAR(40) NOT NULL REFERENCES torrents(info_hash) ON DELETE CASCADE,
    peer_ip INET NOT NULL,
    peer_port INT NOT NULL CHECK (peer_port BETWEEN 0 AND 65535),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (bucket_hour, info_hash, peer_ip, peer_port)
) PARTITION BY RANGE (bucket_hour);

DO $$
DECLARE
    base_month DATE := date_trunc('month', NOW())::date;
    start_date DATE;
    end_date DATE;
    idx INT;
BEGIN
    FOR idx IN 0..1 LOOP
        start_date := (base_month + (idx || ' month')::interval)::date;
        end_date := (start_date + interval '1 month')::date;

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS torrent_hourly_unique_peers_%s PARTITION OF torrent_hourly_unique_peers FOR VALUES FROM (%L) TO (%L)',
            to_char(start_date, 'YYYYMM'),
            start_date,
            end_date
        );
    END LOOP;
END;
$$;

CREATE TABLE IF NOT EXISTS torrent_hourly_unique_peers_default
PARTITION OF torrent_hourly_unique_peers DEFAULT;
