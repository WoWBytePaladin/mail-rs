//! # Mail-RS: A Comprehensive Rust Email Library
//! 
//! Mail-RS is a modern, async-first email library for Rust that provides a clean and
//! intuitive API for building, sending, and parsing emails.
//! 
//! ## Features
//! 
//! - **Simple API**: Easy-to-use builder pattern for constructing emails
//! - **Async/Await Support**: Built with async/await from the ground up
//! - **SMTP Client**: Full-featured SMTP client with TLS/STARTTLS support
//! - **MIME Support**: Complete MIME message generation and parsing
//! - **Attachments**: Support for file attachments and embedded images
//! - **Templates**: Simple template system for dynamic content
//! - **Performance**: Optimized for high-throughput email sending
//! 
//! ## Quick Start
//! 
//! ```rust
//! use mail_rs::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Build an email
//!     let message = MessageBuilder::new()
//!         .from("sender@example.com")
//!         .to("recipient@example.com")
//!         .subject("Hello from Mail-RS!")
//!         .text_body("This is a plain text email.")
//!         .html_body("<h1>This is an HTML email</h1>")
//!         .attachment("document.pdf", include_bytes!("document.pdf"), "application/pdf")
//!         .build();
//! 
//!     // Configure SMTP
//!     let transport = SmtpTransport::new("smtp.example.com", 587)
//!         .with_starttls(TlsConfig::new());
//! 
//!     let client = SmtpClient::new(transport)
//!         .credentials(Credentials::new("username", "password"));
//! 
//!     // Send the email
//!     client.send(&message).await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Crate Organization
//! 
//! Mail-RS is organized as a workspace with the following crates:
//! 
//! - [`mail-core`]: Core types and functionality (Message, Address, etc.)
//! - [`mail-smtp`]: SMTP client implementation
//! - [`mail-builder`]: High-level builder API for constructing emails
//! 
//! ## Examples
//! 
//! The library includes several examples demonstrating different use cases:
//! 
//! - **Simple Email**: Basic email sending
//! - **HTML with Attachments**: Multipart emails with attachments
//! - **Bulk Sending**: Efficient bulk email sending
//! - **Templates**: Dynamic email content with templates
//! - **Embedded Images**: HTML emails with embedded images
//! - **Newsletter**: Complete newsletter sending system

pub mod prelude {
    //! Convenience re-exports for common types and traits.
    
    pub use mail_core::{Address, Header, Message};
    pub use mail_smtp::{SmtpClient, SmtpTransport, TlsConfig, Credentials};
    pub use mail_builder::{MessageBuilder, prelude::*};
}

// Re-export all core functionality
pub use mail_core as core;
pub use mail_smtp as smtp;
pub use mail_builder as builder;

// Common types
pub use mail_core::{Address, Header, Message, Error as MailError, Result as MailResult};
pub use mail_smtp::{SmtpClient, SmtpTransport, TlsConfig, Credentials, Error as SmtpError};
pub use mail_builder::MessageBuilder;