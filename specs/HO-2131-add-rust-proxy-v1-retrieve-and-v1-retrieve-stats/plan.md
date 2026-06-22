# Implementation Plan: Rust Proxy CCR Retrieve Endpoints

**Branch**: `HO-2131-add-rust-proxy-v1-retrieve-and-v1-retrieve-stats` | **Date**: 2026-06-22 | **Spec**: `spec.md`
**Input**: Feature specification `/specs/HO-2131-add-rust-proxy-v1-retrieve-and-v1-retrieve-stats/spec.md`

## Summary

Add first-class Rust proxy endpoints for CCR retrieval: `POST /v1/retrieve` and `GET /v1/retrieve/stats`. The endpoints should be local axum routes mounted before catch-all proxy forwarding, backed by the existing `headroom_core::ccr::CcrStore` abstraction, and observable through structured logs plus bounded metrics/counters.

## Technical Context

**Language/Version**: Rust workspace; exact Rust toolchain follows repo configuration.
**Primary Dependencies**: axum 0.7, reqwest, serde/serde_json, prometheus, tracing, headroom-core CCR modules.
**Storage**: Existing `headroom_core::ccr` backends: in-memory, SQLite, optional Redis.
**Testing**: Cargo unit/integration tests in `crates/headroom-proxy` and `crates/headroom-core`.
**Target Platform**: Headroom Rust proxy binary.
**Project Type**: Rust reverse proxy crate inside a multi-crate workspace.
**Performance Goals**: Local retrieval should not contact upstream; successful lookup should be bounded by configured backend lookup cost.
**Constraints**: Do not leak hash values into unbounded metric labels; do not silently fall back to in-memory backend when configured backend fails; preserve catch-all passthrough behavior for unrelated routes.
**Scale/Scope**: Two local HTTP endpoints plus store/stat plumbing and focused tests.

## Constitution Check

- [x] No GraphQL schema changes.
- [x] No Hasura migration or permissions changes.
- [x] No frontend cache/revalidation changes.
- [x] No payment/order/state-machine behavior.
- [x] Rust proxy changes require focused automated tests and no silent fallback behavior.

## Project Structure

### Documentation

```text
specs/HO-2131-add-rust-proxy-v1-retrieve-and-v1-retrieve-stats/
├── spec.md
├── plan.md
├── tasks.md
├── scope-confirmation.md
└── contracts/
    └── retrieve-endpoints.openapi.yaml
```

### Source Code

```text
crates/headroom-core/src/ccr/
├── mod.rs
└── backends/

crates/headroom-proxy/src/
├── proxy.rs
├── handlers/
├── observability/
└── config.rs

crates/headroom-proxy/tests/
```

**Structure Decision**: Implement endpoint handlers under `crates/headroom-proxy/src/handlers/` and mount them in `proxy.rs` before fallback routing. Extend `AppState` only if the proxy does not already hold a CCR store handle at implementation time.

## Research Notes

- Existing Python proxy exposes `POST /v1/retrieve` and `GET /v1/retrieve/stats`; Rust implementation should match the minimum wire contract needed by local CCR retrieval, not Python's full feedback/search ecosystem.
- Rust `CcrStore` currently exposes `put`, `get`, and `len`; exact stats beyond entry count require a small proxy-side stats/readback layer or explicit unsupported/null fields.
- Existing Rust proxy already mounts explicit axum routes in `proxy.rs` before fallback, including `/healthz`, `/metrics`, `/v1/chat/completions`, `/v1/responses`, Vertex, Bedrock, and conversations routes.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| Add proxy-side retrieve stats layer | `CcrStore::len()` alone cannot prove successful local retrievals | Returning only `entry_count` would repeat the live failure mode where stats stayed empty/ambiguous |

## Implementation Phases

### Phase 0: Design Lock

- Confirm the endpoint contract in `contracts/retrieve-endpoints.openapi.yaml`.
- Confirm whether `AppState` should own `Arc<dyn CcrStore>` or a typed store wrapper with stats.
- Confirm exact response shape for unsupported query search.

### Phase 1: Store/State Plumbing

- Wire configured CCR backend into Rust proxy startup.
- Expose store handle to handlers through `AppState`.
- Add bounded retrieve counters/recent events without unbounded labels.

### Phase 2: Endpoint Handlers

- Add `POST /v1/retrieve`.
- Add `GET /v1/retrieve/stats`.
- Ensure both routes bypass upstream forwarding.

### Phase 3: Verification

- Add focused unit/integration tests for success, miss, invalid request, stats, and no-upstream forwarding.
- Run Rust formatting and focused test suite.
