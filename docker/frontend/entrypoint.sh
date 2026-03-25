#!/usr/bin/env bash
set -euo pipefail

PUID="${PUID:-1000}"
PGID="${PGID:-1000}"

if ! [[ "${PUID}" =~ ^[0-9]+$ ]] || ! [[ "${PGID}" =~ ^[0-9]+$ ]]; then
  echo "PUID and PGID must be numeric values" >&2
  exit 1
fi

mkdir -p /data
chown -R "${PUID}:${PGID}" /data

exec gosu "${PUID}:${PGID}" "$@"
