# HO-2132 Plan - Enable Rust Redis CCR backend via env

## Repo Reality

- `headroom-core` already owns CCR storage abstractions under `crates/headroom-core/src/ccr/backends/`.
- `RedisCcrStore` is compiled only with the `redis` feature and already PINGs during `open`.
- `CcrBackendConfig::Redis { url, ttl_seconds, key_prefix }` already exists and returns `UnsupportedBackend` when the feature is not compiled.
- `headroom-proxy` currently depends on `headroom-core = { path = "../headroom-core" }` without enabling `redis`.
- Rust proxy configuration lives in `crates/headroom-proxy/src/config.rs` using `clap` env parsing.
- Rust proxy startup builds `AppState` in `crates/headroom-proxy/src/main.rs` and `crates/headroom-proxy/src/proxy.rs`.
- Rust proxy routes currently include `/healthz`, `/healthz/upstream`, `/metrics`, and LLM proxy routes; `/v1/retrieve/stats` is not currently visible in `crates/headroom-proxy/src/proxy.rs`.
- The Docker image currently builds the Rust proxy binary without `--features redis`.

## Implementation Shape

1. Add a small Rust proxy CCR runtime config type, preferably in `crates/headroom-proxy/src/config.rs`, that parses:
   - `HEADROOM_CCR_BACKEND`
   - `HEADROOM_REDIS_URL`
   - `HEADROOM_CCR_TENANT_PREFIX`
   - existing TTL env if already supported by core/runtime, otherwise document default TTL behavior.
2. Convert parsed proxy config into `headroom_core::ccr::backends::CcrBackendConfig`.
3. Initialize the selected CCR backend during `AppState::new` or a startup-adjacent constructor so init failure aborts startup.
4. Store backend metadata in `AppState` for `/v1/retrieve/stats`.
5. Add `/v1/retrieve/stats` handler that reports at minimum selected backend and relevant safe metadata. Redis URL must not be leaked.
6. Enable the `redis` feature for the production Rust proxy build path where Redis runtime support is required.
7. Update image smoke to start Redis plus Headroom with Redis env and verify backend stats plus a Redis-backed CCR roundtrip.

## Testing Plan

- Unit tests in `crates/headroom-proxy` for env/config parsing:
  - Redis backend selected with URL.
  - Tenant prefix preserved.
  - Missing URL errors.
  - Empty prefix behavior is explicit.
  - Unsupported backend string errors.
- Core/backend tests:
  - Keep existing `crates/headroom-core/tests/ccr_backends.rs` Redis cfg-gated roundtrip.
  - Add or adjust test coverage only if new prefix behavior needs core-level proof.
- Proxy route tests:
  - `/v1/retrieve/stats` reports backend `redis` when AppState is constructed with Redis selected.
  - `/v1/retrieve/stats` does not leak Redis URL.
- Integration/smoke:
  - Real Redis service available through `HEADROOM_TEST_REDIS_URL` or compose/service test.
  - Start Redis-backed runtime, write/read a CCR entry through the selected backend.
  - Start image with Redis env and verify stats endpoint reports `redis`.

## Validation Plan

Todo docs-only validation:

- Format/check markdown with repo-compatible formatter if available.
- `git diff --check origin/main...HEAD`.
- Diff scope limited to `specs/HO-2132-enable-rust-redis-ccr-backend-via-env/**`.

Later implementation validation:

- `cargo fmt --check`.
- `cargo check -p headroom-proxy --features redis` or equivalent workspace feature command selected by repo policy.
- `cargo test -p headroom-proxy` focused config/stats tests.
- `cargo test -p headroom-core --features redis ccr_backends` with Redis service for Redis tests.
- Docker/image smoke against Redis service.
- `/v1/retrieve/stats` readback showing backend `redis`.

## Rollout Notes For dev-infra

- Redis must be provisioned with persistence appropriate for CCR originals; memory-only Redis is not production-safe for reversible retrieval.
- Redis auth/TLS/secrets must be handled outside this repo before production switch.
- Resource sizing must account for TTL, payload sizes, tenant prefix cardinality, and eviction policy.
- Rollback must support switching `HEADROOM_CCR_BACKEND` back to the previous backend and preserving Redis data until data-disposal approval.
- Health/readiness must fail Redis startup/init failures instead of serving with an unintended backend.

## Out-of-scope Guardrails

- No production switch in Todo.
- No Redis deployment manifests in this repo.
- No public Redis endpoint.
- No Python behavior rewrite unless required for compatibility and recorded before implementation.
- No silent fallback to memory under Redis selection.

## Implementation Gates

- Gate 1: live Linear state must be implementation-eligible before Rust/Docker/workflow changes.
- Gate 2: Redis feature wiring must be chosen before runtime config claims Redis support.
- Gate 3: startup failure paths must be tested before image smoke.
- Gate 4: real Redis roundtrip must pass before claiming acceptance.
- Gate 5: `/v1/retrieve/stats` Redis backend readback must pass before Waiting Merge evidence.
