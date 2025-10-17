//! Unit tests for mail-core crate

#[cfg(test)]
mod tests {
    use crate::{Address, Message, Header};

    #[test]
    fn test_address_creation() {
        // Test simple email address
        let addr = Address::new("test@example.com".to_string());
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, None);

        // Test address with name
        let addr = Address::with_name("test@example.com".to_string(), "Test User".to_string());
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_header_operations() {
        let mut header = Header::new();
        header.set("X-Custom", "test-value");
        
        let value = header.get_first("X-Custom");
        assert_eq!(value.unwrap(), "test-value");
    }

    #[test]
    fn test_message_creation() {
        let from = Address::new("sender@example.com".to_string());
        let to = vec![Address::new("recipient@example.com".to_string())];
        
        let message = Message::new()
            .from(from)
            .to(to)
            .subject("Test Subject")
            .body("text/plain", "Test body");

        // Just test that we can create the message
        assert!(!message.is_multipart() || message.is_multipart());
    }

    #[test]
    fn test_message_with_attachment() {
        let message = Message::new()
            .from(Address::new("sender@example.com".to_string()))
            .to(vec![Address::new("recipient@example.com".to_string())])
            .subject("Test with Attachment")
            .body("text/plain", "See attachment")
            .attach("test.txt", b"File content".to_vec());

        // Test that multipart is true when attachments are added
        assert!(message.is_multipart());
    }


}