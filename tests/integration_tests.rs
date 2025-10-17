//! Integration tests for mail-rs
//! 
//! These tests verify that all components work together correctly.

use mail_builder::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_complete_email_workflow() {
    // Create a complex message with all features
    let message = MessageBuilder::new()
        .from_with_name("sender@test.com", "Test Sender")
        .to_with_name("recipient@test.com", "Test Recipient")
        .cc("cc@test.com")
        .bcc("bcc@test.com")
        .reply_to("noreply@test.com")
        .subject("Integration Test Email")
        .text_body("This is the plain text version of the email.")
        .html_body(r#"
            <h1>Integration Test Email</h1>
            <p>This is the <strong>HTML version</strong> of the email.</p>
            <img src="cid:logo" alt="Company Logo">
        "#)
        .attachment("document.txt", b"This is a test document.", "text/plain")
        .embedded_image("logo", include_bytes!("../assets/test-logo.png"), "image/png")
        .header("X-Test-Header", "integration-test")
        .header("X-Priority", "1")
        .build();

    // Verify message structure
    assert!(message.from().is_some());
    assert_eq!(message.to().len(), 1);
    assert_eq!(message.cc().len(), 1);
    assert_eq!(message.bcc().len(), 1);
    assert!(message.reply_to().is_some());
    assert!(message.subject().is_some());
    assert!(message.has_text_body());
    assert!(message.has_html_body());
    assert!(message.has_attachments());
    assert!(message.has_embedded_files());

    // Test SMTP client creation (without actual sending)
    let transport = SmtpTransport::new("smtp.test.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("test", "password"))
        .timeout(Duration::from_secs(30));

    // In a real integration test, you would send to a test SMTP server
    // For now, just verify the client is configured correctly
    assert_eq!(client.timeout(), Duration::from_secs(30));
}

#[test]
fn test_message_serialization() {
    let message = MessageBuilder::new()
        .from("sender@test.com")
        .to("recipient@test.com")
        .subject("Test Message")
        .text_body("Hello, World!")
        .build();

    // Test that the message can be converted to MIME format
    let mime_content = message.to_mime_string();
    
    // Verify basic MIME headers are present
    assert!(mime_content.contains("From: sender@test.com"));
    assert!(mime_content.contains("To: recipient@test.com"));
    assert!(mime_content.contains("Subject: Test Message"));
    assert!(mime_content.contains("Hello, World!"));
}

#[test]
fn test_multipart_message_structure() {
    let message = MessageBuilder::new()
        .from("sender@test.com")
        .to("recipient@test.com")
        .subject("Multipart Test")
        .text_body("Plain text version")
        .html_body("<h1>HTML version</h1>")
        .attachment("file.txt", b"file content", "text/plain")
        .build();

    let mime_content = message.to_mime_string();
    
    // Should be multipart/mixed at the top level
    assert!(mime_content.contains("Content-Type: multipart/mixed"));
    
    // Should contain multipart/alternative for text and HTML
    assert!(mime_content.contains("Content-Type: multipart/alternative"));
    
    // Should contain the attachment
    assert!(mime_content.contains("Content-Disposition: attachment"));
    assert!(mime_content.contains("filename=\"file.txt\""));
}

#[test]
fn test_address_parsing_edge_cases() {
    // Test various address formats
    let test_cases = vec![
        ("simple@example.com", "simple@example.com", None),
        ("Name <email@example.com>", "email@example.com", Some("Name")),
        ("\"Quoted Name\" <email@example.com>", "email@example.com", Some("Quoted Name")),
        ("<email@example.com>", "email@example.com", None),
    ];

    for (input, expected_email, expected_name) in test_cases {
        let addr = Address::parse(input).unwrap();
        assert_eq!(addr.email(), expected_email);
        assert_eq!(addr.name(), expected_name);
    }
}

#[test]
fn test_encoding_roundtrip() {
    use mail_core::encoding::*;

    // Test data with various characters
    let test_strings = vec![
        "Hello, World!",
        "Héllö, Wörld! 🌍",
        "中文测试",
        "Тест на русском",
        "مرحبا بالعالم",
    ];

    for original in test_strings {
        // Test quoted-printable
        let encoded = quoted_printable_encode(original);
        let decoded = quoted_printable_decode(&encoded).unwrap();
        assert_eq!(original, decoded);

        // Test base64 (for bytes)
        let original_bytes = original.as_bytes();
        let encoded_b64 = base64_encode(original_bytes);
        let decoded_b64 = base64_decode(&encoded_b64).unwrap();
        assert_eq!(original_bytes, decoded_b64.as_slice());
    }
}

#[test]
fn test_header_encoding() {
    use mail_core::encoding::encode_header;

    // Test header encoding for non-ASCII characters
    let headers = vec![
        ("Subject: Hello", "Subject: Hello"), // No encoding needed
        ("Subject: Héllö", "Subject: =?UTF-8?B?SMOpbGzDtg==?="),
        ("From: 中文 <test@example.com>", "From: =?UTF-8?B?5Lit5paH?= <test@example.com>"),
    ];

    for (input, expected_pattern) in headers {
        let encoded = encode_header(input);
        if expected_pattern.contains("=?UTF-8?") {
            assert!(encoded.contains("=?UTF-8?"));
        } else {
            assert_eq!(encoded, expected_pattern);
        }
    }
}

#[tokio::test]
async fn test_bulk_sending_workflow() {
    let recipients = vec![
        "user1@test.com",
        "user2@test.com", 
        "user3@test.com",
    ];

    let mut messages = Vec::new();
    
    for recipient in recipients {
        let message = MessageBuilder::new()
            .from("bulk@test.com")
            .to(recipient)
            .subject(format!("Bulk message for {}", recipient))
            .text_body(format!("Hello {}!", recipient))
            .build();
        
        messages.push(message);
    }

    assert_eq!(messages.len(), 3);
    
    // Verify each message is properly constructed
    for (i, message) in messages.iter().enumerate() {
        let expected_recipient = format!("user{}@test.com", i + 1);
        assert_eq!(message.to()[0].email(), expected_recipient);
        assert!(message.subject().unwrap().contains(&expected_recipient));
    }
}

// Mock test for SMTP error handling
#[tokio::test]
async fn test_smtp_error_handling() {
    use mail_smtp::error::SmtpError;

    // Test different error types
    let connection_error = SmtpError::ConnectionFailed("Connection refused".to_string());
    assert!(matches!(connection_error, SmtpError::ConnectionFailed(_)));

    let auth_error = SmtpError::AuthenticationFailed("Invalid credentials".to_string());
    assert!(matches!(auth_error, SmtpError::AuthenticationFailed(_)));

    let timeout_error = SmtpError::Timeout("Operation timed out".to_string());
    assert!(matches!(timeout_error, SmtpError::Timeout(_)));

    // Verify error messages are preserved
    match connection_error {
        SmtpError::ConnectionFailed(msg) => assert_eq!(msg, "Connection refused"),
        _ => panic!("Wrong error type"),
    }
}