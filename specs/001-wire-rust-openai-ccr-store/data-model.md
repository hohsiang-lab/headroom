# Data Model: Rust Proxy CCR Store Wiring

## CCR Backend Runtime Config

- **Purpose**: Operator-facing runtime selection for the Rust proxy CCR backend.
- **Fields**:
  - `backend`: `in_memory`, `sqlite`, or `redis`.
  - `sqlite_path`: file path used when backend is SQLite.
  - `ttl_seconds`: retention window for stored originals.
  - `capacity`: in-memory capacity for tests/local ephemeral runs.
  - `redis_url`: Redis URL for future feature-gated rollout.
  - `redis_key_prefix`: optional Redis key prefix for multi-tenant rollout.
- **Validation**:
  - SQLite backend requires an openable path and successful schema setup.
  - Redis backend must fail loudly when the feature is not compiled or PING fails.
  - No backend selection may silently fall back to another backend.

## Proxy CCR Store State

- **Purpose**: Shared cloneable state held by the Rust proxy and passed to
  compression paths.
- **Fields**:
  - `store`: initialized `Arc<dyn CcrStore>`.
  - `backend_name`: log/debug label for startup evidence.
- **Validation**:
  - Constructed only after `from_config` succeeds.
  - Request handlers never initialize backend state themselves.

## OpenAI Compression Invocation

- **Purpose**: Per-request call boundary from proxy compression modules into
  core live-zone dispatch.
- **Fields**:
  - `body`: original buffered request bytes.
  - `mode`: compression mode.
  - `auth_mode`: classified auth mode.
  - `request_id`: structured log correlation ID.
  - `ccr_store`: optional store reference, present when proxy CCR init succeeded.
- **Validation**:
  - Store is used only when compression is accepted.
  - No-compression, passthrough, and normalization-only paths do not emit CCR
    markers.

## CCR Marker Contract

- **Purpose**: Link compressed text to recoverable original content.
- **Fields**:
  - `hash`: marker hash emitted in `<<ccr:HASH>>`.
  - `original`: original pre-compression block content.
  - `ttl_seconds`: retention window inherited from backend config.
- **Validation**:
  - Marker hash must resolve to the original content in tests.
  - Marker emission and store write must be coupled.
