//! Integration tests for llm-retry.

use std::cell::Cell;
use std::time::Duration;

use llm_retry::{predicates, retry, Jitter, RetryConfig, RetryError};

fn fast_config() -> RetryConfig {
    RetryConfig::default()
        .with_max_attempts(4)
        .with_base_delay(Duration::from_millis(0))
        .with_max_delay(Duration::from_millis(0))
        .with_jitter(Jitter::None)
}

#[test]
fn anthropic_style_loop_succeeds_after_throttle() {
    let calls = Cell::new(0u32);
    let result: Result<&'static str, RetryError<&'static str>> = retry(
        &fast_config(),
        |e: &&'static str| predicates::is_anthropic_retryable(e),
        || {
            calls.set(calls.get() + 1);
            if calls.get() < 3 {
                Err("rate_limit_error: please slow down")
            } else {
                Ok("response body")
            }
        },
    );
    assert_eq!(result.unwrap(), "response body");
    assert_eq!(calls.get(), 3);
}

#[test]
fn bedrock_style_loop_does_not_retry_validation_error() {
    let calls = Cell::new(0u32);
    let result: Result<&'static str, _> = retry(
        &fast_config(),
        |e: &&'static str| predicates::is_bedrock_retryable(e),
        || {
            calls.set(calls.get() + 1);
            Err("ValidationException: invalid input")
        },
    );
    assert!(matches!(result.unwrap_err(), RetryError::NotRetryable(_)));
    assert_eq!(calls.get(), 1);
}

#[test]
fn exhausted_error_preserves_attempts() {
    let result: Result<u32, _> = retry(
        &fast_config().with_max_attempts(7),
        |_: &&'static str| true,
        || Err("nope"),
    );
    let err = result.unwrap_err();
    assert_eq!(err.attempts(), 7);
    assert_eq!(err.into_inner(), "nope");
}

#[test]
fn http_retryable_status_predicate() {
    assert!(predicates::is_http_status_retryable(429));
    assert!(predicates::is_http_status_retryable(503));
    assert!(!predicates::is_http_status_retryable(401));
}

#[test]
fn custom_predicate_works() {
    let calls = Cell::new(0u32);
    let r: Result<u32, _> = retry(
        &fast_config(),
        |e: &i32| *e == 1,
        || {
            calls.set(calls.get() + 1);
            if calls.get() < 2 {
                Err(1)
            } else {
                Err(2)
            }
        },
    );
    // attempt 1: e=1 retryable -> retry
    // attempt 2: e=2 not retryable -> NotRetryable
    assert!(matches!(r.unwrap_err(), RetryError::NotRetryable(2)));
    assert_eq!(calls.get(), 2);
}

#[test]
fn full_jitter_loop_still_succeeds() {
    // run with real (small) jitter to make sure sleep math doesn't deadlock
    let cfg = RetryConfig::default()
        .with_max_attempts(5)
        .with_base_delay(Duration::from_micros(1))
        .with_max_delay(Duration::from_micros(5))
        .with_jitter(Jitter::Full);
    let calls = Cell::new(0u32);
    let r: Result<u32, RetryError<&'static str>> = retry(
        &cfg,
        |_| true,
        || {
            calls.set(calls.get() + 1);
            if calls.get() < 3 {
                Err("transient")
            } else {
                Ok(99)
            }
        },
    );
    assert_eq!(r.unwrap(), 99);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn async_loop_succeeds_after_two_retries() {
    use llm_retry::retry_async;
    use std::sync::atomic::{AtomicU32, Ordering};

    let calls = AtomicU32::new(0);
    let r: Result<u32, RetryError<&'static str>> = retry_async(
        &fast_config(),
        |_| true,
        || {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err("transient")
                } else {
                    Ok(7)
                }
            }
        },
    )
    .await;
    assert_eq!(r.unwrap(), 7);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn async_loop_respects_non_retryable() {
    use llm_retry::retry_async;

    let r: Result<u32, _> = retry_async(
        &fast_config(),
        |_e: &&'static str| false,
        || async { Err("fatal") },
    )
    .await;
    assert!(matches!(r.unwrap_err(), RetryError::NotRetryable("fatal")));
}
