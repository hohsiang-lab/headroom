# Implementation Plan: Wire Rust OpenAI Compression Paths To CCR Store

**Branch**: `001-wire-rust-openai-ccr-store` | **Date**: 2026-06-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-wire-rust-openai-ccr-store/spec.md`

## Summary

Initialize a CCR store in the Rust proxy using the existing
`CcrBackendConfig` / `from_config` API, hold the initialized store in shared
proxy state, and thread it into OpenAI Chat Completions and Responses
live-zone compression so emitted `<<ccr:...>>` markers are backed by storage.
SQLite/in-memory coverage comes first; Redis rollout and Python production
image changes stay out of scope.

## Technical Context

**Language/Version**: Rust 2021, workspace rust-version 1.80
**Primary Dependencies**: `axum` 0.7, `clap` 4, `reqwest` 0.12,
`headroom-core`, `rusqlite` 0.32 bundled SQLite
**Storage**: CCR store via `headroom_core::ccr::backends`; SQLite default for
single-worker persistent storage, in-memory for tests
**Testing**: `cargo test -p headroom-core`, `cargo test -p headroom-proxy`
focused integration tests, local proxy smoke with wiremock/upstream or local
test upstream
**Target Platform**: Rust proxy Linux/server process
**Project Type**: Rust workspace with `crates/headroom-core` and
`crates/headroom-proxy`
**Performance Goals**: No per-request backend initialization; no additional
body reserialization outside existing byte-range surgery
**Constraints**: No silent backend fallback; no Python production image changes;
preserve existing passthrough/cache-hot byte semantics
**Scale/Scope**: One Rust proxy process with SQLite/in-memory first; Redis
feature stays planned but not rolled out

## Constitution Check

- Repo Reality First: PASS. Plan is based on fork main `bc12ace` and concrete
  files under `crates/headroom-core` / `crates/headroom-proxy`.
- Reversible Compression Contract: PASS. Primary change is store-backed marker
  emission for OpenAI paths.
- Rust Proxy Boundary: PASS. No Python production image or Docker production
  entrypoint work is planned.
- Evidence Before State Movement: PASS. Tests and smoke evidence are defined
  before implementation.
- No Silent Operational Degradation: PASS. Backend init failure is a required
  test and startup/app-state failure path.

## Project Structure

### Documentation (this feature)

```text
specs/001-wire-rust-openai-ccr-store/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── analyze.md
├── contracts/
│   └── rust-proxy-ccr-store.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/headroom-core/
├── src/ccr/backends/mod.rs
├── src/transforms/live_zone.rs
└── tests/

crates/headroom-proxy/
├── src/config.rs
├── src/main.rs
├── src/proxy.rs
├── src/compression/live_zone_openai.rs
├── src/compression/live_zone_responses.rs
└── tests/
    ├── common/mod.rs
    ├── integration_chat_completions.rs
    └── integration_responses.rs
```

**Structure Decision**: Modify the existing Rust proxy/core crates in place.
`headroom-core` owns CCR traits/backend factories and live-zone dispatcher
store-marker behavior. `headroom-proxy` owns operator config, app-state
initialization, request-path threading, and integration smoke tests.

## Phase 0: Research

See [research.md](./research.md). All implementation-detail questions are
resolved from repo reality, Context7 docs, GitHub code search fallback, and web
search. No owner clarification is required before tasks.

## Phase 1: Design & Contracts

See [data-model.md](./data-model.md) and
[contracts/rust-proxy-ccr-store.md](./contracts/rust-proxy-ccr-store.md).

### Design Notes

- Add proxy config fields that map CLI/env values into
  `CcrBackendConfig`. Keep defaults explicit and document backend/TTL/path.
- Initialize the backend once in `AppState::new` or an adjacent constructor
  path. Convert `Box<dyn CcrStore>` from `from_config` into `Arc<dyn CcrStore>`
  for cloneable shared state.
- Add OpenAI core dispatcher variants or signature extension that accepts
  `Option<&dyn CcrStore>`, matching the Anthropic `with_ccr` pattern and keeping
  compatibility shims where useful.
- Pass `state.ccr_store.as_deref()` through `compress_openai_chat_request` and
  `compress_openai_responses_request`.
- Keep marker/store writes gated by accepted compression only; no marker for
  normalization-only changes or no-compression outcomes.

## Plan Review

> Plan reviewed via Context7 / GitHub code search fallback / web search.
> grep.app API returned 503 during review; GitHub code search was used as the
> code-search fallback. No revisions needed beyond explicitly favoring repo-local
> `AppState` + `CcrBackendConfig::from_config` patterns.

## Complexity Tracking

No constitution violations.
