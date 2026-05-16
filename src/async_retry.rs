//! Async retry loop (tokio feature).

use std::future::Future;

use rand::thread_rng;

use crate::config::RetryConfig;
use crate::error::RetryError;

/// Async version of [`retry`](crate::retry). Same semantics but uses
/// `tokio::time::sleep` for backoff so it doesn't block the runtime.
pub async fn retry_async<T, E, F, Fut>(
    config: &RetryConfig,
    should_retry: impl Fn(&E) -> bool,
    mut op: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let attempts = config.max_attempts.max(1);
    let mut rng = thread_rng();

    let mut last_err: Option<E> = None;
    for attempt in 0..attempts {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !should_retry(&e) {
                    return Err(RetryError::NotRetryable(e));
                }
                last_err = Some(e);
                if attempt + 1 < attempts {
                    let delay = config.delay_for(attempt, &mut rng);
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }

    Err(RetryError::Exhausted {
        last_err: last_err.expect("attempts >= 1 so we must have errored"),
        attempts,
    })
}
