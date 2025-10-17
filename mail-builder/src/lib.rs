//! High-level email builder and sender
//!
//! This crate provides a convenient builder API for creating and sending emails.

mod builder;

pub use builder::MessageBuilder;
pub use mail_core::{Address, Message, Encoding, Header, Error, Result};
pub use mail_smtp::{SmtpClient, SmtpTransport, TlsConfig, Credentials, AuthMechanism};

/// Re-export for convenience
pub mod prelude {
    pub use crate::MessageBuilder;
    pub use crate::{Address, Message, Encoding, Header, Error, Result};
    pub use crate::{SmtpClient, SmtpTransport, TlsConfig, Credentials, AuthMechanism};
}
