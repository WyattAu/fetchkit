# Requirements — fetchkit

Numbered, testable requirements. Every requirement maps to at least one named
test or doc-comment contract; security-relevant items cite THREAT-MODEL.md rows.

Scope: Resilient HTTP client — reqwest-middleware wrapper with retries, timeouts, and circuit breaking

## Functional

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-FK-001 | Retries apply only to transient failures with exponential backoff + jitter; non-retryable statuses surface immediately | MUST |
| REQ-FK-002 | Circuit breaker opens after the configured failure threshold and half-opens after the cooldown | MUST |
| REQ-FK-003 | Per-request timeouts are enforced; no unbounded waits | MUST |

## Security

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-FK-100 | TLS verification is never disabled by the crate (no dangerous-config constructors) | MUST |
| REQ-FK-101 | Response bodies are bounded by configured limits before buffering | SHOULD |

## Observability & API hygiene

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-FK-900 | All fallible public APIs return typed errors; production `unwrap`/`expect` is denied or explicitly justified with an invariant comment | MUST |
| REQ-FK-901 | Public items carry doc comments with runnable examples where practical | SHOULD |

Reviewed: 2026-09-11
