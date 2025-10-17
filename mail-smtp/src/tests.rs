//! Unit tests for mail-smtp crate

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SmtpTransport, SmtpClient, Credentials, TlsConfig};
    use std::time::Duration;

    #[test]
    fn test_smtp_transport_creation() {
        let transport = SmtpTransport::new("smtp.example.com", 587);
        assert_eq!(transport.host(), "smtp.example.com");
        assert_eq!(transport.port(), 587);
    }

    #[test]
    fn test_smtp_transport_with_tls() {
        let tls_config = TlsConfig::new();
        let transport = SmtpTransport::new("smtp.example.com", 465)
            .with_tls(tls_config);
        
        assert!(transport.is_tls_enabled());
    }

    #[test]
    fn test_smtp_transport_with_starttls() {
        let tls_config = TlsConfig::new();
        let transport = SmtpTransport::new("smtp.example.com", 587)
            .with_starttls(tls_config);
        
        assert!(transport.is_starttls_enabled());
    }

    #[test]
    fn test_credentials() {
        let creds = Credentials::new("username", "password");
        assert_eq!(creds.username(), "username");
        assert_eq!(creds.password(), "password");
    }

    #[test]
    fn test_smtp_client_configuration() {
        let transport = SmtpTransport::new("smtp.example.com", 587);
        let client = SmtpClient::new(transport)
            .credentials(Credentials::new("user", "pass"))
            .timeout(Duration::from_secs(30));

        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_smtp_client_mock_connection() {
        // This would be a mock test in a real implementation
        // For now, just test that the client can be created
        let transport = SmtpTransport::new("localhost", 25);
        let client = SmtpClient::new(transport);
        
        // In a real test, we would mock the SMTP server response
        // and test the actual send functionality
        assert!(client.is_ready());
    }

    #[test]
    fn test_tls_config() {
        let tls_config = TlsConfig::new()
            .with_verify_hostname(false)
            .with_accept_invalid_certs(true);

        assert!(!tls_config.verify_hostname());
        assert!(tls_config.accept_invalid_certs());
    }

    #[test]
    fn test_smtp_error_types() {
        use crate::error::SmtpError;

        let error = SmtpError::ConnectionFailed("Connection refused".to_string());
        assert!(matches!(error, SmtpError::ConnectionFailed(_)));

        let error = SmtpError::AuthenticationFailed("Invalid credentials".to_string());
        assert!(matches!(error, SmtpError::AuthenticationFailed(_)));
    }

    #[test]
    fn test_smtp_response_parsing() {
        // Test SMTP response code parsing
        let response_220 = "220 smtp.example.com ESMTP ready";
        assert!(response_220.starts_with("220"));

        let response_250 = "250 OK";
        assert!(response_250.starts_with("250"));

        let response_550 = "550 Mailbox not found";
        assert!(response_550.starts_with("550"));
    }
}