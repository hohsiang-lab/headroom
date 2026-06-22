# Scope Confirmation: HO-2131

**Issue**: `[Headroom] Add Rust proxy /v1/retrieve and /v1/retrieve/stats`
**State**: Todo
**Confirmation Date**: 2026-06-22

## Confirmed Scope

- Add Rust proxy local `POST /v1/retrieve`.
- Add Rust proxy local `GET /v1/retrieve/stats`.
- Back endpoints with existing Rust CCR store semantics.
- Mount endpoints before catch-all forwarding.
- Return clear 400/404/200 behavior.
- Add focused Rust tests proving local retrieval and stats.
- Add bounded logs/metrics without hash label cardinality.

## Explicit Non-Scope

- No Rust implementation in Todo state.
- No Python proxy changes.
- No Python full parity for `/v1/retrieve/{hash}`.
- No `/v1/retrieve/tool_call` provider formatting.
- No BM25/search filtering unless Rust CCR already supports it at implementation time.
- No compression algorithm or marker format change.
- No OpenClaw/Gateway routing change.
- No CI monitoring in this orchestration scene.

## Repo Evidence Used

- `crates/headroom-proxy/src/proxy.rs` owns axum router construction and catch-all forwarding.
- `crates/headroom-core/src/ccr/mod.rs` defines `CcrStore` with `put`, `get`, and `len`.
- `headroom/proxy/server.py` has Python reference endpoints for `POST /v1/retrieve` and `GET /v1/retrieve/stats`.
- Existing Rust observability is Prometheus/structured-log based, with bounded-label discipline.

## Scope Decision

Proceed to implementation only after owner/issue workflow advances beyond Todo. The implementation agent should build the minimum Rust-native retrieval/status surface above, not rebuild the broader Python CCR feedback/search/tool-call system.
