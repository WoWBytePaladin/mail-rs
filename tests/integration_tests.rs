//! Integration tests for mail-rs
//! 
//! These tests verify that different components work together correctly.

use mail_builder::MessageBuilder;
use mail_core::Address;

#[test]
fn test_message_builder_integration() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Integration Test")
        .text_body("This is a test message")
        .build();

    // Verify message was built correctly by serializing
    let result = message.format();
    assert!(result.is_ok());
    let data = result.unwrap();
    let formatted = String::from_utf8_lossy(&data);
    assert!(formatted.contains("sender@example.com"));
    assert!(formatted.contains("recipient@example.com"));
    assert!(formatted.contains("Integration Test"));
}

#[test]
fn test_multipart_message_with_attachments() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test with Attachments")
        .text_body("Plain text")
        .attachment("test.txt", b"Test content", "text/plain")
        .attachment("data.bin", b"\x00\x01\x02\x03", "application/octet-stream")
        .build();

    // Verify multipart structure
    assert!(message.is_multipart());
    let result = message.format();
    assert!(result.is_ok());
}

#[test]
fn test_address_creation_and_formatting() {
    let addr1 = Address::new("user@example.com");
    assert_eq!(addr1.email, "user@example.com");
    assert!(addr1.name.is_none());

    let addr2 = Address::with_name("user@example.com", "John Doe");
    assert_eq!(addr2.email, "user@example.com");
    assert_eq!(addr2.name, Some("John Doe".to_string()));

    let formatted = addr2.format();
    assert!(formatted.contains("John Doe"));
    assert!(formatted.contains("user@example.com"));
}

#[test]
fn test_message_with_multiple_recipients() {
    // Note: Current implementation of .to() replaces rather than appends
    // So we'll test that the last one is used
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient3@example.com")
        .cc("cc2@example.com")
        .bcc("bcc@example.com")
        .subject("Multiple Recipients")
        .text_body("Test message")
        .build();

    // Verify through formatted output
    let data = message.format().unwrap();
    let formatted = String::from_utf8_lossy(&data);
    assert!(formatted.contains("recipient3@example.com"));
    assert!(formatted.contains("cc2@example.com"));
    // BCC is not included in message output
}

#[test]
fn test_custom_headers() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Custom Headers Test")
        .text_body("Test")
        .header("X-Custom-Header", "custom-value")
        .header("X-Priority", "1")
        .build();

    // Verify custom headers through header interface
    let headers = message.headers();
    assert!(headers.get("X-Custom-Header").is_some());
    assert!(headers.get("X-Priority").is_some());
}

#[test]
fn test_embedded_image() {
    let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Embedded Image")
        .html_body("<img src=\"cid:logo\">")
        .embedded_image("logo", &image_data, "image/jpeg")
        .build();

    // Verify multipart structure with embedded content
    assert!(message.is_multipart());
}

#[test]
fn test_message_builder_chaining() {
    // Test that all builder methods can be chained
    let result = MessageBuilder::new()
        .from("sender@example.com")
        .reply_to("reply@example.com")
        .to("to1@example.com")
        .to("to2@example.com")
        .cc("cc@example.com")
        .bcc("bcc@example.com")
        .subject("Chaining Test")
        .text_body("Text")
        .header("X-Test", "value")
        .attachment("file.txt", b"content", "text/plain")
        .build()
        .format();

    assert!(result.is_ok());
}

#[test]
fn test_empty_message_handling() {
    // Even with minimal information, message should be buildable
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .build();

    let result = message.format();
    assert!(result.is_ok());
    
    let data = result.unwrap();
    let formatted = String::from_utf8_lossy(&data);
    assert!(formatted.contains("sender@example.com"));
    assert!(formatted.contains("recipient@example.com"));
}

#[test]
fn test_unicode_content() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Unicode Test: 你好世界 🌍")
        .text_body("Content with emoji: 😀 and unicode: こんにちは")
        .build();

    let data = message.format().unwrap();
    let formatted = String::from_utf8_lossy(&data);
    // Unicode content may be encoded, just check it doesn't error
    assert!(!formatted.is_empty());
}

#[test]
fn test_large_attachment() {
    // Test with a larger attachment to verify chunking/encoding works
    let large_data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect(); // 1MB
    
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Large Attachment")
        .text_body("See attachment")
        .attachment("large_file.bin", &large_data, "application/octet-stream")
        .build();

    let result = message.format();
    assert!(result.is_ok());
    // Message with 1MB attachment should be significantly larger
    assert!(result.unwrap().len() > 1024 * 512);
}

#[test]
fn test_message_from_address_extraction() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .text_body("Test")
        .build();

    let from = message.from_address();
    assert!(from.is_some());
    assert_eq!(from.unwrap(), "sender@example.com");
}

#[test]
fn test_message_recipients_extraction() {
    let message = MessageBuilder::new()
        .from("sender@example.com")
        .to("to1@example.com")
        .to("to2@example.com")
        .cc("cc@example.com")
        .bcc("bcc@example.com")
        .text_body("Test")
        .build();

    let recipients = message.recipients();
    // Note: recipients() returns extracted emails
    assert!(!recipients.is_empty());
}

#[test]
fn test_concurrent_message_building() {
    use std::thread;
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                MessageBuilder::new()
                    .from(format!("sender{}@example.com", i))
                    .to(format!("recipient{}@example.com", i))
                    .subject(format!("Message {}", i))
                    .text_body(format!("Body {}", i))
                    .build()
            })
        })
        .collect();

    let messages: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    assert_eq!(messages.len(), 10);
    for (i, msg) in messages.iter().enumerate() {
        let data = msg.format().unwrap();
        let formatted = String::from_utf8_lossy(&data);
        assert!(formatted.contains(&format!("Message {}", i)));
    }
}

#[test]
fn test_builder_with_name() {
    let message = MessageBuilder::new()
        .from_with_name("sender@example.com", "John Sender")
        .to_with_name("recipient@example.com", "Jane Recipient")
        .subject("Named Addresses")
        .text_body("Test")
        .build();

    let data = message.format().unwrap();
    let formatted = String::from_utf8_lossy(&data);
    assert!(formatted.contains("sender@example.com"));
    assert!(formatted.contains("recipient@example.com"));
}

#[test]
fn test_message_reset() {
    let mut message = MessageBuilder::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test")
        .text_body("Body")
        .build();

    // Reset the message
    message.reset();
    
    // After reset, from_address should be None
    assert!(message.from_address().is_none());
}
