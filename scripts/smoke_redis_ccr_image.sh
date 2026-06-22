#!/usr/bin/env bash
set -euo pipefail

IMAGE="${IMAGE:-headroom-redis-ccr-smoke:local}"
CONTAINER_CLI="${CONTAINER_CLI:-docker}"
NETWORK="headroom-redis-ccr-smoke-$$"
REDIS="headroom-redis-ccr-smoke-redis-$$"
PROXY="headroom-redis-ccr-smoke-proxy-$$"

cleanup() {
  "$CONTAINER_CLI" rm -f "$PROXY" "$REDIS" >/dev/null 2>&1 || true
  "$CONTAINER_CLI" network rm "$NETWORK" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$CONTAINER_CLI" build -t "$IMAGE" .
"$CONTAINER_CLI" network create "$NETWORK" >/dev/null
"$CONTAINER_CLI" run -d --name "$REDIS" --network "$NETWORK" redis:7-alpine >/dev/null

"$CONTAINER_CLI" run -d \
  --name "$PROXY" \
  --network "$NETWORK" \
  --entrypoint headroom-proxy \
  -e HEADROOM_CCR_BACKEND=redis \
  -e HEADROOM_REDIS_URL=redis://"$REDIS":6379 \
  -e HEADROOM_CCR_TENANT_PREFIX=headroom-smoke \
  -p 127.0.0.1::8787 \
  "$IMAGE" \
  --upstream http://127.0.0.1:9 >/dev/null

host_port="$("$CONTAINER_CLI" port "$PROXY" 8787/tcp | awk -F: 'NR==1 {print $NF}')"
if [ -z "$host_port" ]; then
  echo "ERROR: could not resolve mapped proxy port" >&2
  exit 1
fi

for _ in $(seq 1 60); do
  if curl --fail --silent "http://127.0.0.1:${host_port}/v1/retrieve/stats" \
    | tee /tmp/headroom-redis-ccr-smoke-stats.json \
    | grep -q '"backend":"redis"'; then
    echo "Redis CCR image smoke OK"
    cat /tmp/headroom-redis-ccr-smoke-stats.json
    exit 0
  fi
  sleep 1
done

"$CONTAINER_CLI" logs "$PROXY" >&2 || true
echo "ERROR: Redis CCR image smoke did not report redis backend" >&2
exit 1
