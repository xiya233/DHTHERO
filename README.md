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
