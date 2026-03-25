# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS builder

WORKDIR /workspace
COPY backend ./backend

WORKDIR /workspace/backend
RUN cargo build --release

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
