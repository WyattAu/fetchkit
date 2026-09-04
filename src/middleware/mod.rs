//! Middleware implementations for `fetchkit`.

#[cfg(feature = "circuit-breaker")]
mod circuit_breaker;

#[cfg(feature = "circuit-breaker")]
pub use circuit_breaker::CircuitBreakerMiddleware;
