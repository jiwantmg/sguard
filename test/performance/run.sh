#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"          # so it works from any directory

URL="${URL:-http://127.0.0.1:8080/user/api}"
THREADS="${THREADS:-4}"
CONNS="${CONNS:-100}"
DURATION="${DURATION:-30s}"

: "${TOKEN:?set TOKEN, e.g. TOKEN=eyJ... ./run.sh}"

echo "warm-up..."
wrk -t2 -c20 -d5s -s post.lua "$URL" > /dev/null

wrk -t"$THREADS" -c"$CONNS" -d"$DURATION" --latency -s post.lua "$URL"