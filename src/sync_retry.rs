//! Synchronous retry loop.

use std::thread::sleep;

use rand::thread_rng;

use crate::config::RetryConfig;
use crate::error::RetryError;

/// Run `op` with retries on errors where `should_retry(&err)` is true.
///
/// On exhausting `config.max_attempts`, returns `RetryError::Exhausted`.
/// On the first non-retryable error, returns `RetryError::NotRetryable`.
/// On success, returns the value.
///
/// Blocks the calling thread during backoff sleeps - use `retry_async` from
/// the `tokio` feature for async contexts.
pub fn retry<T, E, F>(
    config: &RetryConfig,
    should_retry: impl Fn(&E) -> bool,
    mut op: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Result<T, E>,
{
    // `max_attempts == 0` is treated as 1: we always call `op` at least once
    // so there is an error (or value) to report.
    let attempts = config.max_attempts.max(1);
    let mut rng = thread_rng();

    let mut last_err: Option<E> = None;
    for attempt in 0..attempts {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !should_retry(&e) {
                    return Err(RetryError::NotRetryable(e));
                }
                last_err = Some(e);
                if attempt + 1 < attempts {
                    let delay = config.delay_for(attempt, &mut rng);
                    if !delay.is_zero() {
                        sleep(delay);
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use super::*;

    fn fast_config() -> RetryConfig {
        RetryConfig::default()
            .with_max_attempts(5)
            .with_base_delay(Duration::from_millis(0))
            .with_max_delay(Duration::from_millis(0))
    }

    #[test]
    fn returns_on_first_success() {
        let calls = Cell::new(0u32);
        let r: Result<u32, RetryError<&str>> = retry(
            &fast_config(),
            |_| true,
            || {
                calls.set(calls.get() + 1);
                Ok(42)
            },
        );
        assert_eq!(r.unwrap(), 42);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn retries_until_success() {
        let calls = Cell::new(0u32);
        let r: Result<u32, RetryError<&str>> = retry(
            &fast_config(),
            |_| true,
            || {
                calls.set(calls.get() + 1);
                if calls.get() < 3 {
                    Err("transient")
                } else {
                    Ok(7)
                }
            },
        );
        assert_eq!(r.unwrap(), 7);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn exhausts_after_max_attempts() {
        let calls = Cell::new(0u32);
        let r: Result<u32, RetryError<&str>> = retry(
            &fast_config(),
            |_| true,
            || {
                calls.set(calls.get() + 1);
                Err("nope")
            },
        );
        match r.unwrap_err() {
            RetryError::Exhausted { last_err, attempts } => {
                assert_eq!(last_err, "nope");
                assert_eq!(attempts, 5);
            }
            other => panic!("unexpected: {other:?}"),
        }
        assert_eq!(calls.get(), 5);
    }

    #[test]
    fn non_retryable_returns_immediately() {
        let calls = Cell::new(0u32);
        let r: Result<u32, RetryError<&str>> = retry(
            &fast_config(),
            |e| *e != "fatal",
            || {
                calls.set(calls.get() + 1);
                Err("fatal")
            },
        );
        match r.unwrap_err() {
            RetryError::NotRetryable(e) => assert_eq!(e, "fatal"),
            other => panic!("unexpected: {other:?}"),
        }
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn max_attempts_zero_still_calls_once() {
        let calls = Cell::new(0u32);
        let r: Result<u32, RetryError<&str>> = retry(
            &fast_config().with_max_attempts(0),
            |_| true,
            || {
                calls.set(calls.get() + 1);
                Err("x")
            },
        );
        assert!(r.is_err());
        assert_eq!(calls.get(), 1);
    }
}
