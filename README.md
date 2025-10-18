# mail-rs

[![Rust](https://img.shields.io/badge/rust-1.90.0%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

A modern, async Rust email library inspired by [go-gomail/gomail](https://github.com/go-gomail/gomail).

## Features

- 📧 **Simple and intuitive API** - Easy to use builder pattern for creating emails
- 🚀 **Async/await support** - Built on tokio for high-performance async I/O
- 🔒 **TLS/STARTTLS** - Secure connections with rustls
- 📎 **Attachments & Embedded Images** - Full support for file attachments and inline images
- 🎨 **HTML & Plain Text** - Multipart messages with alternative content types
- 🔐 **Multiple auth mechanisms** - PLAIN and LOGIN authentication
- 📦 **Workspace organization** - Clean modular architecture
- ✅ **Well tested** - Comprehensive test coverage

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
mail-builder = { path = "mail-builder" }
tokio = { version = "1", features = ["full"] }
```

### Simple Email

```rust
use mail_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::new()
        .from(Address::with_name("sender@example.com", "Sender Name"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Hello from mail-rs!")
        .body("text/plain", "This is a test email.");

    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"));

    client.send(&message).await?;
    Ok(())
}
```

### HTML Email with Attachments

```rust
let message = Message::new()
    .from(Address::new("sender@example.com"))
    .to(vec![Address::new("recipient@example.com")])
    .subject("Newsletter")
    .body("text/plain", "Plain text version")
    .alternative("text/html", "<h1>HTML version</h1>")
    .attach("report.pdf", pdf_data)
    .attach("image.jpg", image_data);
```

### Embedded Images

```rust
let html = r#"<img src="cid:logo.png" alt="Logo" />"#;

let message = Message::new()
    .from(Address::new("sender@example.com"))
    .to(vec![Address::new("recipient@example.com")])
    .subject("Welcome!")
    .body("text/html", html)
    .embed("logo.png", logo_data);
```

### Connection Pool for High Performance

```rust
use mail_smtp::{SmtpPool, PoolConfig};

let pool_config = PoolConfig {
    max_connections: 10,
    min_connections: 2,
    max_idle_time: Duration::from_secs(300),
    ..Default::default()
};

let pool = SmtpPool::with_config(transport, pool_config)
    .credentials(credentials);

// Send emails efficiently using pooled connections
pool.send(&message).await?;
```

### Retry Logic with Exponential Backoff

```rust
use mail_smtp::{RetryableSmtpClient, RetryConfig};

let retry_config = RetryConfig {
    max_attempts: 5,
    initial_delay: Duration::from_millis(100),
    backoff_multiplier: 2.0,
    retry_on_server_error: true,
    ..Default::default()
};

let retryable_client = RetryableSmtpClient::with_config(client, retry_config);
retryable_client.send(&message).await?; // Automatically retries on failure
```

### Rate Limiting

```rust
use mail_smtp::{RateLimitedSmtpClient, RateLimitConfig};

let rate_config = RateLimitConfig {
    max_emails: 100,
    time_window: Duration::from_secs(60), // 100 emails per minute
    sliding_window: true,
    ..Default::default()
};

let rate_limited_client = RateLimitedSmtpClient::with_config(client, rate_config);
rate_limited_client.send(&message).await?; // Respects rate limits
```

### Template Engine

```rust
use mail_builder::{EmailTemplate, TemplateContext, CommonTemplates};

// Use built-in templates
let template = CommonTemplates::welcome()
    .from("welcome@{{company_domain}}")
    .to("{{user_email}}");

let context = TemplateContext::new()
    .set("company_name", "My Company")
    .set("company_domain", "mycompany.com")
    .set("user_name", "John Doe")
    .set("user_email", "john@example.com")
    .set("app_url", "https://app.mycompany.com")
    .set("support_email", "support@mycompany.com");

let message = template.render(&context)?;

// Or create custom templates
let custom_template = EmailTemplate::new()
    .subject("Welcome {{user_name}} to {{company_name}}")
    .text_body("Hi {{user_name}}, welcome to our platform!")
    .html_body("<h1>Welcome {{user_name}}</h1><p>Thanks for joining {{company_name}}!</p>");
```

## Architecture

The library is organized as a cargo workspace with three crates:

- **mail-core** - Core types (Message, Address, Header, Encoding)
- **mail-smtp** - SMTP client implementation with TLS support
- **mail-builder** - High-level builder API (recommended for most users)

```
mail-rs/
├── mail-core/       # Core email types and RFC implementations
├── mail-smtp/       # Async SMTP client with TLS
├── mail-builder/    # High-level builder API
└── examples/        # Usage examples
```

## Examples

The `mail-builder/examples/` directory contains several examples:

- `simple.rs` - Basic email sending
- `html_with_attachments.rs` - HTML email with files
- `embedded_images.rs` - Inline images in HTML
- `bulk_send.rs` - Sending multiple emails efficiently
- `tls_connection.rs` - Direct TLS connection (port 465)
- `connection_pool.rs` - Using connection pools for performance
- `retry_logic.rs` - Implementing retry mechanisms
- `rate_limiting.rs` - Rate limiting email sending
- `template_engine.rs` - Dynamic email templates with variables
- `cram_md5_auth.rs` - CRAM-MD5 authentication with fallback

Run an example:

```bash
cargo run --example simple
```

## Configuration

### SMTP Servers

#### Gmail
```rust
let transport = SmtpTransport::new("smtp.gmail.com", 587)
    .with_starttls(TlsConfig::new());
// Use App Password, not regular password
```

#### Outlook/Office365
```rust
let transport = SmtpTransport::new("smtp-mail.outlook.com", 587)
    .with_starttls(TlsConfig::new());
```

#### Custom SMTP
```rust
// STARTTLS (port 587)
let transport = SmtpTransport::new("smtp.example.com", 587)
    .with_starttls(TlsConfig::new());

// Direct TLS (port 465)
let transport = SmtpTransport::new("smtp.example.com", 465)
    .with_tls(TlsConfig::new());
```

### Insecure Connections (Testing Only)

⚠️ **Never use in production!**

```rust
let tls_config = TlsConfig::danger_accept_invalid_certs();
let transport = SmtpTransport::new("localhost", 587)
    .with_starttls(tls_config);
```

## API Overview

### Message Builder

```rust
Message::new()
    // Headers
    .from(Address)
    .to(Vec<Address>)
    .cc(Vec<Address>)
    .bcc(Vec<Address>)
    .subject(String)
    .header(name, value)
    
    // Body
    .body(content_type, content)
    .alternative(content_type, content)
    
    // Attachments
    .attach(filename, data)
    .attach_file(path)
    .embed(filename, data)
    .embed_file(path)
    
    // Settings
    .charset(charset)
    .encoding(Encoding)
    .date_now()
```

### Address Types

```rust
// Simple email
Address::new("user@example.com")

// With display name
Address::with_name("user@example.com", "John Doe")

// From string
Address::from("user@example.com")
```

### Encoding Options

```rust
Encoding::QuotedPrintable  // Default, good for text
Encoding::Base64           // For binary data
Encoding::EightBit         // No encoding
Encoding::SevenBit         // ASCII only
```

## Requirements

- Rust 1.90.0 or later
- Edition 2024
- tokio runtime for async operations

## Testing

Run tests for all crates:

```bash
cargo test --workspace
```

Run tests for a specific crate:

```bash
cargo test -p mail-core
cargo test -p mail-smtp
cargo test -p mail-builder
```

## Comparison with go-gomail

This library is inspired by go-gomail but adapted for Rust idioms:

| Feature | go-gomail | mail-rs |
|---------|-----------|---------|
| Language | Go | Rust |
| Async | Channels | async/await |
| TLS | Go stdlib | rustls |
| API Style | Methods | Builder pattern |
| Type Safety | Runtime | Compile-time |
| Memory | GC | Zero-copy where possible |

## Roadmap

- [x] Core message types
- [x] SMTP client with TLS
- [x] Authentication (PLAIN, LOGIN, CRAM-MD5)
- [x] Attachments and embedded files
- [x] Multipart messages
- [x] Connection pooling
- [x] Retry logic
- [x] Rate limiting
- [x] Template support
- [x] **DKIM signing** ✨
- [x] **S/MIME support** ✨
- [x] **OAuth2 authentication** ✨
- [x] **Advanced template system** ✨
- [x] **Enhanced connection pool** ✨

## Documentation

For detailed information about the new features:

- 📚 **[Enhanced Features Guide](docs/ENHANCED_FEATURES.md)** - Comprehensive documentation for all advanced features
- 📋 **[Implementation Summary](docs/IMPLEMENTATION_SUMMARY.md)** - Complete implementation details and statistics

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [go-gomail/gomail](https://github.com/go-gomail/gomail)
- Built with [tokio](https://tokio.rs/) for async runtime
- Uses [rustls](https://github.com/rustls/rustls) for TLS

## Resources

- [RFC 5321](https://tools.ietf.org/html/rfc5321) - SMTP Protocol
- [RFC 5322](https://tools.ietf.org/html/rfc5322) - Internet Message Format
- [RFC 2045](https://tools.ietf.org/html/rfc2045) - MIME Part One
- [RFC 2047](https://tools.ietf.org/html/rfc2047) - MIME Part Three (Header Extensions)
- [RFC 3207](https://tools.ietf.org/html/rfc3207) - SMTP Service Extension for STARTTLS

---

**Note**: This is a learning project and should be thoroughly tested before production use.
