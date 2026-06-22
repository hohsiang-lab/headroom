# Tasks: Wire Rust OpenAI Compression Paths To CCR Store

**Input**: Design documents from `specs/001-wire-rust-openai-ccr-store/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Required by HO-2130 acceptance evidence.

**Organization**: Tasks are grouped by independently testable user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm baseline and prepare shared fixtures.

- [ ] T001 Read current CCR backend API in `crates/headroom-core/src/ccr/backends/mod.rs` and current proxy state/config in `crates/headroom-proxy/src/config.rs`, `crates/headroom-proxy/src/proxy.rs`, and `crates/headroom-proxy/src/main.rs`
- [ ] T002 [P] Add reusable test helper for extracting `<<ccr:HASH>>` markers in `crates/headroom-proxy/tests/common/mod.rs`
- [ ] T003 [P] Add reusable compressible OpenAI payload fixtures for chat/responses tests in `crates/headroom-proxy/tests/common/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Initialize and expose shared CCR store state before any OpenAI path
uses it.

**CRITICAL**: No OpenAI path wiring should start until the proxy has one
initialized store in shared state.

- [ ] T004 Add Rust proxy CCR config fields and conversion to `CcrBackendConfig` in `crates/headroom-proxy/src/config.rs`
- [ ] T005 Thread initialized `Arc<dyn CcrStore>` into `AppState` using `from_config` in `crates/headroom-proxy/src/proxy.rs`
- [ ] T006 Update proxy startup construction/error propagation in `crates/headroom-proxy/src/main.rs` so CCR init failure is loud
- [ ] T007 Add backend-init failure test coverage in `crates/headroom-proxy/tests/integration_health.rs` or a new focused proxy config test file

**Checkpoint**: Proxy app state cannot exist with a failed configured CCR
backend.

---

## Phase 3: User Story 1 - Chat Completions CCR-backed compression (Priority: P1)

**Goal**: `/v1/chat/completions` compression emits store-backed CCR markers.

**Independent Test**: A chat integration test sends a compressible tool payload,
captures the upstream body, extracts the marker hash, and verifies store
retrieval returns the original content.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add failing Chat Completions store-backed CCR marker integration test in `crates/headroom-proxy/tests/integration_chat_completions.rs`
- [ ] T009 [P] [US1] Add core dispatcher regression for OpenAI chat CCR marker/store behavior in `crates/headroom-core/src/transforms/live_zone.rs` or `crates/headroom-core/tests/`

### Implementation for User Story 1

- [ ] T010 [US1] Extend OpenAI chat core dispatcher to accept an optional CCR store in `crates/headroom-core/src/transforms/live_zone.rs`
- [ ] T011 [US1] Pass `AppState` CCR store into `compress_openai_chat_request` from the chat handler/forwarding path in `crates/headroom-proxy/src/proxy.rs` and `crates/headroom-proxy/src/compression/live_zone_openai.rs`
- [ ] T012 [US1] Ensure normalization-only chat changes do not emit CCR markers in `crates/headroom-proxy/src/compression/live_zone_openai.rs`

**Checkpoint**: Chat Completions CCR marker test passes independently.

---

## Phase 4: User Story 2 - Responses CCR-backed compression (Priority: P1)

**Goal**: `/v1/responses` compression emits store-backed CCR markers.

**Independent Test**: A responses integration test sends a compressible output
payload, captures the upstream body, extracts the marker hash, and verifies store
retrieval returns the original content.

### Tests for User Story 2

- [ ] T013 [P] [US2] Add failing Responses store-backed CCR marker integration test in `crates/headroom-proxy/tests/integration_responses.rs`
- [ ] T014 [P] [US2] Add core dispatcher regression for OpenAI responses CCR marker/store behavior in `crates/headroom-core/src/transforms/live_zone.rs` or `crates/headroom-core/tests/`

### Implementation for User Story 2

- [ ] T015 [US2] Extend OpenAI responses core dispatcher to accept an optional CCR store in `crates/headroom-core/src/transforms/live_zone.rs`
- [ ] T016 [US2] Pass `AppState` CCR store into `compress_openai_responses_request` from the responses handler/forwarding path in `crates/headroom-proxy/src/proxy.rs` and `crates/headroom-proxy/src/compression/live_zone_responses.rs`
- [ ] T017 [US2] Ensure responses unknown/encrypted/cache-hot item tests remain byte-preserving in `crates/headroom-proxy/tests/integration_responses.rs`

**Checkpoint**: Responses CCR marker test passes independently.

---

## Phase 5: User Story 3 - Loud CCR backend initialization failure (Priority: P2)

**Goal**: Invalid CCR backend config fails app/proxy initialization without
fallback.

**Independent Test**: Configure invalid SQLite path or unsupported Redis and
assert initialization returns error.

### Tests for User Story 3

- [ ] T018 [P] [US3] Add invalid SQLite path test for proxy CCR init failure in `crates/headroom-proxy/tests/`
- [ ] T019 [P] [US3] Add unsupported Redis feature-off config test if proxy config exposes Redis selection in `crates/headroom-proxy/tests/`

### Implementation for User Story 3

- [ ] T020 [US3] Map `CcrBackendInitError` into existing proxy error type without swallowing details in `crates/headroom-proxy/src/error.rs`
- [ ] T021 [US3] Add startup log fields for selected CCR backend after successful init in `crates/headroom-proxy/src/main.rs` or `crates/headroom-proxy/src/proxy.rs`

**Checkpoint**: Invalid backend config cannot silently run with another store.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validate acceptance evidence and guard scope.

- [ ] T022 Run `cargo fmt --check`
- [ ] T023 Run focused Rust tests for `headroom-core` CCR/live-zone coverage
- [ ] T024 Run focused Rust proxy OpenAI chat/responses integration tests
- [ ] T025 Run local Rust proxy smoke for `/v1/chat/completions` and `/v1/responses`
- [ ] T026 Verify final diff contains no Python production image or Docker production-entrypoint changes
- [ ] T027 Update PR body with acceptance evidence and implementation-detail clarity line

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup and blocks all user stories.
- **US1 / US2**: Both depend on Foundational and can proceed in parallel after shared store state exists.
- **US3**: Depends on Foundational and can proceed in parallel with OpenAI path wiring.
- **Polish**: Depends on implemented stories selected for the PR.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T008/T009 and T013/T014 can run in parallel after Foundational.
- US1 and US2 implementation can proceed in parallel if one worker owns chat and another owns responses.
- T018 and T019 can run in parallel once config selection exists.

## Implementation Strategy

### MVP First

1. Complete T001-T007.
2. Complete US1 Chat Completions marker/store proof.
3. Complete US2 Responses marker/store proof.
4. Complete US3 no-silent-fallback proof.
5. Run focused tests and smoke before moving to review.

### Stop Condition

Do not implement any task while Linear HO-2130 remains `Todo` or `Waiting`.
Implementation starts only after Linear is moved to `In Progress`.
