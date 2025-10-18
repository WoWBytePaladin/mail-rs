# API Documentation Guide

This document provides comprehensive documentation for mail-rs public APIs.

## Core Concepts

### Message Building

The library provides two main ways to build messages:

1. **MessageBuilder** (Recommended) - High-level, fluent API
2. **Message** - Direct construction for advanced use cases

### Authentication

Supported authentication mechanisms:

- **PLAIN**: Simple username/password authentication
- **LOGIN**: Similar to PLAIN but with different encoding
- **CRAM-MD5**: Challenge-response authentication
- **OAuth2**: Modern token-based authentication for Gmail, Outlook, etc.

### Security Features

#### DKIM Signing

```rust
use mail_core::dkim::{DkimSigner, DkimConfig};

let config = DkimConfig {
    domain: "example.com".to_string(),
    selector: "default".to_string(),
    private_key_path: "private_key.pem".to_string(),
    // ... other fields
};

let signer = DkimSigner::new(config)?;
let signed_message = signer.sign_message(&message)?;
```

#### S/MIME Encryption

```rust
use mail_core::smime::{SmimeSigner, SmimeConfig};

let config = SmimeConfig {
    certificate_path: "cert.pem".to_string(),
    private_key_path: "key.pem".to_string(),
    recipient_certificates: vec!["recipient.pem".to_string()],
};

let signer = SmimeSigner::new(config)?;
let encrypted = signer.encrypt(&message)?;
```

### Template System

#### Basic Templates

```rust
use mail_builder::template::{TemplateEngine, TemplateContext};

let engine = TemplateEngine::new();
let mut context = TemplateContext::new();
context.set("name", "John Doe");
context.set("order_id", "12345");

let content = engine.render(template, &context)?;
```

#### Advanced Templates

```rust
use mail_builder::advanced_template::AdvancedTemplateEngine;
use serde_json::json;

let mut engine = AdvancedTemplateEngine::new();

let context = json!({
    "user": {
        "name": "Alice",
        "premium": true
    },
    "items": [
        {"name": "Product A", "price": 29.99},
        {"name": "Product B", "price": 49.99}
    ]
});

let body = engine.render(template, &context)?;
```

Template syntax supports:
- Variables: `{{variable}}`
- Conditionals: `{{#if condition}}...{{/if}}`
- Loops: `{{#each items}}...{{/each}}`
- Partials: `{{> partial_name}}`
- Custom helpers

### Connection Management

#### Basic Connection

```rust
let transport = SmtpTransport::new("smtp.example.com", 587);
let client = SmtpClient::new(transport);
client.send(&message).await?;
```

#### Connection Pool

```rust
use mail_smtp::pool::{SmtpPool, PoolConfig};

let config = PoolConfig {
    max_size: 20,
    min_idle: 5,
    idle_timeout: Duration::from_secs(300),
    connection_timeout: Duration::from_secs(30),
};

let pool = SmtpPool::new(transport, config);
pool.send(&message).await?;
```

#### Enhanced Connection Pool

```rust
use mail_smtp::enhanced_pool::{EnhancedSmtpPool, EnhancedPoolConfig};

let config = EnhancedPoolConfig {
    max_connections: 20,
    min_connections: 5,
    circuit_breaker_failure_threshold: 5,
    health_check_interval: Duration::from_secs(60),
    // ... other configuration
    ..Default::default()
};

let pool = EnhancedSmtpPool::with_config(transport, config);

// Send with automatic retry and circuit breaker
pool.send(&message).await?;

// Get performance metrics
let metrics = pool.metrics();
println!("Success rate: {:.2}%", metrics.success_rate() * 100.0);
```

### Error Handling

The library uses a comprehensive error system:

