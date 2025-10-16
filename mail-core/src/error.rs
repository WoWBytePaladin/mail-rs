use thiserror::Error;

/// Result type for mail-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when working with email messages
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Quoted-printable decode error: {0}")]
    QuotedPrintableDecode(String),

    #[error("{0}")]
    Custom(String),
}
