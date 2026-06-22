# HO-2132 Tasks - Enable Rust Redis CCR backend via env

## Todo Scope Tasks

- [x] T001 Read Linear HO-2132 scope, acceptance evidence, and Todo no-code instruction.
- [x] T002 Read Headroom repo shape and current Rust CCR backend implementation.
- [x] T003 Confirm Redis feature gate and existing `CcrBackendConfig::Redis` behavior.
- [x] T004 Confirm Rust proxy config/startup/route surfaces.
- [x] T005 Confirm Docker image build currently does not enable Redis feature.
- [x] T006 Create `spec.md` with functional requirements, scenarios, edge cases, acceptance criteria, non-goals, and stop conditions.
- [x] T007 Create `plan.md` with implementation shape, tests, validation, rollout notes, and gates.
- [x] T008 Create `analyze.md` with repo reality, risk classification, and fatal/critical findings.
- [x] T009 Create `scope-confirmation.md` with assignee-visible scope verdict.
- [x] T010 Run docs-only validation.
- [x] T011 Confirm diff limited to `specs/HO-2132-enable-rust-redis-ccr-backend-via-env/**`.
- [x] T012 Commit docs-only SpecKit artifacts.
- [x] T013 Push branch and open/update draft PR.
- [x] T014 Post Todo scope report to Discord issue thread.
- [x] T015 Run Todo -> Waiting guard only after PR/thread evidence exists.

## Later Implementation Tasks

- [x] T100 Re-read live Linear state; do not touch implementation files unless state is implementation-eligible.
- [x] T101 Add Rust proxy CCR config parsing for `HEADROOM_CCR_BACKEND`, `HEADROOM_REDIS_URL`, and `HEADROOM_CCR_TENANT_PREFIX`.
- [x] T102 Convert proxy config into `headroom_core::ccr::backends::CcrBackendConfig`.
- [x] T103 Initialize selected CCR backend at startup and propagate init errors so startup fails loud.
- [x] T104 Ensure Redis selection cannot silently fall back to memory or SQLite.
- [x] T105 Enable `headroom-core/redis` for the Rust proxy production build path where Redis runtime support is required.
- [x] T106 Add `/v1/retrieve/stats` route or equivalent Rust proxy handler that reports selected CCR backend.
- [x] T107 Ensure stats response redacts Redis URL and only exposes safe metadata.
- [x] T108 Add focused Rust tests for Redis backend env parsing and prefix behavior.
- [x] T109 Add focused Rust tests for backend selection and loud error cases.
- [x] T110 Add stats endpoint test proving backend `redis` is reported when enabled.
- [x] T111 Add real Redis integration/smoke roundtrip using `HEADROOM_TEST_REDIS_URL` or service-backed test harness.
- [x] T112 Update image smoke to prove Redis-capable runtime against a Redis service.
- [x] T113 Document dev-infra Redis persistence/resource/auth prerequisites before production switch.
- [x] T114 Run Rust fmt/check/tests selected by repo policy.
- [x] T115 Run Redis real-service smoke and record evidence.
- [x] T116 Run image smoke and record `/v1/retrieve/stats` Redis readback.

## Stop Conditions

- [x] S001 Stop if live Linear state is Todo and any implementation code would be required.
- [x] S002 Stop if Redis-selected runtime would compile without the `redis` feature.
- [x] S003 Stop if Redis init failure can fall back to memory or SQLite.
- [x] S004 Stop if `/v1/retrieve/stats` cannot truthfully report the selected backend.
- [x] S005 Stop if real Redis smoke is unavailable; do not claim Redis-backed CCR roundtrip.
- [x] S006 Stop if dev-infra production switch prerequisites are missing.

## Implementation Evidence

- `cargo fmt --check` passed.
- `cargo check -p headroom-proxy` passed.
- `cargo clippy -p headroom-proxy --all-targets -- -D warnings` passed.
- `cargo test -p headroom-proxy ccr_env -- --nocapture` passed 2 tests.
- `cargo test -p headroom-proxy --test integration_health -- --nocapture` passed 5 tests.
- `cargo test -p headroom-proxy --test integration_retrieve -- --nocapture` passed 5 tests.
- With Redis service `HEADROOM_TEST_REDIS_URL=redis://127.0.0.1:41935`, `cargo test -p headroom-core --features redis --test ccr_backends -- --nocapture` passed 8 tests.
- With the same Redis service, `cargo test -p headroom-proxy --test integration_retrieve redis_backend_roundtrip_and_stats_when_test_redis_configured -- --nocapture` passed 1 test.
- `CONTAINER_CLI=podman IMAGE=ho2132-headroom-redis-ccr-smoke:local scripts/smoke_redis_ccr_image.sh` passed and read `/v1/retrieve/stats` with `"backend":"redis"`.
- `git diff --check hohsiang/main...HEAD` passed.
- Root `package.json` is absent, so repo-standard Node lint gate is not applicable to this Rust/Docker/docs change.
