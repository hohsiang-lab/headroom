# Research: Wire Rust OpenAI Compression Paths To CCR Store

## Repo Reality Findings

- Decision: Base implementation on `hohsiang-lab/headroom` fork main at
  `bc12ace`, not the stale upstream-facing local base.
  Rationale: Linear HO-2130 names `hohsiang-lab/headroom` as the fork for
  Headroom changes; GitHub readback confirmed the fork exists and default branch
  is `main`.
  Alternatives considered: Working from local `origin` was rejected because it
  still points at `chopratejas/headroom`.

- Decision: Wire a shared CCR store into `AppState` and pass it from proxy
  OpenAI handlers into core live-zone dispatch.
  Rationale: Current `AppState` owns shared proxy runtime state in
  `crates/headroom-proxy/src/proxy.rs`; OpenAI chat and responses compression
  call `compress_openai_chat_live_zone` and
  `compress_openai_responses_live_zone` without any CCR store parameter.
  Alternatives considered: Per-request backend initialization was rejected
  because CCR backends are startup/runtime state, not request-local work.

- Decision: Reuse `headroom_core::ccr::backends::{CcrBackendConfig, from_config}`
  for initialization.
  Rationale: `from_config` already documents and implements loud init failure
  behavior and supports in-memory, SQLite, and Redis-feature-gated selection.
  Alternatives considered: Adding a proxy-specific store factory was rejected as
  duplicate backend logic.

## Dependency / Docs Checks

- Decision: Keep Axum state wiring as cloneable application state.
  Rationale: Context7 for `/tokio-rs/axum` documents `Router::with_state` and
  `State<AppState>` as the standard global state pattern. This matches the
  repo's existing `build_app(state).with_state(state)` shape.
  Alternatives considered: `Extension` middleware was unnecessary because the
  repo already uses typed `State<AppState>`.

- Decision: Add CLI/env-facing CCR config through the existing `clap` derive
  pattern.
  Rationale: Context7 for clap confirms `ValueEnum`, `default_value_t`, and
  `env = "..."` attributes are supported with the derive/env features; the
  proxy already uses that style in `config.rs`.
  Alternatives considered: Manual env parsing was rejected because it would
  drift from current `CliArgs` conventions.

- Decision: Start with SQLite and in-memory tests; leave Redis runtime rollout
  as follow-up.
  Rationale: `headroom-core` already includes `rusqlite` with bundled SQLite,
  WAL setup in the SQLite backend, and optional Redis behind a crate feature.
  Context7 for rusqlite confirms `Connection::open` and `PRAGMA journal_mode =
  WAL` patterns match the existing backend.
  Alternatives considered: Redis-first rollout was rejected by the issue scope
  and because feature-gated unsupported backend behavior needs separate runtime
  rollout proof.

## GitHub / Real-World Search

- grep.app API returned `503 Service Temporarily Unavailable` during plan
  review, so it was not counted as successful evidence.
- GitHub code search for `from_config CcrBackendConfig language:Rust` found the
  current Headroom/Copium backend factory pattern and tests; no better external
  pattern superseded the local API.
- GitHub code search for `Arc<dyn CcrStore>` and `AppState Arc<dyn ... axum`
  did not find a more specific reusable pattern than the repo-local `AppState`
  shape and Axum docs.

## Open Questions Resolved

- Repo ownership: `hohsiang-lab/headroom`.
- Layer boundary: Rust crates only; no Python production image changes.
- Backend rollout: SQLite/in-memory first; Redis only as feature-compatible
  follow-up unless explicitly requested later.
- Verification method: Rust unit/integration tests plus local Rust proxy smoke.
