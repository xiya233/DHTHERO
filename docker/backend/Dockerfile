# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS chef

RUN cargo install --locked cargo-chef

FROM chef AS planner

WORKDIR /workspace
COPY backend ./backend

WORKDIR /workspace/backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS deps

WORKDIR /workspace/backend
COPY --from=planner /workspace/backend/recipe.json ./recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/workspace/backend/target \
  cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder

WORKDIR /workspace
COPY --from=deps /workspace/backend/recipe.json /workspace/backend/recipe.json
COPY backend ./backend

WORKDIR /workspace/backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/workspace/backend/target \
  cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl gosu tini \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/backend/target/release/backend /usr/local/bin/backend
COPY docker/backend/entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN chmod +x /usr/local/bin/docker-entrypoint.sh /usr/local/bin/backend

ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    PUID=1000 \
    PGID=1000

EXPOSE 8080 9000 6881/udp

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/docker-entrypoint.sh"]
CMD ["backend"]
