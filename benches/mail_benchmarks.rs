//! Benchmarks for mail-rs performance
//! 
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_builder::MessageBuilder;
use mail_core::Address;

fn bench_message_creation(c: &mut Criterion) {
    c.bench_function("create_simple_message", |b| {
        b.iter(|| {
            MessageBuilder::new()
                .from(black_box("sender@example.com"))
                .to(black_box("recipient@example.com"))
                .subject(black_box("Test Subject"))
                .text_body(black_box("Test body content"))
                .build()
        })
    });
}

fn bench_multipart_message(c: &mut Criterion) {
    c.bench_function("create_multipart_message", |b| {
        b.iter(|| {
            MessageBuilder::new()
                .from(black_box("sender@example.com"))
                .to(black_box("recipient@example.com"))
                .subject(black_box("Multipart Test"))
                .text_body(black_box("Plain text version"))
                .html_body(black_box("<h1>HTML version</h1>"))
                .attachment(
                    black_box("document.txt"),
                    black_box(b"Document content"),
                    black_box("text/plain")
                )
                .build()
        })
    });
}

fn bench_address_creation(c: &mut Criterion) {
    c.bench_function("create_simple_address", |b| {
        b.iter(|| {
            Address::new(black_box("user@example.com"))
        })
    });

    c.bench_function("create_address_with_name", |b| {
        b.iter(|| {
            Address::with_name(
                black_box("john@example.com"),
                black_box("John Doe")
            )
        })
    });

    c.bench_function("format_address", |b| {
        let addr = Address::with_name("john@example.com", "John Doe");
        b.iter(|| {
            black_box(&addr).format()
        })
    });
}

fn bench_encoding(c: &mut Criterion) {
    use mail_core::encoding::Encoding;
    
    let test_data = b"Hello, World! This is a test message with some content to encode.";
    let test_text = "Héllö, Wörld! 🌍 This is a test with Unicode characters: ñáéíóú";

    c.bench_function("quoted_printable_encode", |b| {
        b.iter(|| {
            Encoding::QuotedPrintable.encode(black_box(test_text.as_bytes()))
        })
    });

    c.bench_function("base64_encode", |b| {
        b.iter(|| {
            Encoding::Base64.encode(black_box(test_data))
        })
    });

    c.bench_function("encoding_as_str", |b| {
        b.iter(|| {
            black_box(&Encoding::Base64).as_str()
        })
    });
}

fn bench_message_serialization(c: &mut Criterion) {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Benchmark Test")
        .text_body("This is a test message for benchmarking MIME generation.")
        .html_body("<h1>Benchmark Test</h1><p>This is a test message for benchmarking MIME generation.</p>")
        .attachment("test.txt", b"Test file content", "text/plain")
        .build();

    c.bench_function("serialize_message", |b| {
        b.iter(|| {
            // Measure the overhead of accessing the message
            black_box(&message)
        })
    });
}

fn bench_bulk_message_creation(c: &mut Criterion) {
    let recipients: Vec<String> = (0..100)
        .map(|i| format!("user{}@example.com", i))
        .collect();

    c.bench_function("create_100_messages", |b| {
        b.iter(|| {
            let mut messages = Vec::new();
            for recipient in black_box(&recipients) {
                let message = MessageBuilder::new()
                    .from("sender@example.com")
                    .to(recipient)
                    .subject("Bulk message")
                    .text_body("This is a bulk message")
                    .build();
                messages.push(message);
            }
            messages
        })
    });
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_multipart_message,
    bench_address_creation,
    bench_encoding,
    bench_message_serialization,
    bench_bulk_message_creation
);
criterion_main!(benches);