//! Unit tests for mail-builder crate

#[cfg(test)]
mod tests {
    use crate::MessageBuilder;

    #[test]
    fn test_message_builder_basic() {
        let message = MessageBuilder::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test Subject")
            .text_body("Plain text content")
            .build();

        // Just test that we can build a message
        assert!(message.from_address().is_some());
    }

    #[test]
    fn test_message_builder_construction() {
        let _message = MessageBuilder::new()
            .from_with_name("sender@example.com", "John Doe")
            .to_with_name("recipient@example.com", "Jane Smith")
            .subject("Test")
            .text_body("Hello")
            .build();

        // Test passes if no panics occur
    }
}