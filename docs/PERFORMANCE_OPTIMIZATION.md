# Performance Optimization Guide

This document outlines performance optimization opportunities and best practices for mail-rs.

## Current Performance Characteristics

### Benchmarks

Run benchmarks with:
```bash
cargo bench
```

Key performance metrics:
- Address creation: ~100ns
- Simple message building: ~500ns
- Message with attachment (1KB): ~2μs
- Message serialization: varies with size
- Base64 encoding (1KB): ~1.5μs
- Quoted-Printable encoding (1KB): ~2μs

## Optimization Opportunities

### 1. String Allocations

#### Current Issues
- Multiple `to_string()` calls in template rendering
- Cloning in `MessageBuilder` methods
- Repeated allocations in encoding operations

#### Recommendations
```rust
// Instead of:
let addr = Address::new(address.into());
let to_addrs = vec![addr];
Self { message: self.message.to(to_addrs) }

// Consider accumulating addresses:
pub fn to(mut self, address: impl Into<String>) -> Self {
    // Accumulate addresses instead of replacing
    self
}
```

### 2. Template Engine Optimization

#### Current Performance Bottlenecks
- Cloning entire variable maps in `render_with_context`
- String allocations for each variable substitution
- Repeated regex matching for variable extraction

#### Recommended Improvements

**Pre-compiled Templates:**
```rust
pub struct CompiledTemplate {
    parts: Vec<TemplatePart>,
}

enum TemplatePart {
    Literal(String),
    Variable(String),
}

impl EmailTemplate {
    pub fn compile(&self) -> CompiledTemplate {
        // Parse template once, reuse multiple times
    }
}
```

**In-place Rendering:**
```rust
// Use a buffer for rendering instead of repeated String operations
pub fn render_to(&self, writer: &mut impl Write, context: &TemplateContext) -> Result<()> {
    // Write directly to output buffer
}
```

### 3. Connection Pool Optimization

#### Current Implementation
Basic connection pooling with timestamp tracking.

#### Recommended Improvements

**Connection Reuse Metrics:**
```rust
pub struct PoolMetrics {
    pub connections_created: usize,
    pub connections_reused: usize,
    pub wait_time_ms: u64,
}
```

**Lazy Connection Creation:**
```rust
// Don't create all connections upfront
// Create on-demand up to max_size
```

**Connection Warmup:**
```rust
pub async fn warmup(&mut self, count: usize) -> Result<()> {
    // Pre-create connections for better performance
}
```

### 4. Message Serialization

#### Current Bottlenecks
- Multiple buffer allocations
- Repeated encoding operations
- No caching of formatted messages

#### Optimizations

**Buffer Pre-allocation:**
```rust
impl Message {
    pub fn format(&self) -> Result<Vec<u8>> {
        // Estimate size and pre-allocate buffer
        let estimated_size = self.estimate_size();
        let mut buffer = Vec::with_capacity(estimated_size);
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }
    
    fn estimate_size(&self) -> usize {
        // Rough estimate: headers + body + attachments
        1024 + self.parts.iter().map(|p| p.content.len()).sum::<usize>()
            + self.attachments.iter().map(|a| a.content.len() * 4 / 3).sum::<usize>() // Base64 expansion
    }
}
```

**Streaming Serialization:**
```rust
// For large messages, write directly to network stream
pub async fn write_to_stream<W: AsyncWrite + Unpin>(
    &self,
    writer: &mut W
) -> Result<()> {
    // Avoid loading entire message in memory
}
```

### 5. Encoding Optimizations

#### Base64 Encoding
Current implementation allocates for each operation.

**Optimization:**
```rust
use base64::{Engine as _, engine::general_purpose};

impl Encoding {
    pub fn encode_to(&self, data: &[u8], output: &mut Vec<u8>) {
        match self {
            Encoding::Base64 => {
                let start_len = output.len();
                output.resize(start_len + data.len() * 4 / 3 + 4, 0);
                let written = general_purpose::STANDARD
                    .encode_slice(data, &mut output[start_len..])
                    .unwrap();
                output.truncate(start_len + written);
            }
            // ...
        }
    }
}
```

### 6. Memory Usage Optimization

#### Attachment Handling
Large attachments can consume significant memory.

**Streaming Attachments:**
```rust
pub struct StreamingAttachment {
    filename: String,
    content_type: String,
    reader: Box<dyn Read + Send>,
}

impl Message {
    pub fn attach_stream(
        mut self,
        filename: impl Into<String>,
        reader: impl Read + Send + 'static
    ) -> Self {
        // Stream attachment during serialization
        // instead of loading into memory
        self
    }
}
```

### 7. Concurrent Operations

#### Parallel Message Building
For bulk operations, process messages in parallel:

