use std::sync::Arc;
use rustls::ClientConfig;
use webpki_roots;

/// TLS configuration for SMTP connections
#[derive(Clone)]
pub struct TlsConfig {
    config: Arc<ClientConfig>,
    /// Skip certificate verification (insecure, for testing only)
    pub danger_accept_invalid_certs: bool,
}

impl TlsConfig {
    /// Create a new TLS configuration with default settings
    pub fn new() -> Self {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Self {
            config: Arc::new(config),
            danger_accept_invalid_certs: false,
        }
    }

    /// Create an insecure configuration that accepts invalid certificates
    /// ⚠️  WARNING: This should only be used for testing!
    pub fn danger_accept_invalid_certs() -> Self {
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth();

        Self {
            config: Arc::new(config),
            danger_accept_invalid_certs: true,
        }
    }

    /// Get the underlying rustls config
    pub(crate) fn rustls_config(&self) -> Arc<ClientConfig> {
        self.config.clone()
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// A certificate verifier that accepts all certificates (INSECURE!)
#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

/// SMTP transport layer
#[derive(Clone)]
pub struct SmtpTransport {
    host: String,
    port: u16,
    tls_config: Option<TlsConfig>,
    use_tls: bool,
    use_starttls: bool,
}

impl SmtpTransport {
    /// Create a new SMTP transport
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            tls_config: None,
            use_tls: port == 465,
            use_starttls: port == 587,
        }
    }

    /// Enable TLS (for port 465)
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(tls_config);
        self.use_tls = true;
        self
    }

    /// Enable STARTTLS (for port 587)
    pub fn with_starttls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(tls_config);
        self.use_starttls = true;
        self
    }

    /// Get the host
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if using direct TLS
    pub fn is_tls(&self) -> bool {
        self.use_tls
    }

    /// Check if using STARTTLS
    pub fn is_starttls(&self) -> bool {
        self.use_starttls
    }

    /// Get TLS configuration
    pub fn tls_config(&self) -> Option<&TlsConfig> {
        self.tls_config.as_ref()
    }
}

impl Default for SmtpTransport {
    fn default() -> Self {
        Self::new("localhost", 587)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_creation() {
        let transport = SmtpTransport::new("smtp.example.com", 587);
        assert_eq!(transport.host(), "smtp.example.com");
        assert_eq!(transport.port(), 587);
        assert!(transport.is_starttls());
    }

    #[test]
    fn test_transport_tls() {
        let tls_config = TlsConfig::new();
        let transport = SmtpTransport::new("smtp.example.com", 465)
            .with_tls(tls_config);
        assert!(transport.is_tls());
    }
}
