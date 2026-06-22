# Tasks: Rust Proxy CCR Retrieve Endpoints

**Input**: Design documents `/specs/HO-2131-add-rust-proxy-v1-retrieve-and-v1-retrieve-stats/`
**Prerequisites**: `spec.md`, `plan.md`, `contracts/retrieve-endpoints.openapi.yaml`
**Tests**: Required for endpoint behavior and no-upstream forwarding.

## Phase 1: Setup

- [X] T001 Read `spec.md`, `plan.md`, and `contracts/retrieve-endpoints.openapi.yaml`.
- [X] T002 Confirm current Rust proxy router shape in `crates/headroom-proxy/src/proxy.rs`.
- [X] T003 Confirm current CCR store trait/backends in `crates/headroom-core/src/ccr/`.

## Phase 2: Foundational Store And Observability

- [X] T004 Decide and document the minimal store state shape needed in `AppState` for handlers.
- [X] T005 Add or expose a CCR store handle for Rust proxy handlers.
- [X] T006 Add a bounded proxy-side retrieve stats structure for success/miss/invalid/stats counters.
- [X] T007 Add bounded structured logs for retrieve success, retrieve miss, invalid request, and stats read.
- [X] T008 Add bounded Prometheus metrics only if they use static labels; never label by hash.

## Phase 3: User Story 1 - Retrieve Stored CCR Content (P1)

**Goal**: `POST /v1/retrieve` returns locally stored CCR payloads without upstream forwarding.

**Independent Test**: Seed store, call endpoint, assert response body and upstream untouched.

- [X] T009 [US1] Add request/response structs for `POST /v1/retrieve`.
- [X] T010 [US1] Validate `hash` presence, type, and canonical hash format.
- [X] T011 [US1] Implement successful lookup returning `hash` and `original_content`.
- [X] T012 [US1] Implement missing/expired lookup returning HTTP 404.
- [X] T013 [US1] Mount `POST /v1/retrieve` before catch-all in `crates/headroom-proxy/src/proxy.rs`.
- [X] T014 [US1] Add focused tests for success, missing hash, invalid hash, malformed JSON, and no-upstream forwarding.

## Phase 4: User Story 2 - Inspect CCR Store Status (P2)

**Goal**: `GET /v1/retrieve/stats` returns local CCR store status and retrieve counters.

**Independent Test**: Call stats on empty store, then after seeded/retrieved entry, and assert counters changed.

- [X] T015 [US2] Add response struct for `GET /v1/retrieve/stats`.
- [X] T016 [US2] Return `store.entry_count` from existing CCR store capabilities.
- [X] T017 [US2] Return retrieve counters for success, miss, invalid request, and stats calls.
- [X] T018 [US2] Represent unsupported backend-specific fields explicitly as `null` or omitted by contract.
- [X] T019 [US2] Mount `GET /v1/retrieve/stats` before catch-all in `crates/headroom-proxy/src/proxy.rs`.
- [X] T020 [US2] Add focused tests for empty stats, populated stats, and counter updates after retrieval.

## Phase 5: Polish And Verification

- [X] T021 Run `cargo fmt --check`.
- [X] T022 Run focused Rust tests for `headroom-core` CCR and `headroom-proxy` retrieve handlers.
- [X] T023 Run a route audit proving `/v1/retrieve` and `/v1/retrieve/stats` are not forwarded upstream.
- [X] T024 Update docs only if implementation changes public operator behavior beyond these endpoint contracts.

## Dependencies

- T001-T003 before implementation.
- T004-T008 block both user stories.
- US1 can ship without US2 only if stats endpoint is explicitly deferred, but HO-2131 scope includes both endpoints.
- T021-T024 after both user stories.

## Notes

- Do not implement Python parity endpoints outside this issue scope.
- Do not add `GET /v1/retrieve/{hash}` or `/v1/retrieve/tool_call` unless a later scope explicitly expands.
- Do not use hash values as metric labels.
- Do not silently create an in-memory store when configured persistent CCR backend fails.

## Implementation Evidence

- `cargo fmt --all --check` passed.
- `cargo check -p headroom-proxy` passed.
- `cargo clippy -p headroom-proxy --all-targets -- -D warnings` passed.
- `cargo test -q -p headroom-proxy --test integration_retrieve` passed: 4 tests.
- `cargo test -q -p headroom-core ccr` passed focused CCR coverage.
- Route audit is covered by `integration_retrieve`: retrieve and stats tests use unreachable upstream `http://127.0.0.1:9`, so any catch-all forwarding would fail.
