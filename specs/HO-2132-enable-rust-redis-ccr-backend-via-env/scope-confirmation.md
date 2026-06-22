# HO-2132 Scope Confirmation

## 1. What will change

Later implementation will wire the Rust Headroom proxy to select Redis CCR by env:

- `HEADROOM_CCR_BACKEND=redis`
- `HEADROOM_REDIS_URL`
- `HEADROOM_CCR_TENANT_PREFIX`

Redis startup/init failure must abort startup and must not fall back to memory.

The current Todo PR changes only SpecKit docs under:
`specs/HO-2132-enable-rust-redis-ccr-backend-via-env/**`

## 2. What will not change

This issue will not:

- provision Redis infrastructure
- switch production to Redis automatically
- expose Redis publicly
- remove SQLite or in-memory CCR backends
- deploy dev-infra manifests
- change Python CCR behavior unless Rust proxy compatibility requires an explicit follow-up decision

## 3. Owning surfaces

Expected later implementation surfaces:

- `crates/headroom-proxy/src/config.rs` for env parsing
- `crates/headroom-proxy/src/proxy.rs` or adjacent module for startup backend initialization and `/v1/retrieve/stats`
- `crates/headroom-proxy/Cargo.toml` or workspace feature wiring for `headroom-core/redis`
- `Dockerfile` and/or image smoke workflow for Redis-capable runtime verification
- `crates/headroom-proxy` tests for config/stats behavior
- `crates/headroom-core/tests/ccr_backends.rs` or equivalent integration path for real Redis roundtrip

## 4. Expected behavior

When Redis env is valid and the binary/image includes Redis support, Rust Headroom starts with Redis CCR and `/v1/retrieve/stats` reports backend `redis`.

When Redis env is invalid, Redis is unavailable, Redis PING fails, or Redis support is not compiled, startup fails loud. Memory fallback is explicitly forbidden.

## 5. UI alignment

UI N/A. This is backend/runtime configuration and image smoke scope; no user-facing frontend route, mockscreen, or visual alignment source is involved.

## 6. Missing part

Missing part: exact Redis image-smoke owner/surface is not selected yet. This matters because acceptance requires a real Redis-backed CCR roundtrip and Redis-capable runtime proof; implementation must choose and document the CI/smoke path before claiming acceptance.

## Scope verdict

Scope bounded: implement env-driven Redis CCR selection for Rust Headroom runtime, add focused tests and Redis-backed smoke, and document dev-infra Redis prerequisites. Current Todo work stops at docs-only scope confirmation.
