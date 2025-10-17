use thiserror::Error;

/// Result type for SMTP operations
pub type Result<T> = std::result::Result<T, Error>;

/// SMTP client errors
#[derive(Debug, Error)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("SMTP command failed: {0}")]
    Command(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Timeout")]
    Timeout,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Mail core error: {0}")]
    MailCore(#[from] mail_core::Error),

    #[error("{0}")]
    Custom(String),
}
