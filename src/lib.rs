#![forbid(unsafe_code)]

//! Resilient HTTP client for Rust.
//!
//! `fetchkit` wraps `reqwest-middleware` with sensible defaults for retries,
//! timeouts, and typed JSON helpers. Enable the `circuit-breaker` feature for
//! automatic circuit-breaking on repeated failures.

pub mod error;

use std::time::Duration;

use reqwest_middleware::{ClientBuilder as ReqwestClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{de::DeserializeOwned, Serialize};

pub use error::FetchError;
pub use reqwest::Response;

/// A resilient HTTP client with retry, timeout, and typed JSON support.
#[derive(Clone, Debug)]
pub struct Client {
    inner: ClientWithMiddleware,
    base_url: Option<String>,
}

/// Builder for constructing a [`Client`] with custom configuration.
pub struct ClientBuilder {
    reqwest_builder: reqwest::ClientBuilder,
    retry_policy: ExponentialBackoff,
    base_url: Option<String>,
    retries: u32,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Create a new builder with defaults (30s timeout, 3 retries, exponential backoff).
    pub fn new() -> Self {
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(3);

        let reqwest_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30));

        Self {
            reqwest_builder,
            retry_policy,
            base_url: None,
            retries: 3,
        }
    }

    /// Set a base URL that is prepended to all relative paths.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.reqwest_builder = self.reqwest_builder.timeout(timeout);
        self
    }

    /// Set the maximum number of retries.
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self.retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(retries);
        self
    }

    /// Add a raw `reqwest::ClientBuilder` for advanced configuration.
    pub fn reqwest_builder(mut self, builder: reqwest::ClientBuilder) -> Self {
        self.reqwest_builder = builder;
        self
    }

    /// Build the [`Client`].
    pub fn build(self) -> Client {
        let reqwest_client = self.reqwest_builder.build().expect("failed to build reqwest client");
        let inner = ReqwestClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(self.retry_policy))
            .build();

        Client {
            inner,
            base_url: self.base_url,
        }
    }
}

impl Client {
    /// Create a new client with default settings.
    pub fn new() -> Self {
        ClientBuilder::new().build()
    }

    /// Start building a client with custom settings.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Resolve a path against the optional base URL.
    fn resolve_url(&self, url: &str) -> String {
        match &self.base_url {
            Some(base) => {
                let base = base.trim_end_matches('/');
                let path = url.trim_start_matches('/');
                format!("{base}/{path}")
            }
            None => url.to_string(),
        }
    }

    /// Perform a GET request and deserialize the JSON response.
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, FetchError> {
        let url = self.resolve_url(url);
        let response = self.inner.get(&url).send().await?;
        let response = Self::check_status(response).await?;
        Ok(response.json().await?)
    }

    /// Perform a POST request with a JSON body and deserialize the JSON response.
    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, FetchError> {
        let url = self.resolve_url(url);
        let response = self.inner
            .post(&url)
            .json(body)
            .send()
            .await?;
        let response = Self::check_status(response).await?;
        Ok(response.json().await?)
    }

    /// Fetch from `primary_url`; on failure, fall back to `fallback_url`.
    pub async fn fetch_with_fallback<T: DeserializeOwned>(
        &self,
        primary_url: &str,
        fallback_url: &str,
    ) -> Result<T, FetchError> {
        match self.get_json(primary_url).await {
            Ok(value) => Ok(value),
            Err(_) => self.get_json(fallback_url).await,
        }
    }

    /// Access the inner `reqwest_middleware::ClientWithMiddleware`.
    pub fn inner(&self) -> &ClientWithMiddleware {
        &self.inner
    }

    async fn check_status(response: Response) -> Result<Response, FetchError> {
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            let code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            Err(FetchError::StatusCode { status: code, body })
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
