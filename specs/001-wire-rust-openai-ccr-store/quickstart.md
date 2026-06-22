# Quickstart: Rust OpenAI CCR Store Wiring Verification

## Focused Tests

```bash
cargo test -p headroom-core ccr
cargo test -p headroom-proxy integration_chat_completions -- --nocapture
cargo test -p headroom-proxy integration_responses -- --nocapture
```

## Expected New Evidence

- Chat Completions test captures upstream body, extracts `<<ccr:HASH>>`, and
  verifies the configured test store returns the original tool/user content.
- Responses test captures upstream body, extracts `<<ccr:HASH>>`, and verifies
  the configured test store returns the original output content.
- Backend-init failure test configures an invalid backend and asserts
  initialization errors instead of falling back.

## Local Rust Proxy Smoke

```bash
cargo run -p headroom-proxy -- \
  --upstream http://127.0.0.1:8788 \
  --compression \
  --compression-mode live_zone
```

Smoke both paths against a local upstream:

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-4o","messages":[{"role":"user","content":"hello"}]}'

curl -sS -X POST http://127.0.0.1:8787/v1/responses \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-4o","input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"hello"}]}]}'
```

The smoke is successful when both requests reach the upstream and return the
upstream response without proxy startup/config errors.
