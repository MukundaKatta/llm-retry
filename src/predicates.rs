//! Built-in retryable-error code lists for major LLM providers.
//!
//! Each list is what the corresponding provider documents as a transient,
//! caller-should-retry error. Use these against a stringified error code
//! or message:
//!
//! ```
//! use llm_retry::predicates::is_anthropic_retryable;
//! assert!(is_anthropic_retryable("rate_limit_error"));
//! assert!(!is_anthropic_retryable("authentication_error"));
//! ```
//!
//! The match is `contains` (case-sensitive) so you can pass a full error
//! message or just the code substring.

/// Anthropic API error codes that are safe to retry.
///
/// Source: <https://docs.anthropic.com/en/api/errors>
pub const ANTHROPIC_RETRYABLE: &[&str] = &[
    "rate_limit_error",
    "overloaded_error",
    "api_error",
    "timeout",
];

/// OpenAI API error codes/conditions that are safe to retry.
///
/// Source: <https://platform.openai.com/docs/guides/error-codes>
pub const OPENAI_RETRYABLE: &[&str] = &[
    "rate_limit_exceeded",
    "server_error",
    "engine_overloaded",
    "tokens_exhausted",
    "timeout",
];

/// AWS Bedrock service-side error codes that are safe to retry.
///
/// Source: <https://docs.aws.amazon.com/bedrock/latest/userguide/troubleshoot.html>
pub const BEDROCK_RETRYABLE: &[&str] = &[
    "ThrottlingException",
    "Throttling",
    "TooManyRequestsException",
    "ServiceUnavailableException",
    "ProvisionedThroughputExceededException",
    "ModelTimeoutException",
];

/// Google Gemini API status codes that are safe to retry.
///
/// Source: <https://ai.google.dev/api/rest/v1/HttpStatusCode>
pub const GEMINI_RETRYABLE: &[&str] = &[
    "RESOURCE_EXHAUSTED",
    "UNAVAILABLE",
    "DEADLINE_EXCEEDED",
    "INTERNAL",
];

/// Generic HTTP status codes that are typically retryable.
pub const HTTP_RETRYABLE_STATUSES: &[u16] = &[408, 425, 429, 500, 502, 503, 504];

/// True if `s` contains any of `patterns`.
pub fn contains_any(s: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| s.contains(p))
}

/// True if `s` looks like an Anthropic retryable error code/message.
pub fn is_anthropic_retryable(s: &str) -> bool {
    contains_any(s, ANTHROPIC_RETRYABLE)
}

/// True if `s` looks like an OpenAI retryable error code/message.
pub fn is_openai_retryable(s: &str) -> bool {
    contains_any(s, OPENAI_RETRYABLE)
}

/// True if `s` looks like a Bedrock retryable error code/message.
pub fn is_bedrock_retryable(s: &str) -> bool {
    contains_any(s, BEDROCK_RETRYABLE)
}

/// True if `s` looks like a Gemini retryable error code/message.
pub fn is_gemini_retryable(s: &str) -> bool {
    contains_any(s, GEMINI_RETRYABLE)
}

/// True if `code` is a typically-retryable HTTP status.
pub fn is_http_status_retryable(code: u16) -> bool {
    HTTP_RETRYABLE_STATUSES.contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_matches_codes() {
        assert!(is_anthropic_retryable("rate_limit_error"));
        assert!(is_anthropic_retryable("overloaded_error"));
        assert!(is_anthropic_retryable(
            "a wrapped: rate_limit_error happened"
        ));
        assert!(!is_anthropic_retryable("authentication_error"));
        assert!(!is_anthropic_retryable("invalid_request_error"));
    }

    #[test]
    fn bedrock_matches_codes() {
        assert!(is_bedrock_retryable("ThrottlingException"));
        assert!(is_bedrock_retryable("ServiceUnavailableException"));
        assert!(is_bedrock_retryable("ModelTimeoutException"));
        assert!(!is_bedrock_retryable("ValidationException"));
        assert!(!is_bedrock_retryable("AccessDeniedException"));
    }

    #[test]
    fn openai_matches_codes() {
        assert!(is_openai_retryable("rate_limit_exceeded"));
        assert!(is_openai_retryable("server_error"));
        assert!(!is_openai_retryable("invalid_api_key"));
    }

    #[test]
    fn gemini_matches_codes() {
        assert!(is_gemini_retryable("RESOURCE_EXHAUSTED"));
        assert!(is_gemini_retryable("UNAVAILABLE"));
        assert!(!is_gemini_retryable("PERMISSION_DENIED"));
    }

    #[test]
    fn http_status_retryable() {
        assert!(is_http_status_retryable(429));
        assert!(is_http_status_retryable(503));
        assert!(is_http_status_retryable(504));
        assert!(!is_http_status_retryable(400));
        assert!(!is_http_status_retryable(401));
        assert!(!is_http_status_retryable(404));
    }
}
