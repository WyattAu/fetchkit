use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};

use crate::error::FetchError;

/// A `reqwest-middleware` middleware that wraps a [`breaker::CircuitBreaker`].
///
/// Before each request the circuit state is checked. If the circuit is
/// **Open** the request is short-circuited with [`FetchError::CircuitOpen`].
///
/// After a successful response (2xx / 3xx / 4xx other than 429) the breaker
/// records a success. On 5xx, 429, or network errors a failure is recorded.
pub struct CircuitBreakerMiddleware {
    breaker: breaker::CircuitBreaker,
}

impl CircuitBreakerMiddleware {
    /// Create a new middleware wrapping the given circuit breaker.
    pub fn new(breaker: breaker::CircuitBreaker) -> Self {
        Self { breaker }
    }
}

#[async_trait::async_trait]
impl Middleware for CircuitBreakerMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        use breaker::State;

        match self.breaker.state() {
            State::Open => {
                return Err(reqwest_middleware::Error::Middleware(anyhow::anyhow!(
                    FetchError::CircuitOpen
                )));
            }
            State::HalfOpen => {}
            State::Closed => {}
        }

        match next.run(req, extensions).await {
            Ok(response) => {
                let status = response.status().as_u16();
                if status == 429 || status >= 500 {
                    self.breaker.record_failure();
                } else {
                    self.breaker.record_success();
                }
                Ok(response)
            }
            Err(err) => {
                self.breaker.record_failure();
                Err(err)
            }
        }
    }
}