```rust
use rayon::prelude::*;

pub fn build_messages_parallel(
    templates: &[EmailTemplate],
    contexts: &[TemplateContext]
) -> Vec<Message> {
    templates.par_iter()
        .zip(contexts.par_iter())
        .map(|(template, context)| {
            // Build message
        })
        .collect()
}
```

## Profiling

### CPU Profiling
```bash
cargo install cargo-flamegraph
cargo flamegraph --bench mail_benchmarks
```

### Memory Profiling
```bash
cargo install cargo-instruments
cargo instruments -t alloc --bench mail_benchmarks
```

### Heap Profiling
```bash
# With valgrind
cargo build --release
valgrind --tool=massif target/release/examples/bulk_send
```

## Performance Testing

### Load Testing
Test with realistic workloads:

```rust
#[test]
fn test_bulk_message_creation() {
    use std::time::Instant;
    
    let start = Instant::now();
    let messages: Vec<_> = (0..1000)
        .map(|i| {
            MessageBuilder::new()
                .from("sender@example.com")
                .to(format!("user{}@example.com", i))
                .subject("Test")
                .text_body("Body")
                .build()
        })
        .collect();
    
    let duration = start.elapsed();
    println!("Created {} messages in {:?}", messages.len(), duration);
    println!("Avg: {:?} per message", duration / messages.len() as u32);
    
    // Assert reasonable performance
    assert!(duration.as_millis() < 100); // Should be < 100ms for 1000 messages
}
```

### Memory Usage Testing
```rust
#[test]
fn test_memory_efficient_large_attachment() {
    // Test that we can handle large attachments without excessive memory
    let size = 10 * 1024 * 1024; // 10MB
    let data = vec![0u8; size];
    
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .text_body("Large file")
        .attachment("large.bin", &data, "application/octet-stream")
        .build();
    
    // Memory usage should be reasonable (not 3-4x the attachment size)
    let formatted = message.format().unwrap();
    let overhead = formatted.len() as f64 / size as f64;
    
    println!("Size overhead: {:.2}x", overhead);
    assert!(overhead < 2.0); // Base64 is 1.33x, total should be < 2x
}
```

## Best Practices

### 1. Reuse Objects
```rust
// Don't create new builder for each message
let mut message = Message::new();
for recipient in recipients {
    message.reset();
    message = message
        .from(sender.clone())
        .to(vec![recipient])
        .subject("Newsletter")
        .body("text/plain", content.clone());
    
    send(&message)?;
}
```

### 2. Batch Operations
```rust
// Send multiple messages in one connection
let mut connection = pool.get_connection().await?;
for message in messages {
    connection.send(&message).await?;
}
drop(connection); // Return to pool
```

### 3. Pre-compile Templates
```rust
// Compile templates once at startup
let templates: HashMap<String, CompiledTemplate> = load_templates()
    .into_iter()
    .map(|(name, tmpl)| (name, tmpl.compile()))
    .collect();

// Reuse compiled templates
let message = templates["welcome"].render(&context)?;
```

### 4. Use Appropriate Encodings
```rust
// For text content, use quoted-printable (smaller)
Message::new()
    .encoding(Encoding::QuotedPrintable)
    .body("text/plain", content)

// For binary content, use base64
Message::new()
    .encoding(Encoding::Base64)
    .attach("image.png", data)
```

### 5. Profile Before Optimizing
Always measure before optimizing:
```bash
# Run benchmarks to establish baseline
cargo bench --baseline baseline

# Make changes

# Compare results
cargo bench --baseline baseline
```

## Monitoring

### Runtime Metrics
```rust
use std::time::Instant;

pub struct PerformanceMonitor {
    message_build_times: Vec<Duration>,
    send_times: Vec<Duration>,
}

impl PerformanceMonitor {
    pub fn record_message_build<F, T>(&mut self, f: F) -> T
    where F: FnOnce() -> T
    {
        let start = Instant::now();
        let result = f();
        self.message_build_times.push(start.elapsed());
        result
    }
    
    pub fn report(&self) {
        let avg_build = self.message_build_times.iter().sum::<Duration>() 
            / self.message_build_times.len() as u32;
        println!("Average message build time: {:?}", avg_build);
    }
}
```

## Future Optimizations

1. **Zero-copy Serialization**: Use `bytes` crate for zero-copy buffer management
2. **SIMD Encoding**: Use SIMD instructions for faster Base64/QP encoding
3. **Async Template Rendering**: Allow async operations in templates
4. **Connection Pool Sharding**: Reduce lock contention with sharded pools
5. **Message Caching**: Cache formatted messages for identical content

## Conclusion

The current implementation prioritizes correctness and API ergonomics. For high-performance scenarios:

1. Profile your specific workload
2. Focus on the hottest paths identified by profiling
3. Implement optimizations incrementally
4. Measure impact with benchmarks
5. Balance performance with code maintainability

Most applications will find the current performance more than adequate. Optimize only when profiling indicates a real bottleneck.
