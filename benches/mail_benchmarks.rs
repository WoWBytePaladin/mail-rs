//! Benchmarks for mail-rs performance
//! 
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_builder::prelude::*;

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

fn bench_address_parsing(c: &mut Criterion) {
    c.bench_function("parse_simple_address", |b| {
        b.iter(|| {
            Address::parse(black_box("user@example.com"))
        })
    });

    c.bench_function("parse_address_with_name", |b| {
        b.iter(|| {
            Address::parse(black_box("John Doe <john@example.com>"))
        })
    });
}

fn bench_encoding(c: &mut Criterion) {
    use mail_core::encoding::*;
    
    let test_data = b"Hello, World! This is a test message with some content to encode.";
    let test_text = "Héllö, Wörld! 🌍 This is a test with Unicode characters: ñáéíóú";

    c.bench_function("base64_encode", |b| {
        b.iter(|| {
            base64_encode(black_box(test_data))
        })
    });

    c.bench_function("base64_decode", |b| {
        let encoded = base64_encode(test_data);
        b.iter(|| {
            base64_decode(black_box(&encoded))
        })
    });

    c.bench_function("quoted_printable_encode", |b| {
        b.iter(|| {
            quoted_printable_encode(black_box(test_text))
        })
    });

    c.bench_function("quoted_printable_decode", |b| {
        let encoded = quoted_printable_encode(test_text);
        b.iter(|| {
            quoted_printable_decode(black_box(&encoded))
        })
    });
}

fn bench_mime_generation(c: &mut Criterion) {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Benchmark Test")
        .text_body("This is a test message for benchmarking MIME generation.")
        .html_body("<h1>Benchmark Test</h1><p>This is a test message for benchmarking MIME generation.</p>")
        .attachment("test.txt", b"Test file content", "text/plain")
        .build();

    c.bench_function("generate_mime", |b| {
        b.iter(|| {
            black_box(&message).to_mime_string()
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
    bench_address_parsing,
    bench_encoding,
    bench_mime_generation,
    bench_bulk_message_creation
);
criterion_main!(benches);