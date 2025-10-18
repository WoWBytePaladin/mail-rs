# Enhanced Features - mail-rs

This document describes the advanced features that have been implemented in the mail-rs library.

## Table of Contents

- [DKIM Signing](#dkim-signing)
- [S/MIME Support](#smime-support)
- [OAuth2 Authentication](#oauth2-authentication)
- [Advanced Template System](#advanced-template-system)
- [Enhanced Connection Pool](#enhanced-connection-pool)

## DKIM Signing

DomainKeys Identified Mail (DKIM) is an email authentication method that allows recipients to verify that an email was indeed sent and authorized by the owner of that domain.

### Features

- RSA-SHA256 signing algorithm
- Simple and relaxed canonicalization for headers and body
- Configurable signing domains and selectors
- Automatic header selection for signing
- Integration with existing Message API

### Usage Example

```rust
use mail_core::dkim::{DkimSigner, DkimConfig, CanonicalizationAlgorithm};

// Create DKIM configuration
let config = DkimConfig {
    domain: "example.com".to_string(),
    selector: "default".to_string(),
    private_key_path: "private_key.pem".to_string(),
    canonicalization: (
        CanonicalizationAlgorithm::Relaxed,
        CanonicalizationAlgorithm::Relaxed,
    ),
    headers_to_sign: vec![
        "from".to_string(),
        "to".to_string(),
        "subject".to_string(),
    ],
};

// Create signer and sign message
let signer = DkimSigner::new(config)?;
let signed_message = signer.sign_message(&message)?;
```

See `mail-builder/examples/dkim_signing.rs` for a complete example.

## S/MIME Support

S/MIME (Secure/Multipurpose Internet Mail Extensions) provides encryption and digital signatures for email messages.

### Features

- Email encryption with recipient certificates
- Digital signature generation
- Certificate management
- PKCS#7 message format support
- Integration with standard certificate stores

### Usage Example

```rust
use mail_core::smime::{SmimeSigner, SmimeConfig};

// Configure S/MIME
let config = SmimeConfig {
    certificate_path: "cert.pem".to_string(),
    private_key_path: "key.pem".to_string(),
    recipient_certs: vec!["recipient_cert.pem".to_string()],
};

let signer = SmimeSigner::new(config)?;

// Sign a message
let signed_message = signer.sign(&message)?;

// Encrypt a message
let encrypted_message = signer.encrypt(&message)?;
```

See `mail-builder/examples/smime_example.rs` for a complete example.

## OAuth2 Authentication

Modern SMTP authentication using OAuth2 tokens for services like Gmail and Outlook.

### Features

- OAuth2 token-based authentication
- Automatic token refresh
- Support for Gmail and Outlook providers
- XOAUTH2 SASL mechanism
- Secure token storage
- Scope management

### Usage Example

```rust
use mail_smtp::oauth2::{OAuth2Client, OAuth2Provider, OAuth2Config};

// Configure OAuth2 for Gmail
let config = OAuth2Config {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    redirect_uri: "http://localhost:8080/callback".to_string(),
    auth_url: "https://accounts.google.com/o/oauth2/auth".to_string(),
    token_url: "https://oauth2.googleapis.com/token".to_string(),
    scopes: vec!["https://mail.google.com/".to_string()],
};

let oauth_client = OAuth2Client::new(config);

// Get authorization URL
let auth_url = oauth_client.get_authorization_url();

// Exchange code for token
let token = oauth_client.exchange_code("authorization-code").await?;

// Use with SMTP client
let client = SmtpClient::new(transport)
    .oauth2_credentials(token.access_token);
```

See `mail-builder/examples/oauth2_smtp.rs` for a complete example.

## Advanced Template System

A powerful template engine for generating dynamic email content with Handlebars-like syntax.

### Features

- Variable substitution: `{{variable}}`
- Conditional rendering: `{{#if condition}}...{{/if}}`
- Loops: `{{#each items}}...{{/each}}`
- Partials: `{{> header}}`
- Custom helpers
- Built-in helpers: `uppercase`, `lowercase`, `capitalize`, `format_date`, `format_number`
- JSON context support
- Nested data structures

### Usage Example

```rust
use mail_builder::advanced_template::{AdvancedTemplateEngine, AdvancedTemplateContext};
use serde_json::json;

// Create template engine
let mut engine = AdvancedTemplateEngine::new();

// Define email template
let template = r#"
Hello {{user.name}},

{{#if user.premium}}
Thank you for being a premium member!
{{/if}}

Your items:
{{#each items}}
  - {{this.name}}: ${{this.price}}
{{/each}}

Total: ${{total}}

Best regards,
{{> footer}}
"#;

// Create context
let context = json!({
    "user": {
        "name": "John Doe",
        "premium": true
    },
    "items": [
        {"name": "Product A", "price": 29.99},
        {"name": "Product B", "price": 49.99}
    ],
    "total": 79.98
});

// Render template
let email_body = engine.render(template, &context)?;
```

See `mail-builder/examples/template_engine.rs` and `mail-builder/examples/advanced_template.rs` for complete examples.

## Enhanced Connection Pool

An advanced SMTP connection pool with enterprise-grade features for high-volume email sending.

### Features

- **Dynamic Connection Management**: Automatically scales between min and max connections
- **Circuit Breaker Pattern**: Prevents cascading failures with automatic recovery
- **Health Monitoring**: Periodic health checks and connection validation
- **Connection Lifecycle**: Automatic rotation based on age and idle time
- **Comprehensive Metrics**: Detailed tracking of pool performance
- **Retry Logic**: Configurable retry attempts with exponential backoff
- **Fault Tolerance**: Graceful degradation under load
- **Thread-Safe**: Concurrent access from multiple threads

### Configuration

```rust
use mail_smtp::enhanced_pool::{EnhancedSmtpPool, EnhancedPoolConfig};
use std::time::Duration;

let config = EnhancedPoolConfig {
    max_connections: 20,
    min_connections: 5,
    max_idle_time: Duration::from_secs(300),
    connection_timeout: Duration::from_secs(30),
    acquire_timeout: Duration::from_secs(10),
    health_check_interval: Duration::from_secs(60),
    circuit_breaker_failure_threshold: 5,
    circuit_breaker_recovery_timeout: Duration::from_secs(60),
    validate_connections: true,
    max_connection_age: Duration::from_secs(3600),
    max_retry_attempts: 3,
    retry_backoff_multiplier: 2.0,
};

let transport = SmtpTransport::new("smtp.example.com", 587);
let pool = EnhancedSmtpPool::with_config(transport, config);
```

### Usage Example

```rust
use mail_smtp::enhanced_pool::EnhancedSmtpPool;

// Create pool
let pool = EnhancedSmtpPool::new(transport);

// Send email through pool
pool.send(&message).await?;

// Get metrics
let metrics = pool.metrics();
println!("Success rate: {:.2}%", metrics.success_rate() * 100.0);
println!("Active connections: {}", metrics.active_connections);
println!("Circuit breaker trips: {}", metrics.circuit_breaker_trips);
```

### Metrics

The pool provides comprehensive metrics:

- `active_connections`: Currently in-use connections
- `idle_connections`: Available connections in the pool
- `total_connections()`: Total managed connections
- `successful_sends`: Number of successful email sends
- `failed_sends`: Number of failed email sends
- `success_rate()`: Success rate as a percentage
- `connections_created`: Lifetime connection creation count
- `connections_destroyed`: Lifetime connection destruction count
- `acquire_timeouts`: Number of connection acquisition timeouts
- `avg_connection_age_ms`: Average connection age in milliseconds
- `health_check_failures`: Failed health checks
- `circuit_breaker_trips`: Number of times circuit breaker opened

### Circuit Breaker

The circuit breaker protects your system from cascading failures:

1. **Closed State**: Normal operation, requests pass through
2. **Open State**: After threshold failures, immediately rejects requests
3. **Half-Open State**: After recovery timeout, allows test requests

See `mail-builder/examples/enhanced_pool.rs` for a complete example.

## Examples

All features include comprehensive examples in the `mail-builder/examples/` directory:

- `dkim_signing.rs` - DKIM email signing
- `smime_example.rs` - S/MIME encryption and signatures
- `oauth2_smtp.rs` - OAuth2 SMTP authentication
- `template_engine.rs` - Basic template usage
- `advanced_template.rs` - Advanced template features
- `enhanced_pool.rs` - Connection pool with monitoring
- `connection_pool.rs` - Basic connection pooling
- `rate_limiting.rs` - Rate limiting for bulk sends
- `retry_logic.rs` - Retry mechanisms

## Running Examples

To run any example:

```bash
cargo run --package mail-builder --example <example_name>
```

For example:

```bash
cargo run --package mail-builder --example enhanced_pool
cargo run --package mail-builder --example template_engine
cargo run --package mail-builder --example oauth2_smtp
```

## Integration

All features are designed to work together seamlessly:

```rust
use mail_builder::MessageBuilder;
use mail_core::dkim::DkimSigner;
use mail_smtp::{
    transport::SmtpTransport,
    enhanced_pool::EnhancedSmtpPool,
    oauth2::OAuth2Client,
};

// Build message with template
let message = MessageBuilder::new()
    .from("sender@example.com")
    .to("recipient@example.com")
    .subject("Hello!")
    .html_body(template_engine.render(template, &context)?)
    .build();

// Sign with DKIM
let signed_message = dkim_signer.sign_message(&message)?;

// Send through enhanced pool with OAuth2
let transport = SmtpTransport::new("smtp.gmail.com", 587);
let pool = EnhancedSmtpPool::new(transport)
    .credentials(oauth2_token);

pool.send(&signed_message).await?;
```

## Contributing

When contributing to these features, please ensure:

1. All tests pass: `cargo test`
2. Examples compile: `cargo check --examples`
3. Documentation is updated
4. Code follows Rust best practices

## License

This project is licensed under the MIT License - see the LICENSE file for details.
