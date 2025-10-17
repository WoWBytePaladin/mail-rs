//! SMTP client implementation for mail-rs
//!
//! This crate provides asynchronous SMTP client functionality with support for:
//! - STARTTLS and direct TLS connections
//! - Multiple authentication mechanisms (PLAIN, LOGIN, CRAM-MD5)
//! - Connection pooling for efficient bulk sending
//! - Retry logic with exponential backoff
//! - Rate limiting for server compliance
//! - Async/await API

pub mod client;
pub mod transport;
pub mod auth;
pub mod error;
pub mod pool;
pub mod retry;
pub mod ratelimit;

#[cfg(test)]
mod tests;

pub use client::{SmtpClient, SmtpConnection};
pub use transport::{SmtpTransport, TlsConfig};
pub use auth::{Credentials, AuthMechanism};
pub use error::{Error, Result};
pub use pool::{SmtpPool, PoolConfig, PoolStats};
pub use retry::{RetryConfig, RetryableSmtpClient, RetryableSmtpPool, send_with_retry};
pub use ratelimit::{
    RateLimitConfig, RateLimitedSmtpClient, RateLimitedSmtpPool,
    TokenBucketRateLimiter, SlidingWindowRateLimiter, RateLimitStats
};

// Re-export mail-core types
pub use mail_core::{Message, Address};
