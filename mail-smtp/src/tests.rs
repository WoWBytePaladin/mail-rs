//! Unit tests for mail-smtp crate

#[cfg(test)]
mod tests {
    use crate::{SmtpTransport, SmtpClient, Credentials, TlsConfig};
    use std::time::Duration;

    #[test]
    fn test_smtp_transport_creation() {
        let transport = SmtpTransport::new("smtp.example.com", 587);
        assert_eq!(transport.host(), "smtp.example.com");
        assert_eq!(transport.port(), 587);
    }

    #[test]
    fn test_smtp_transport_with_starttls() {
        let tls_config = TlsConfig::new();
        let transport = SmtpTransport::new("smtp.example.com", 587)
            .with_starttls(tls_config);
        
        assert!(transport.is_starttls());
    }

    #[test]
    fn test_credentials() {
        let creds = Credentials::new("username", "password");
        assert_eq!(creds.username, "username");
        assert_eq!(creds.password, "password");
    }

    #[test]
    fn test_smtp_client_configuration() {
        let transport = SmtpTransport::new("smtp.example.com", 587);
        let _client = SmtpClient::new(transport)
            .credentials(Credentials::new("user", "pass"))
            .timeout(Duration::from_secs(30));

        // Test passes if no errors occur during construction
    }

    #[test]
    fn test_tls_config() {
        let _tls_config = TlsConfig::new();
        // Test passes if no errors occur during construction
    }

    #[test]
    fn test_smtp_error_types() {
        use crate::error::Error;

        let error = Error::Custom("Connection refused".to_string());
        assert!(matches!(error, Error::Custom(_)));
    }
}