# llm-retry

[![Crates.io](https://img.shields.io/crates/v/llm-retry.svg)](https://crates.io/crates/llm-retry)
[![Documentation](https://docs.rs/llm-retry/badge.svg)](https://docs.rs/llm-retry)
[![CI](https://github.com/MukundaKatta/llm-retry/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/llm-retry/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/llm-retry.svg)](https://crates.io/crates/llm-retry)

**Runtime-agnostic exponential backoff with full jitter for LLM API calls.**

Most retry crates are tower middleware or async-only. This one is a small
function you can drop in front of any `Result`-returning call. Built-in
predicate lists for Anthropic, OpenAI, AWS Bedrock, and Google Gemini
retryable error codes. Sync core. Async behind a `tokio` feature.

## Install

```toml
[dependencies]
llm-retry = "0.1"
# for async
llm-retry = { version = "0.1", features = ["tokio"] }
```

## Use

```rust
use llm_retry::{retry, RetryConfig, predicates};

let resp = retry(
    &RetryConfig::default(),
    |e: &MyError| predicates::is_anthropic_retryable(&e.to_string()),
    || anthropic_client.messages_create(&req),
)?;
```

Async with tokio:

```rust
use llm_retry::{retry_async, RetryConfig};

let resp = retry_async(
    &RetryConfig::default(),
    |e: &reqwest::Error| e.is_timeout() || e.is_connect(),
    || async { reqwest::get(url).await },
).await?;
```

## Configuration

`RetryConfig` defaults: 6 attempts, 500ms base delay, 30s cap, full jitter.

```rust
use llm_retry::{RetryConfig, Jitter};
use std::time::Duration;

let cfg = RetryConfig::default()
    .with_max_attempts(8)
    .with_base_delay(Duration::from_millis(250))
    .with_max_delay(Duration::from_secs(60))
    .with_jitter(Jitter::Full);   // Full (AWS-recommended), Equal, or None
```

## Built-in predicates

```rust
use llm_retry::predicates;

assert!(predicates::is_anthropic_retryable("rate_limit_error"));
assert!(predicates::is_openai_retryable("server_error"));
assert!(predicates::is_bedrock_retryable("ThrottlingException"));
assert!(predicates::is_gemini_retryable("RESOURCE_EXHAUSTED"));
assert!(predicates::is_http_status_retryable(503));
```

The lists are `pub const` so you can compose them or extend them.

## Error type

```rust
pub enum RetryError<E> {
    Exhausted { last_err: E, attempts: u32 },
    NotRetryable(E),
}
```

`.into_inner()` and `.attempts()` are available on both arms.

## What it does NOT do

- No HTTP. Wraps any `Result`-returning call.
- No circuit breaker. If you want one, layer it on top.
- No "stop after N seconds total" deadline. Combine with your own timeout.
- No async runtime other than tokio (via the `tokio` feature). Async-std
  or smol users: wrap the sync version with their `spawn_blocking`
  equivalent.

## License

MIT OR Apache-2.0

Sibling to [`bedrock-kit`](https://github.com/MukundaKatta/bedrock-kit)
(Python AWS Bedrock client) which embeds the same retry shape.
