# Contract: Rust Proxy CCR Store Wiring

## Startup / App-State Contract

**Given** Rust proxy configuration selects a CCR backend
**When** `AppState` is constructed
**Then** the selected backend is initialized through `from_config` exactly once
for the process state.

**Given** backend initialization returns an error
**When** the proxy starts or test app state is created
**Then** initialization returns an error and the proxy does not continue with a
different backend.

## OpenAI Chat Completions Contract

**Given** compression is enabled, `compression_mode` is `live_zone`, and the
configured CCR store initialized successfully
**When** a compressible `/v1/chat/completions` body is processed
**Then** the upstream-bound body contains a `<<ccr:HASH>>` marker and
`store.get(HASH)` returns the original block content.

## OpenAI Responses Contract

**Given** compression is enabled, `compression_mode` is `live_zone`, and the
configured CCR store initialized successfully
**When** a compressible `/v1/responses` body is processed
**Then** the upstream-bound body contains a `<<ccr:HASH>>` marker and
`store.get(HASH)` returns the original block content.

## Non-Compression Contract

**Given** the request is passthrough, below threshold, cache-hot, unknown,
encrypted, or normalization-only
**When** it is processed
**Then** no CCR marker is emitted for that path and byte-preservation guarantees
remain unchanged for fields outside accepted compression replacements.
