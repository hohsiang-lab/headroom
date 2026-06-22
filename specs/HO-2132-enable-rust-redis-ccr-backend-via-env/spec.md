# HO-2132 Spec - Enable Rust Redis CCR backend via env

## Goal

Enable the Rust Headroom proxy runtime to select the Redis CCR backend through environment variables so multi-host or horizontally scaled deployments can share reversible CCR entries without sticky-session dependence.

## Repo / Issue Alignment

- Linear issue: `HO-2132`
- Title: `[Headroom] Enable Rust Redis CCR backend via env`
- Target repo: `hohsiang-lab/headroom`
- Target runtime: Rust `headroom-proxy` binary and container image smoke path
- Current Todo PR scope: SpecKit docs only. No Rust, Python, Docker, workflow, or runtime config changes in Todo state.

## Current Reality To Respect

- `crates/headroom-core/src/ccr/backends/redis.rs` already implements `RedisCcrStore` behind the `redis` Cargo feature.
- `crates/headroom-core/src/ccr/backends/mod.rs` already exposes `CcrBackendConfig::Redis` and `from_config`, and documents that backend initialization errors must surface instead of silently falling back.
- `crates/headroom-core/tests/ccr_backends.rs` already has cfg-gated Redis roundtrip coverage using `HEADROOM_TEST_REDIS_URL`.
- `crates/headroom-proxy` currently depends on `headroom-core` without enabling the `redis` feature, so a Redis backend selected at runtime would be unsupported unless feature wiring is added.
- `crates/headroom-proxy/src/config.rs` owns CLI/env parsing for the Rust proxy and should own CCR env parsing rather than ad hoc reads from handlers.
- `crates/headroom-proxy/src/proxy.rs` owns `AppState` construction and Axum routes; CCR backend initialization should happen during startup so Redis URL/PING failures fail loud before serving traffic.
- The current Rust proxy route set does not expose `/v1/retrieve/stats`; acceptance requires that stats report the selected Redis backend when enabled.
- `Dockerfile` builds `cargo build --release --locked --bin headroom-proxy` without `--features redis`, so Redis-capable runtime images require build feature adjustment and image smoke validation.

## Functional Requirements

- FR-001: The Rust proxy MUST support `HEADROOM_CCR_BACKEND=redis` as an environment-driven backend selector.
- FR-002: The Rust proxy MUST read `HEADROOM_REDIS_URL` when Redis CCR is selected.
- FR-003: The Rust proxy MUST read optional `HEADROOM_CCR_TENANT_PREFIX` and pass it as the Redis key prefix.
- FR-004: Selecting Redis without a Redis URL MUST fail startup with an operator-visible error.
- FR-005: Redis connection open and PING failure MUST fail startup; the proxy MUST NOT silently fall back to memory or SQLite.
- FR-006: Redis selection in a build without the `redis` feature MUST fail loud with an unsupported-backend error.
- FR-007: The Rust proxy package/image path that production uses MUST compile `headroom-core` with the Redis feature where Redis runtime support is expected.
- FR-008: `/v1/retrieve/stats` MUST report the effective CCR backend name, and must report `redis` when Redis is enabled.
- FR-009: Redis URL and tenant prefix parsing/selection MUST have focused Rust unit tests.
- FR-010: Redis CCR storage MUST have a real Redis roundtrip smoke/integration test that writes a CCR entry and retrieves it through the selected backend.
- FR-011: Image smoke MUST verify the published runtime can start with Redis CCR enabled against a Redis service and expose Redis backend stats.
- FR-012: Documentation or rollout evidence MUST state Redis persistence/resource prerequisites before production switch.

## Scenarios

### Scenario 1 - Redis backend selected successfully

Given the Rust proxy image was built with Redis CCR support
And `HEADROOM_CCR_BACKEND=redis`
And `HEADROOM_REDIS_URL=redis://redis:6379`
And `HEADROOM_CCR_TENANT_PREFIX=headroom-prod`
When the proxy starts
Then startup initializes Redis CCR using the configured URL and prefix
And `/v1/retrieve/stats` reports backend `redis`.

### Scenario 2 - Redis init failure is loud

Given `HEADROOM_CCR_BACKEND=redis`
And `HEADROOM_REDIS_URL` points to an unavailable Redis service
When the proxy starts
Then startup exits with a non-zero error
And no in-memory CCR fallback is installed.

### Scenario 3 - Default backend behavior remains bounded

Given `HEADROOM_CCR_BACKEND` is unset
When the Rust proxy starts
Then existing non-Redis behavior remains unchanged unless implementation explicitly updates the default with documented migration evidence.

### Scenario 4 - Runtime image validation

Given a container image intended for dev-infra is built
And a Redis service is reachable in smoke
When the image starts with Redis CCR env
Then smoke proves a Redis-backed CCR roundtrip and Redis backend stats before production rollout.

## Edge Cases

- `HEADROOM_CCR_BACKEND=redis` with empty `HEADROOM_REDIS_URL`.
- Redis URL malformed.
- Redis reachable for TCP but PING fails or auth fails.
- Empty `HEADROOM_CCR_TENANT_PREFIX`; implementation must decide whether to reject empty or treat it as unset and test that choice.
- Redis feature disabled at compile time but Redis selected at runtime.
- Existing SQLite/in-memory tests must remain deterministic and not require Redis unless Redis-specific test env is set.
- `/v1/retrieve/stats` must not claim Redis when Redis init failed or Redis was not selected.

## Out Of Scope

- Do not deploy or provision Redis in this repo.
- Do not change dev-infra manifests in this issue; dev-infra rollout belongs to its own infra change.
- Do not switch production to Redis automatically.
- Do not add public Redis exposure.
- Do not remove existing SQLite or in-memory CCR backends.
- Do not change Python CCR backend behavior unless Rust proxy interop requires a documented compatibility shim.

## Acceptance Criteria

- AC-001: Focused Rust tests pass for Redis URL parsing, tenant prefix parsing, and backend selection.
- AC-002: Redis-selected startup fails loud on missing URL, malformed URL, unsupported feature, or failed Redis PING.
- AC-003: Redis-backed CCR roundtrip works against a real Redis service in smoke/integration.
- AC-004: `/v1/retrieve/stats` reports Redis backend when enabled.
- AC-005: Runtime image smoke proves Redis-capable Headroom image behavior against Redis.
- AC-006: dev-infra rollout notes document Redis persistence, memory/resource sizing, auth/TLS/secrets, and rollback prerequisites before production switch.

## Open Questions

- OQ-001: Should the default Rust proxy backend remain the current effective behavior, or should implementation explicitly align Rust proxy default to SQLite in a later scope?
- OQ-002: Should `HEADROOM_CCR_TENANT_PREFIX=""` be rejected or normalized to unset?
- OQ-003: Should Redis stats include only backend metadata, or also Redis key-prefix/TTL fields with URL redaction?
- OQ-004: Which CI/image smoke surface owns the Redis service: existing GitHub Actions workflow, Docker Compose smoke, or dev-infra image workflow?

## Stop Conditions

- Stop if implementation state has not moved beyond Todo before touching Rust, Docker, workflow, or runtime config files.
- Stop if Redis feature wiring requires upstream package/release policy changes outside `hohsiang-lab/headroom`.
- Stop if a Redis-backed real-service roundtrip cannot be run or delegated as acceptance evidence.
- Stop if `/v1/retrieve/stats` cannot be made operator-visible without inventing an incompatible API contract.
