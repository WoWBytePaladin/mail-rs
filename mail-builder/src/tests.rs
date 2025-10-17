//! Unit tests for mail-builder crate

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_message_builder_basic() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test Subject")
            .text_body("Plain text content")
            .build();

        assert_eq!(message.from().unwrap().email(), "sender@example.com");
        assert_eq!(message.to()[0].email(), "recipient@example.com");
        assert_eq!(message.subject(), Some("Test Subject"));
    }

    #[test]
    fn test_message_builder_with_names() {
        let message = MessageBuilder::new()
            .from_with_name("sender@example.com", "John Doe")
            .to_with_name("recipient@example.com", "Jane Smith")
            .subject("Test")
            .text_body("Hello")
            .build();

        assert_eq!(message.from().unwrap().name(), Some("John Doe"));
        assert_eq!(message.to()[0].name(), Some("Jane Smith"));
    }

    #[test]
    fn test_message_builder_multipart() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Multipart Message")
            .text_body("Plain text version")
            .html_body("<h1>HTML version</h1>")
            .build();

        assert!(message.has_text_body());
        assert!(message.has_html_body());
    }

    #[test]
    fn test_message_builder_attachments() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("With Attachment")
            .text_body("Please see attachment")
            .attachment("document.txt", b"File content", "text/plain")
            .build();

        assert!(message.has_attachments());
        assert_eq!(message.attachments().len(), 1);
    }

    #[test]
    fn test_message_builder_embedded_images() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("With Image")
            .html_body(r#"<img src="cid:logo">"#)
            .embedded_image("logo", b"image data", "image/png")
            .build();

        assert!(message.has_embedded_files());
        assert_eq!(message.embedded_files().len(), 1);
    }

    #[test]
    fn test_message_builder_cc_bcc() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .cc("cc@example.com")
            .bcc("bcc@example.com")
            .subject("Test")
            .text_body("Test")
            .build();

        assert_eq!(message.cc().len(), 1);
        assert_eq!(message.bcc().len(), 1);
        assert_eq!(message.cc()[0].email(), "cc@example.com");
        assert_eq!(message.bcc()[0].email(), "bcc@example.com");
    }

    #[test]
    fn test_message_builder_multiple_recipients() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient1@example.com")
            .to("recipient2@example.com")
            .cc("cc1@example.com")
            .cc("cc2@example.com")
            .subject("Multiple Recipients")
            .text_body("Hello everyone")
            .build();

        assert_eq!(message.to().len(), 2);
        assert_eq!(message.cc().len(), 2);
    }

    #[test]
    fn test_message_builder_headers() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test")
            .text_body("Test")
            .header("X-Priority", "1")
            .header("X-Custom", "value")
            .build();

        let headers = message.headers();
        assert!(headers.iter().any(|h| h.name() == "X-Priority" && h.value() == "1"));
        assert!(headers.iter().any(|h| h.name() == "X-Custom" && h.value() == "value"));
    }

    #[test]
    fn test_message_builder_reply_to() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .reply_to("noreply@example.com")
            .to("recipient@example.com")
            .subject("Test")
            .text_body("Test")
            .build();

        assert_eq!(message.reply_to().unwrap().email(), "noreply@example.com");
    }

    #[test]
    fn test_message_builder_date() {
        use chrono::{DateTime, Utc};
        
        let now = Utc::now();
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test")
            .text_body("Test")
            .date(now)
            .build();

        // The message should have the date set
        assert!(message.date().is_some());
    }

    #[test]
    fn test_message_builder_from_file() {
        // Test reading attachment from file (mock)
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("File Attachment")
            .text_body("See attachment")
            .attachment_from_data("test.txt", b"Test file content".to_vec(), "text/plain")
            .build();

        assert!(message.has_attachments());
    }

    #[test]
    fn test_address_parsing_in_builder() {
        let message = MessageBuilder::new()
            .from("John Doe <john@example.com>")
            .to("Jane Smith <jane@example.com>")
            .subject("Parsed Addresses")
            .text_body("Test")
            .build();

        assert_eq!(message.from().unwrap().email(), "john@example.com");
        assert_eq!(message.from().unwrap().name(), Some("John Doe"));
        assert_eq!(message.to()[0].email(), "jane@example.com");
        assert_eq!(message.to()[0].name(), Some("Jane Smith"));
    }
}