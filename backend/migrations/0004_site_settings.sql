CREATE TABLE IF NOT EXISTS site_settings (
    id SMALLINT PRIMARY KEY CHECK (id = 1),
    site_title TEXT NOT NULL,
    site_description TEXT NOT NULL,
    home_hero_markdown TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO site_settings (
    id,
    site_title,
    site_description,
    home_hero_markdown
) VALUES (
    1,
    'DHT_MAGNET',
    'Bauhaus inspired DHT magnet search engine',
    E'SEARCH\nTHE_NET'
)
ON CONFLICT (id) DO NOTHING;
