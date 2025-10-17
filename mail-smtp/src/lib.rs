//! SMTP client implementation for mail-rs
//!
//! This crate provides asynchronous SMTP client functionality with support for:
//! - STARTTLS and direct TLS connections
//! - Multiple authentication mechanisms (PLAIN, LOGIN, CRAM-MD5)
//! - Connection pooling for efficient bulk sending
//! - Async/await API

pub mod client;
pub mod transport;
pub mod auth;
pub mod error;

pub use client::SmtpClient;
pub use transport::{SmtpTransport, TlsConfig};
pub use auth::{Credentials, AuthMechanism};
pub use error::{Error, Result};

// Re-export mail-core types
pub use mail_core::{Message, Address};
