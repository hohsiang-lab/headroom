# HO-2132 Analyze - Enable Rust Redis CCR backend via env

## Speckit Phase Availability

- `speckit-*` phase skills were not present in this Codex session skill list.
- Repo did not expose a `.specify/templates/spec-template.md` path during discovery.
- Fallback used existing project SpecKit artifact shape under `specs/<issue-slug>/` and kept changes docs-only because live issue state is Todo.

## Repo Reality Findings

- `crates/headroom-core/src/ccr/backends/redis.rs` implements `RedisCcrStore` behind `cfg(feature = "redis")`.
- `RedisCcrStore::open` is intended to validate Redis during open; implementation phase must preserve or extend that PING/readiness behavior.
- `crates/headroom-core/src/ccr/backends/mod.rs` exposes `CcrBackendConfig::Redis { url, ttl_seconds, key_prefix }`.
- `from_config` already returns an unsupported-backend error when Redis is selected without the Redis feature.
- `crates/headroom-core/tests/ccr_backends.rs` already contains Redis cfg-gated roundtrip tests gated by `HEADROOM_TEST_REDIS_URL`.
- `crates/headroom-proxy/Cargo.toml` currently depends on `headroom-core` without enabling the Redis feature.
- `Dockerfile` currently builds the Rust proxy via `cargo build --release --locked --bin headroom-proxy` without `--features redis`.
- `crates/headroom-proxy/src/config.rs` is the correct env/CLI parsing surface for Rust proxy config.
- `crates/headroom-proxy/src/proxy.rs` is the correct startup/app-state/route surface for CCR backend initialization and stats endpoint wiring.
- Existing Rust proxy routes found in `build_app` do not include `/v1/retrieve/stats`.

## Risk Classification

- Fatal: 0
- Critical: 0
- Minor: 4

## Minor Findings

- M001: Redis backend exists in core, but runtime image support is not guaranteed until proxy/package/Docker feature wiring is explicit.
- M002: `/v1/retrieve/stats` acceptance requires a Rust proxy route or handler that is not currently visible in the route table.
- M003: Real Redis integration must remain opt-in for normal test runs or CI must provide a Redis service; otherwise existing deterministic tests could become flaky.
- M004: Redis URL must be redacted from stats/log evidence; tenant prefix may be safe, but implementation should document exposure choice.

## Missing Parts

- Exact image smoke surface is not yet selected: existing GitHub Actions workflow, Docker Compose smoke, or dev-infra image workflow.
- Empty `HEADROOM_CCR_TENANT_PREFIX` behavior needs implementation decision and tests.
- dev-infra Redis provisioning details belong outside this repo but must be documented before production switch.

## Scope Verdict

Scope is implementable in `hohsiang-lab/headroom` after the issue moves out of Todo. Todo work must stop at docs-only scope confirmation.
