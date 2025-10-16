//! Core types for mail-rs email library
//!
//! This crate provides the fundamental types and traits for building and
//! manipulating email messages.

pub mod error;
pub mod message;
pub mod header;
pub mod encoding;
pub mod address;

pub use error::{Error, Result};
pub use message::Message;
pub use header::Header;
pub use encoding::Encoding;
pub use address::Address;