```rust
use mail_core::{Error, Result};

match client.send(&message).await {
    Ok(_) => println!("Email sent successfully!"),
    Err(Error::Smtp(e)) => eprintln!("SMTP error: {}", e),
    Err(Error::Io(e)) => eprintln!("I/O error: {}", e),
    Err(Error::Tls(e)) => eprintln!("TLS error: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Best Practices

### 1. Use Connection Pooling for Bulk Sending

```rust
// Good: Reuses connections
let pool = SmtpPool::new(transport, config);
for recipient in recipients {
    pool.send(&message).await?;
}

// Not optimal: Creates new connection for each send
for recipient in recipients {
    let client = SmtpClient::new(transport.clone());
    client.send(&message).await?;
}
```

### 2. Implement Rate Limiting

```rust
use mail_smtp::ratelimit::{RateLimiter, RateLimitConfig};

let config = RateLimitConfig {
    max_requests: 100,
    time_window: Duration::from_secs(60),
};

let limiter = RateLimiter::new(config);

for message in messages {
    limiter.wait().await;
    client.send(&message).await?;
}
```

### 3. Use Retry Logic

```rust
use mail_smtp::retry::{RetryConfig, retry_with_backoff};

let retry_config = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(30),
    backoff_multiplier: 2.0,
};

retry_with_backoff(
    || client.send(&message),
    &retry_config
).await?;
```

### 4. Sign Emails with DKIM

```rust
// Always sign outgoing emails for better deliverability
let dkim_signer = DkimSigner::new(dkim_config)?;
let signed_message = dkim_signer.sign_message(&message)?;
client.send(&signed_message).await?;
```

### 5. Use OAuth2 for Modern Email Providers

```rust
use mail_smtp::oauth2::{OAuth2Client, OAuth2Provider};

let oauth_client = OAuth2Client::for_gmail(
    "client_id",
    "client_secret",
    "redirect_uri"
);

let token = oauth_client.get_token().await?;
let client = SmtpClient::new(transport)
    .oauth2_credentials(token.access_token);
```

## Performance Tips

1. **Batch Operations**: Group multiple sends together when possible
2. **Connection Reuse**: Use connection pools instead of creating new connections
3. **Async Operations**: Leverage async/await for concurrent sending
4. **Template Caching**: Cache compiled templates for repeated use
5. **Health Checks**: Enable health checks in connection pools to avoid bad connections

## Security Considerations

1. **TLS/STARTTLS**: Always use encrypted connections
2. **OAuth2**: Prefer OAuth2 over password authentication when available
3. **DKIM Signing**: Sign all outgoing emails
4. **S/MIME**: Use S/MIME for sensitive communications
5. **Credential Storage**: Never hardcode credentials; use environment variables or secret management

## Troubleshooting

### Common Issues

**Authentication Failures**
- Verify credentials are correct
- Check if 2FA is enabled (use app passwords)
- Try different authentication mechanisms

**Connection Timeouts**
- Increase connection timeout in configuration
- Check network connectivity
- Verify SMTP server address and port

**TLS Errors**
- Ensure correct TLS configuration (STARTTLS vs direct TLS)
- Check certificate validity
- Try with TLS verification disabled for testing (not recommended for production)

**Rate Limiting**
- Reduce sending rate
- Implement exponential backoff
- Check provider's rate limits

## API Reference

For detailed API documentation, run:

```bash
cargo doc --no-deps --open
```

This will generate and open the complete API documentation in your browser.

## Examples

All examples are located in `mail-builder/examples/`:

```bash
# Basic examples
cargo run --example simple
cargo run --example html_with_attachments

# Advanced examples
cargo run --example oauth2_smtp
cargo run --example dkim_signing
cargo run --example enhanced_pool
cargo run --example template_engine
```

## Further Reading

- [Enhanced Features Documentation](ENHANCED_FEATURES.md)
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)
- [RFC 5321 - SMTP](https://tools.ietf.org/html/rfc5321)
- [RFC 5322 - Internet Message Format](https://tools.ietf.org/html/rfc5322)
- [RFC 6376 - DKIM](https://tools.ietf.org/html/rfc6376)
