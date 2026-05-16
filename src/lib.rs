//! # llm-retry
//!
//! Runtime-agnostic exponential backoff with full jitter for LLM API calls.
//!
//! ## Sync
//!
//! ```
//! use llm_retry::{retry, RetryConfig, RetryError};
//! use std::time::Duration;
//!
//! #[derive(Debug)]
//! struct MyErr(bool /* retryable */);
//!
//! let mut tries = 0;
//! let result: Result<u32, _> = retry(
//!     &RetryConfig::default().with_max_delay(Duration::from_millis(10)),
//!     |e: &MyErr| e.0,
//!     || {
//!         tries += 1;
//!         if tries < 3 { Err(MyErr(true)) } else { Ok(42) }
//!     },
//! );
//! assert_eq!(result.unwrap(), 42);
//! assert_eq!(tries, 3);
//! ```
//!
//! ## Async (with `tokio` feature)
//!
//! ```ignore
//! use llm_retry::{retry_async, RetryConfig};
//!
//! let response = retry_async(
//!     &RetryConfig::default(),
//!     |e: &reqwest::Error| e.is_timeout() || e.is_connect(),
//!     || async {
//!         reqwest::get("https://api.example.com/v1/messages").await
//!     },
//! ).await;
//! ```
//!
//! ## Provider-specific retryable codes
//!
//! For string-shaped errors (e.g. AWS error codes, JSON error fields), use
//! the [`predicates`] module:
//!
//! ```
//! use llm_retry::predicates;
//!
//! assert!(predicates::is_anthropic_retryable("rate_limit_error"));
//! assert!(predicates::is_anthropic_retryable("overloaded_error"));
//! assert!(!predicates::is_anthropic_retryable("invalid_request_error"));
//!
//! assert!(predicates::is_bedrock_retryable("ThrottlingException"));
//! assert!(predicates::is_bedrock_retryable("ServiceUnavailableException"));
//! assert!(!predicates::is_bedrock_retryable("ValidationException"));
//! ```

#![deny(missing_docs)]

mod config;
mod error;
pub mod predicates;
mod sync_retry;

#[cfg(feature = "tokio")]
mod async_retry;

pub use config::{Jitter, RetryConfig};
pub use error::RetryError;
pub use sync_retry::retry;

#[cfg(feature = "tokio")]
pub use async_retry::retry_async;
