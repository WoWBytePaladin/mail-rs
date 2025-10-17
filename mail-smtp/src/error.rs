use thiserror::Error;

/// Result type for SMTP operations
pub type Result<T> = std::result::Result<T, Error>;

/// SMTP client errors
#[derive(Debug, Error, Clone)]
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
    Io(String),

    #[error("Mail core error: {0}")]
    MailCore(String),

    #[error("{0}")]
    Custom(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<mail_core::Error> for Error {
    fn from(err: mail_core::Error) -> Self {
        Error::MailCore(err.to_string())
    }
}
