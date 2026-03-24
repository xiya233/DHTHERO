# DHT Magnet Search Engine

Monorepo structure:

- `frontend`: Next.js App Router + Tailwind CSS (pnpm)
- `backend`: Rust + Axum + dht-crawler + PostgreSQL

## Prerequisites

- [mise](https://mise.jdx.dev/)
- PostgreSQL 14+

## Bootstrap

```bash
mise install
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env.local
```

## Backend

```bash
cd backend
cargo run
```

Server starts on `http://localhost:8080` by default and runs SQL migrations automatically.

Optional Meilisearch integration (MVP):

- Start local Meilisearch via `docker compose up -d meilisearch`
- Enable in `backend/.env`:
  - `FEATURE_MEILI_ENABLED=true`
  - `MEILI_URL=http://127.0.0.1:7700`
- Incremental sync is batched by queue (defaults):
  - `MEILI_SYNC_BATCH_SIZE=256`
  - `MEILI_SYNC_FLUSH_INTERVAL_MS=250`
  - `MEILI_SYNC_QUEUE_CAPACITY=20000`
- `/api/v1/search` will use Meilisearch for `sort=relevance|latest`
- On Meilisearch timeout/failure, backend automatically falls back to PostgreSQL search

Crawler ingest throughput tuning:

- `CRAWLER_INGEST_WORKER_COUNT=0` enables auto worker sizing:
  - `clamp(2 * available_parallelism, 4, max(1, DATABASE_MAX_CONNECTIONS - 4))`
- `CRAWLER_INGEST_QUEUE_CAPACITY=50000` controls backpressure queue depth before dropping
- Ingest now uses bounded queue + concurrent workers (non-blocking drop-on-full) and only writes
  `torrent_files` on first-seen info_hash to reduce write amplification

## Frontend

```bash
cd frontend
npx pnpm@latest install
npx pnpm@latest dev
```

Frontend starts on `http://localhost:3000` and calls backend through `NEXT_PUBLIC_API_BASE_URL`.

## Core API

- `GET /api/v1/healthz`
- `GET /api/v1/features`
- `GET /api/v1/site/stats`
- `GET /api/v1/categories`
- `GET /api/v1/search?q=...`
- `GET /api/v1/latest`
- `GET /api/v1/trending?window=24h|72h|7d`
- `GET /api/v1/torrents/{info_hash}`
- `GET /api/v1/torrents/{info_hash}/files?flat=true|false`
