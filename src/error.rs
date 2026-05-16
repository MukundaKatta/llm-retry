//! Error type for retry loops.

use thiserror::Error;

/// Outcome of a retry loop that did not succeed.
#[derive(Debug, Error)]
pub enum RetryError<E> {
    /// All attempts were exhausted. Holds the last error and the total
    /// number of attempts that ran.
    #[error("retry exhausted after {attempts} attempts: {last_err}")]
    Exhausted {
        /// The error returned on the final attempt.
        #[source]
        last_err: E,
        /// Total attempts executed (including the first).
        attempts: u32,
    },

    /// The predicate said this error is not retryable. Holds it unchanged.
    #[error("non-retryable error: {0}")]
    NotRetryable(#[source] E),
}

impl<E> RetryError<E> {
    /// Unwrap into the inner error if you don't care which arm it is.
    pub fn into_inner(self) -> E {
        match self {
            RetryError::Exhausted { last_err, .. } => last_err,
            RetryError::NotRetryable(e) => e,
        }
    }

    /// Number of attempts run before giving up. Always 1 for `NotRetryable`.
    pub fn attempts(&self) -> u32 {
        match self {
            RetryError::Exhausted { attempts, .. } => *attempts,
            RetryError::NotRetryable(_) => 1,
        }
    }
}
