#!/usr/bin/env bash
# Usage: JWT_SECRET=... ./gen_token.sh   (optional: SUB, AUD, TTL seconds)
set -euo pipefail
: "${JWT_SECRET:?set JWT_SECRET}"

b64url() { openssl base64 -A | tr '+/' '-_' | tr -d '='; }

now=$(date +%s)
header='{"alg":"HS256","typ":"JWT"}'
payload=$(printf '{"sub":"%s","aud":"%s","iat":%d,"exp":%d}' \
  "${SUB:-test}" "${AUD:-www.lekhabook.com}" "$now" "$((now + ${TTL:-2592000}))")

h=$(printf '%s' "$header"  | b64url)
p=$(printf '%s' "$payload" | b64url)
s=$(printf '%s.%s' "$h" "$p" | openssl dgst -sha256 -hmac "$JWT_SECRET" -binary | b64url)

echo "$h.$p.$s"