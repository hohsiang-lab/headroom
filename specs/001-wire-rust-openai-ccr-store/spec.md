# Feature Specification: Wire Rust OpenAI Compression Paths To CCR Store

**Feature Branch**: `001-wire-rust-openai-ccr-store`
**Created**: 2026-06-22
**Status**: Draft
**Input**: Linear HO-2130 "[Headroom] Wire Rust OpenAI compression paths to CCR store"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Chat Completions CCR-backed compression (Priority: P1)

As a Headroom operator running the Rust proxy for OpenAI-compatible Chat
Completions traffic, I need compressed live-zone tool/user content to include
CCR markers that are backed by the configured store, so retrieval can recover
the original payload when needed.

**Why this priority**: Chat Completions is one of the two OpenAI paths already
served by the Rust proxy and is explicitly listed in the acceptance evidence.

**Independent Test**: A Rust integration test can send a compressible
`/v1/chat/completions` request through the proxy with a test CCR backend, assert
the upstream body contains a `<<ccr:...>>` marker, and assert the store contains
the original payload for that marker.

**Acceptance Scenarios**:

1. **Given** the Rust proxy is configured with a working CCR backend and
   compression is enabled, **When** a compressible OpenAI Chat Completions
   request is proxied, **Then** the upstream body contains at least one CCR
   marker and the corresponding original content is retrievable from the store.
2. **Given** a Chat Completions request does not compress because it is below
   threshold or is otherwise ineligible, **When** it is proxied, **Then** no CCR
   marker is emitted and no spurious store entry is required.

---

### User Story 2 - Responses CCR-backed compression (Priority: P1)

As a Headroom operator running the Rust proxy for OpenAI Responses traffic, I
need compressed live-zone output items to include CCR markers backed by the
configured store, so the Responses path has the same reversible compression
contract as Chat Completions.

**Why this priority**: `/v1/responses` is the Codex-facing path and is explicitly
listed in the acceptance evidence.

**Independent Test**: A Rust integration test can send a compressible
`/v1/responses` request through the proxy with a test CCR backend, assert the
upstream body contains a `<<ccr:...>>` marker, and assert the store contains the
original output for that marker.

**Acceptance Scenarios**:

1. **Given** the Rust proxy is configured with a working CCR backend and
   compression is enabled, **When** a compressible OpenAI Responses request is
   proxied, **Then** the upstream body contains at least one CCR marker and the
   corresponding original content is retrievable from the store.
2. **Given** a Responses request includes non-compressible or cache-hot item
   types, **When** it is proxied, **Then** those fields remain byte-preserved and
   no unrelated CCR markers are introduced.

---

### User Story 3 - Loud CCR backend initialization failure (Priority: P2)

As a Headroom operator, I need an invalid configured CCR backend to fail startup
or app initialization loudly, so the proxy never silently runs with missing
retrieval backing.

**Why this priority**: Silent fallback would make emitted markers misleading and
is called out directly in the issue.

**Independent Test**: A Rust test can configure an invalid SQLite path or
unsupported backend and assert proxy/app initialization returns an error instead
of substituting in-memory storage.

**Acceptance Scenarios**:

1. **Given** a configured CCR backend cannot initialize, **When** the Rust proxy
   initializes, **Then** initialization fails with a visible error and does not
   continue with another backend.
2. **Given** no explicit CCR backend override is supplied, **When** the Rust
   proxy initializes for the supported single-worker path, **Then** it uses the
   documented default backend and surfaces that choice in startup evidence.

### Edge Cases

- Configured SQLite path is invalid, unwritable, or points at a missing parent
  directory.
- Redis is requested while the Rust binary is not compiled with the `redis`
  feature.
- Compression mode is off: requests pass through without CCR writes or markers.
- OpenAI `n > 1` Chat Completions requests remain passthrough and do not create
  CCR entries.
- Responses unknown or encrypted item types remain byte-preserved.
- Tool-definition normalization may change a request without live-zone
  compression; this must not create a fake CCR marker.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Rust proxy MUST expose configuration for selecting a CCR
  backend using the existing backend concepts: in-memory for tests, SQLite for
  the first persistent rollout, and Redis only as a later opt-in path.
- **FR-002**: The Rust proxy MUST initialize the selected CCR backend at startup
  or app-state construction using the existing `CcrBackendConfig` /
  `from_config` contract.
- **FR-003**: The Rust proxy MUST fail loudly when the selected CCR backend
  cannot initialize, without silently substituting another backend.
- **FR-004**: OpenAI Chat Completions live-zone compression MUST receive the
  initialized CCR store and emit store-backed CCR markers when compression is
  accepted.
- **FR-005**: OpenAI Responses live-zone compression MUST receive the initialized
  CCR store and emit store-backed CCR markers when compression is accepted.
- **FR-006**: Existing passthrough, byte-preservation, cache-hot-zone, and
  no-compression semantics MUST remain intact for OpenAI Chat Completions and
  Responses requests that are not eligible for live-zone compression.
- **FR-007**: Rust tests MUST prove Chat Completions and Responses can each emit
  CCR markers backed by a store.
- **FR-008**: Rust tests MUST prove invalid CCR backend initialization is not
  silently downgraded.
- **FR-009**: Local Rust proxy smoke evidence MUST cover both `/v1/responses`
  and `/v1/chat/completions` after the wiring is implemented.
- **FR-010**: Python production image behavior MUST remain unchanged.

### Key Entities

- **CCR Backend Selection**: Operator-facing choice of in-memory, SQLite, or
  Redis-capable backend configuration.
- **CCR Store**: Shared runtime storage object that accepts original payloads by
  hash and supports retrieval by emitted marker hash.
- **OpenAI Compression Path**: Chat Completions and Responses request rewriting
  flow that may replace live-zone content with compressed text plus CCR marker.

### Out Of Scope

- Enabling Redis rollout in production.
- Replacing the Python production proxy/image entrypoint.
- Adding `/v1/retrieve` or stats endpoint parity for the Rust proxy.
- Changing OpenAI API request/response semantics beyond CCR marker backing.
- Changing compression algorithms or thresholds.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: One Chat Completions Rust test demonstrates a compressible request
  emits a `<<ccr:...>>` marker and the marker hash resolves to the original
  payload in the configured store.
- **SC-002**: One Responses Rust test demonstrates a compressible request emits a
  `<<ccr:...>>` marker and the marker hash resolves to the original payload in
  the configured store.
- **SC-003**: One Rust test demonstrates an invalid CCR backend configuration
  fails initialization instead of falling back.
- **SC-004**: Local Rust proxy smoke returns successful upstream passthrough for
  both `/v1/chat/completions` and `/v1/responses`.
- **SC-005**: The final implementation diff contains no Python production image
  or Docker production-entrypoint changes.
