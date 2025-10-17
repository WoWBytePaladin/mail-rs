//! High-level email builder and sender
//!
//! This crate provides a convenient builder API for creating and sending emails.

pub use mail_core::{Address, Message, Encoding};
pub use mail_smtp::{SmtpClient, SmtpTransport, TlsConfig, Credentials};

/// Re-export for convenience
pub mod prelude {
    pub use crate::{Address, Message, Encoding};
    pub use crate::{SmtpClient, SmtpTransport, TlsConfig, Credentials};
}
