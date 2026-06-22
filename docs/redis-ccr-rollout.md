# Redis CCR Rollout Notes

Rust Headroom proxy can use Redis as a shared CCR backend for multi-host
deployments. Enable it only after the runtime image has passed the Redis CCR
smoke test.

## Runtime Configuration

- `HEADROOM_CCR_BACKEND=redis`
- `HEADROOM_REDIS_URL=redis://...`
- `HEADROOM_CCR_TENANT_PREFIX=<deployment-or-tenant-prefix>`

The proxy initializes Redis during startup. Missing, malformed, or unreachable
Redis configuration must fail startup; operators must not silently fall back to
SQLite or in-memory CCR.

## Production Prerequisites

- Persistence: Redis must use durable storage or a managed Redis tier with
  persistence guarantees appropriate for reversible CCR originals.
- Memory sizing: size Redis for peak compressed-turn payload volume,
  configured TTL, tenant prefix cardinality, and expected horizontal worker
  count.
- Eviction policy: do not use an eviction policy that can remove live CCR keys
  before their TTL unless the rollback/runbook accepts retrieval misses.
- Secrets: keep Redis URL, credentials, and TLS material in the deployment
  secret manager; never commit production URLs or passwords.
- Network: expose Redis only to Headroom runtime workloads; no public Redis
  endpoint.
- TLS/auth: use Redis authentication and TLS where the platform supports it,
  especially outside a private single-cluster network.
- Rollback: switch `HEADROOM_CCR_BACKEND` back to the previous backend and keep
  Redis data until the incident owner approves disposal.
- Observability: verify `/v1/retrieve/stats` reports `redis` before sending
  production traffic through the deployment.

## Smoke Command

Use the image smoke script before a production switch:

```bash
CONTAINER_CLI=docker IMAGE=headroom-redis-ccr-smoke:local scripts/smoke_redis_ccr_image.sh
```

The script starts a Redis container, launches the shipped Rust `headroom-proxy`
binary with Redis env vars, and requires `/v1/retrieve/stats` to report
`"backend":"redis"`.
