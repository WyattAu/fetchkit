# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [0.1.2] - 2026-09-12

### Added

- Wire-level integration suite (`tests/wire.rs`, 16 tests) against real
  HTTP servers (wiremock): GET/POST/JSON round trips, base-URL resolution,
  header/auth/query propagation, PUT/DELETE passthrough, status-error
  mapping, primary/fallback recovery, client and per-request timeouts,
  retry-until-success / retry-exhaustion / no-retry-on-4xx, multipart
  upload shape, and the `circuit-breaker` feature over the wire (breaker
  opens after 3 consecutive 5xx and short-circuits with
  `FetchError::CircuitOpen`; stays closed through sub-threshold failures).

### Fixed

- Error fidelity: `FetchError::CircuitOpen` now surfaces as the
  `CircuitOpen` variant instead of being buried in
  `FetchError::Middleware(String)`. The middleware no longer erases the
  concrete type (`anyhow!` → `Into<anyhow::Error>`), and the
  `reqwest_middleware::Error → FetchError` conversion downcasts through
  both fetchkit's own middleware errors and `reqwest_retry::RetryError`
  wrapping. Callers matching on `FetchError::CircuitOpen` now work as the
  API always documented.

### CI

- New `integration` job running the wire suite with all features.

## [0.1.1] - 2026-09-11

### Fixed

- 22-gate quality audit pass: documentation completeness
  (README badges, REQUIREMENTS/THREAT-MODEL coverage) and
  feature-gated test hygiene.

## [0.1.0] - 2026-09-05

### Added
- Initial public release.
