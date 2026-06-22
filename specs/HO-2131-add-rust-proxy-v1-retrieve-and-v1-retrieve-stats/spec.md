# Feature Specification: Rust Proxy CCR Retrieve Endpoints

**Feature Branch**: `HO-2131-add-rust-proxy-v1-retrieve-and-v1-retrieve-stats`
**Created**: 2026-06-22
**Status**: Draft
**Input**: Linear HO-2131: "[Headroom] Add Rust proxy /v1/retrieve and /v1/retrieve/stats"

## User Scenarios & Testing

### User Story 1 - Retrieve Stored CCR Content (Priority: P1)

An agent runtime or operator can call the Rust proxy `POST /v1/retrieve` with a CCR hash emitted in compressed content and receive the original stored payload without forwarding the request upstream.

**Why priority**: CCR markers are only useful if the Rust proxy can resolve them locally after compression.

**Independent Test**: Seed the Rust proxy CCR store with a known hash and payload, call `POST /v1/retrieve`, and verify the response returns that payload with HTTP 200.

**Acceptance Scenarios**:

1. **Given** a live CCR store entry for hash `abc123`, **When** a client posts `{"hash":"abc123"}` to `/v1/retrieve`, **Then** the proxy returns HTTP 200 with `hash` and `original_content`.
2. **Given** no live CCR store entry for hash `missing`, **When** a client posts `{"hash":"missing"}` to `/v1/retrieve`, **Then** the proxy returns HTTP 404 and does not contact upstream.
3. **Given** a request body without `hash`, **When** a client posts it to `/v1/retrieve`, **Then** the proxy returns HTTP 400 and does not contact upstream.

---

### User Story 2 - Inspect CCR Store Status (Priority: P2)

An operator can call the Rust proxy `GET /v1/retrieve/stats` and see whether CCR storage is populated and whether retrieve calls are succeeding.

**Why priority**: Recent live tests showed compression could work while retrieve stats stayed empty; operators need a direct Rust proxy health/readback surface.

**Independent Test**: Start the proxy with an empty store, call `/v1/retrieve/stats`, then seed/retrieve one entry and verify counters reflect store count and retrieval activity.

**Acceptance Scenarios**:

1. **Given** an empty CCR store, **When** a client calls `/v1/retrieve/stats`, **Then** the proxy returns HTTP 200 with `store.entry_count = 0`.
2. **Given** one stored CCR entry, **When** a client calls `/v1/retrieve/stats`, **Then** the proxy returns HTTP 200 with `store.entry_count >= 1`.
3. **Given** a successful `/v1/retrieve` call, **When** a client calls `/v1/retrieve/stats`, **Then** the response exposes retrieval counters sufficient to prove the Rust endpoint handled the request locally.

## Edge Cases

- Hash is missing, empty, not a string, or not canonical 24 lowercase hex.
- Request body is malformed JSON or larger than the existing proxy body limit.
- CCR backend is configured but unavailable at proxy startup.
- CCR entry expired between stats read and retrieve call.
- Redis backend cannot efficiently report exact live entry count.
- Retrieve endpoint receives an optional `query`; for this scope, search filtering is not implemented unless existing Rust CCR APIs already support it.

## Requirements

### Functional Requirements

- **FR-001**: Rust proxy MUST mount `POST /v1/retrieve` before the catch-all forwarder so retrieval requests never proxy upstream.
- **FR-002**: `POST /v1/retrieve` MUST accept JSON body `hash` and optional `query`.
- **FR-003**: `POST /v1/retrieve` MUST validate `hash` as a non-empty canonical CCR hash; invalid input returns HTTP 400.
- **FR-004**: `POST /v1/retrieve` MUST return HTTP 200 with the stored original payload when the hash is live in the CCR store.
- **FR-005**: `POST /v1/retrieve` MUST return HTTP 404 for missing or expired hashes.
- **FR-006**: Rust proxy MUST mount `GET /v1/retrieve/stats` before the catch-all forwarder.
- **FR-007**: `GET /v1/retrieve/stats` MUST return JSON containing current store status and retrieval counters.
- **FR-008**: Retrieve success, miss, invalid request, and stats calls MUST emit bounded structured logs.
- **FR-009**: Retrieve metrics/counters MUST use bounded labels only; hash values MUST NOT be Prometheus labels.
- **FR-010**: Implementation MUST reuse the existing Rust CCR store abstraction where possible and surface unsupported stats fields explicitly rather than silently lying.

### Entities

- **CCR Store Entry**: Existing Rust CCR payload keyed by hash.
  - Source: `headroom_core::ccr::CcrStore`
  - Key fields: `hash`, `payload`, live/expired state as supported by backend
  - Persistence: existing backend config only; no new database table in this scope

- **CCR Retrieve Event**: In-process operational event/counter for local retrieve calls.
  - Key fields: `hash`, `status`, `retrieval_type`, `items_retrieved`, `created_at`
  - Persistence: in-memory bounded recent events or metrics; durable audit storage is out of scope

## Success Criteria

### Measurable Outcomes

- **SC-001**: A local integration test proves `POST /v1/retrieve` returns a seeded payload from Rust proxy without upstream traffic.
- **SC-002**: A local integration test proves missing/invalid hashes return HTTP 404/400 respectively.
- **SC-003**: A local integration test proves `GET /v1/retrieve/stats` returns JSON with store entry count and retrieval counters.
- **SC-004**: `cargo test -p headroom-proxy retrieve` or equivalent focused Rust tests pass.
- **SC-005**: Existing passthrough routes continue to fall through catch-all unchanged.

## Out Of Scope

- Python proxy endpoint changes.
- `GET /v1/retrieve/{hash}` parity endpoint.
- `POST /v1/retrieve/tool_call` provider-specific formatting.
- BM25/search query filtering unless already supported by Rust CCR store APIs.
- Changing compression algorithms or marker generation.
- Wiring OpenClaw/Gateway runtime traffic to Headroom.
- Any implementation code in Todo state.
