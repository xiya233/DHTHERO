# DHT Magnet Search Engine

Monorepo structure:

- `frontend`: Next.js App Router + Tailwind CSS (pnpm)
- `backend`: Rust + Axum + dht-crawler + PostgreSQL

## Prerequisites

- [mise](https://mise.jdx.dev/)
- PostgreSQL 14+
- Docker + Docker Compose

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

Admin dashboard (crawler metrics):

- Backend env:
  - `ADMIN_DASHBOARD_PASSWORD=your_password`
  - `CORS_ALLOWED_ORIGINS=http://localhost:3000` (set your frontend domain in production)
  - `CRAWLER_PROMETHEUS_ENABLED=true`
  - `CRAWLER_PROMETHEUS_LISTEN_ADDR=0.0.0.0:9000`
- Frontend env:
  - `ADMIN_DASHBOARD_PASSWORD=your_password`
  - `SESSION_SECRET=long_random_secret`
- Visit:
  - `http://localhost:3000/admin/login`
  - Prometheus metrics endpoint: `http://localhost:9000/metrics`
  - Site copy (title/description/home hero markdown) can be edited in `/admin`

Private mode (site + backend API password gate):

- Backend env:
  - `PRIVATE_MODE_ENABLED=true`
  - `PRIVATE_SITE_PASSWORD=your_password`
- Frontend env:
  - `PRIVATE_MODE_ENABLED=true`
  - `PRIVATE_SITE_PASSWORD=your_password`
  - `SESSION_SECRET=long_random_secret`
- Behavior:
  - When enabled, browser must login at `http://localhost:3000/login`
  - Backend requires header `x-site-password` for `/api/v1/*` except `/api/v1/healthz`
  - Frontend login/admin sessions use signed cookie tokens (no plaintext password in cookie)
  - Login endpoints include in-memory rate limit for brute-force mitigation
  - `/admin` remains double-protected (site login + admin password)

## Frontend

```bash
cd frontend
npx pnpm@latest install
npx pnpm@latest dev
```

Frontend starts on `http://localhost:3000` and calls backend through `NEXT_PUBLIC_API_BASE_URL`.

## Production Docker Compose

This repository includes a production-oriented compose stack:

- `frontend` + `backend` use prebuilt GHCR images
- `postgres` + `meilisearch` use official images
- all services use bind mounts under `./data`
- one command starts everything (`frontend + backend + postgres + meilisearch`)

### 1) Prepare compose variables

```bash
cp .env.example .env
```

Set at least:

- `GHCR_NAMESPACE` (your GitHub user/org)
- `APP_IMAGE_TAG` (`latest`/`sha-*`/`v*`)
- `PUID` and `PGID` (host user/group id for bind mount ownership)

### 2) Prepare service env files

```bash
cp docker/env/postgres.env.example docker/env/postgres.env
cp docker/env/meilisearch.env.example docker/env/meilisearch.env
cp docker/env/backend.env.example docker/env/backend.env
cp docker/env/frontend.env.example docker/env/frontend.env
```

Each variable in these files is documented inline with detailed comments and usage guidance.

### 3) Prepare bind mount directories

```bash
mkdir -p data/frontend data/backend data/postgres data/meilisearch
```

### 4) Start all services

```bash
docker compose pull
docker compose up -d
```

### 5) Access

- Frontend: `http://localhost:3000`
- Backend API: `http://localhost:8080`

`postgres` and `meilisearch` are internal-only by default (not exposed to host ports).

If you want to expose the stack through host-installed Nginx (not Dockerized Nginx), see:

- [Nginx host reverse proxy guide](docs/nginx-host-reverse-proxy.md)

## GHCR Publish Workflow

GitHub Actions workflow: `.github/workflows/ghcr-publish.yml`

- Trigger: push `main`, push tag `v*`, or manual dispatch
- Publishes:
  - `ghcr.io/<owner>/dht-backend`
  - `ghcr.io/<owner>/dht-frontend`
- Architectures: `linux/amd64`, `linux/arm64`
- Tags: `latest` (default branch), `sha-<short>`, release tag names

## Core API

- `GET /api/v1/healthz`
- `GET /api/v1/features`
- `GET /api/v1/site/stats`
- `GET /api/v1/site/content`
- `GET /api/v1/categories`
- `GET /api/v1/search?q=...`
- `GET /api/v1/latest`
- `GET /api/v1/trending?window=24h|72h|7d`
- `GET /api/v1/torrents/{info_hash}`
- `GET /api/v1/torrents/{info_hash}/files?flat=true|false`
- `GET /api/v1/admin/site-settings` (admin password required)
- `PUT /api/v1/admin/site-settings` (admin password required)
