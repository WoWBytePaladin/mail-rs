# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace setup with edition 2024 and rust-version 1.90.0
- `mail-core` crate with core email types:
  - `Message` - Email message builder
  - `Header` - RFC 5322 header handling
  - `Address` - Email address with display name support
  - `Encoding` - Support for quoted-printable, base64, 8bit, 7bit
  - `Attachment` - File attachment support
  - Multipart message support (alternative, mixed, related)
- `mail-smtp` crate with async SMTP client:
  - TLS support via rustls
  - STARTTLS support
  - PLAIN and LOGIN authentication
  - Connection timeout configuration
  - Async/await API with tokio
- `mail-builder` crate as high-level API
- Comprehensive examples:
  - Simple email sending
  - HTML with attachments
  - Embedded images
  - Bulk sending
  - TLS connections
- MIT License
- Comprehensive README with examples

### Features
- ✅ Send emails via SMTP
- ✅ TLS/STARTTLS encryption
- ✅ Multiple recipients (To, Cc, Bcc)
- ✅ HTML and plain text bodies
- ✅ File attachments
- ✅ Embedded images (inline)
- ✅ Multipart messages
- ✅ RFC 5322 compliant headers
- ✅ Quoted-printable and base64 encoding
- ✅ Builder pattern API
- ✅ Async/await support

## [0.1.0] - 2025-10-16

### Added
- Initial release
- Core functionality for building and sending emails
- SMTP client with TLS support
- Authentication mechanisms (PLAIN, LOGIN)
- Examples and documentation
