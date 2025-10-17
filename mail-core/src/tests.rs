//! Unit tests for mail-core crate

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, Header, Message, MessageBuilder};

    #[test]
    fn test_address_parsing() {
        // Test simple email address
        let addr = Address::new("test@example.com");
        assert_eq!(addr.email(), "test@example.com");
        assert_eq!(addr.name(), None);

        // Test address with name
        let addr = Address::with_name("test@example.com", "Test User");
        assert_eq!(addr.email(), "test@example.com");
        assert_eq!(addr.name(), Some("Test User"));

        // Test address formatting
        let addr = Address::with_name("test@example.com", "Test User");
        assert_eq!(addr.to_string(), "Test User <test@example.com>");

        let addr = Address::new("test@example.com");
        assert_eq!(addr.to_string(), "test@example.com");
    }

    #[test]
    fn test_header_creation() {
        let header = Header::new("X-Custom", "test-value");
        assert_eq!(header.name(), "X-Custom");
        assert_eq!(header.value(), "test-value");
    }

    #[test]
    fn test_message_builder() {
        let from = Address::new("sender@example.com");
        let to = vec![Address::new("recipient@example.com")];
        
        let message = Message::new()
            .from(from.clone())
            .to(to.clone())
            .subject("Test Subject")
            .body("text/plain", "Test body");

        assert_eq!(message.from(), Some(&from));
        assert_eq!(message.to(), &to);
        assert_eq!(message.subject(), Some("Test Subject"));
    }

    #[test]
    fn test_message_with_cc_bcc() {
        let cc = vec![Address::new("cc@example.com")];
        let bcc = vec![Address::new("bcc@example.com")];

        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .cc(cc.clone())
            .bcc(bcc.clone())
            .subject("Test")
            .body("text/plain", "Test");

        assert_eq!(message.cc(), &cc);
        assert_eq!(message.bcc(), &bcc);
    }

    #[test]
    fn test_multipart_message() {
        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("Multipart Test")
            .body("text/plain", "Plain text version")
            .body("text/html", "<h1>HTML version</h1>");

        // Should have both text and HTML parts
        assert!(message.has_text_body());
        assert!(message.has_html_body());
    }

    #[test]
    fn test_attachment() {
        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("With Attachment")
            .body("text/plain", "Please see attachment")
            .attach("document.pdf", b"fake pdf content", "application/pdf");

        assert!(message.has_attachments());
        assert_eq!(message.attachments().len(), 1);
    }

    #[test]
    fn test_embedded_file() {
        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("With Embedded Image")
            .body("text/html", r#"<img src="cid:logo.png">"#)
            .embed("logo.png", b"fake png data", "image/png");

        assert!(message.has_embedded_files());
        assert_eq!(message.embedded_files().len(), 1);
    }

    #[test]
    fn test_custom_headers() {
        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("Test")
            .body("text/plain", "Test")
            .header("X-Custom-Header", "custom-value")
            .header("X-Priority", "1");

        let headers = message.headers();
        assert!(headers.iter().any(|h| h.name() == "X-Custom-Header" && h.value() == "custom-value"));
        assert!(headers.iter().any(|h| h.name() == "X-Priority" && h.value() == "1"));
    }

    #[test]
    fn test_reply_to() {
        let reply_to = Address::new("noreply@example.com");
        let message = Message::new()
            .from(Address::new("sender@example.com"))
            .reply_to(reply_to.clone())
            .to(vec![Address::new("recipient@example.com")])
            .subject("Test")
            .body("text/plain", "Test");

        assert_eq!(message.reply_to(), Some(&reply_to));
    }

    #[test]
    fn test_encoding_functions() {
        use crate::encoding::*;

        // Test base64 encoding
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());

        // Test quoted-printable encoding
        let text = "Hello, World! = Special chars: ñáéíóú";
        let encoded = quoted_printable_encode(text);
        let decoded = quoted_printable_decode(&encoded).unwrap();
        assert_eq!(text, decoded);

        // Test header encoding
        let header = "Subject: Héllö Wörld!";
        let encoded = encode_header(header);
        assert!(encoded.contains("=?UTF-8?"));
    }
}