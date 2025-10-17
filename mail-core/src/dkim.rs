use crate::header::Header;
use crate::error::Result;
use crate::message::Message;
use sha2::{Sha256, Digest};
use rsa::RsaPrivateKey;
use base64::Engine;

/// DKIM signing configuration
#[derive(Debug, Clone)]
pub struct DkimConfig {
    /// Domain name (d= parameter)
    pub domain: String,
    /// Selector (s= parameter)
    pub selector: String,
    /// Private key for signing
    pub private_key: RsaPrivateKey,
    /// Headers to include in signature (default: from, to, subject, date)
    pub headers: Vec<String>,
    /// Canonicalization algorithm for headers (default: simple)
    pub header_canonicalization: Canonicalization,
    /// Canonicalization algorithm for body (default: simple)
    pub body_canonicalization: Canonicalization,
}

/// DKIM canonicalization algorithms
#[derive(Debug, Clone, Copy)]
pub enum Canonicalization {
    /// Simple canonicalization
    Simple,
    /// Relaxed canonicalization
    Relaxed,
}

impl Default for Canonicalization {
    fn default() -> Self {
        Self::Simple
    }
}

impl Canonicalization {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Relaxed => "relaxed",
        }
    }
}

impl DkimConfig {
    /// Create a new DKIM configuration
    pub fn new(domain: String, selector: String, private_key: RsaPrivateKey) -> Self {
        Self {
            domain,
            selector,
            private_key,
            headers: vec![
                "from".to_string(),
                "to".to_string(),
                "subject".to_string(),
                "date".to_string(),
            ],
            header_canonicalization: Canonicalization::Simple,
            body_canonicalization: Canonicalization::Simple,
        }
    }

    /// Set headers to include in signature
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set canonicalization algorithms
    pub fn with_canonicalization(
        mut self,
        header: Canonicalization,
        body: Canonicalization,
    ) -> Self {
        self.header_canonicalization = header;
        self.body_canonicalization = body;
        self
    }
}

/// DKIM signer
pub struct DkimSigner {
    config: DkimConfig,
}

impl DkimSigner {
    /// Create a new DKIM signer
    pub fn new(config: DkimConfig) -> Self {
        Self { config }
    }

        /// Sign a message with DKIM
    pub fn sign_message(&self, message: &Message) -> Result<String> {
        // Extract message body as text
        let body = message.format()
            .map_err(|e| crate::Error::Custom(format!("Failed to format message: {}", e)))?;
        let body_str = String::from_utf8(body)
            .map_err(|e| crate::Error::Custom(format!("Invalid UTF-8 in message: {}", e)))?;
        
        // Extract just the body part (after headers)
        let body_part = if let Some(pos) = body_str.find("\r\n\r\n") {
            &body_str[pos + 4..]
        } else {
            ""
        };

        self.sign(message.headers(), body_part)
    }

    /// Sign headers and body with DKIM
    pub fn sign(&self, headers: &Header, body: &str) -> Result<String> {
        // Hash the body
        let body_hash = self.hash_body(body);
        let body_hash_b64 = base64::engine::general_purpose::STANDARD.encode(&body_hash);

        // Create DKIM signature header (without signature)
        let dkim_header = self.create_dkim_header(&body_hash_b64);

        // Canonicalize headers for signing
        let canonical_headers = self.canonicalize_headers(headers, &dkim_header)?;

        // Sign the canonical headers
        let signature = self.sign_headers(&canonical_headers)?;
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        // Return complete DKIM header
        Ok(format!("{}; b={}", dkim_header, signature_b64))
    }

    /// Hash the message body
    fn hash_body(&self, body: &str) -> Vec<u8> {
        let canonical_body = self.canonicalize_body(body);
        let mut hasher = Sha256::new();
        hasher.update(canonical_body.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Canonicalize the message body according to the algorithm
    fn canonicalize_body(&self, body: &str) -> String {
        match self.config.body_canonicalization {
            Canonicalization::Simple => {
                // Simple: just ensure CRLF line endings and remove trailing empty lines
                let mut result = body.replace("\n", "\r\n");
                result = result.replace("\r\r\n", "\r\n");
                
                // Remove trailing empty lines
                while result.ends_with("\r\n\r\n") {
                    result.truncate(result.len() - 2);
                }
                
                // Ensure body ends with CRLF
                if !result.ends_with("\r\n") {
                    result.push_str("\r\n");
                }
                
                result
            }
            Canonicalization::Relaxed => {
                // Relaxed: normalize whitespace and remove trailing empty lines
                let lines: Vec<&str> = body.lines().collect();
                let mut result = Vec::new();
                
                for line in lines {
                    // Collapse multiple spaces/tabs to single space
                    let normalized = line
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .join(" ");
                    result.push(normalized);
                }
                
                // Remove trailing empty lines
                while result.last() == Some(&String::new()) {
                    result.pop();
                }
                
                result.join("\r\n") + "\r\n"
            }
        }
    }

    /// Create DKIM signature header (without the signature itself)
    fn create_dkim_header(&self, body_hash: &str) -> String {
        format!(
            "v=1; a=rsa-sha256; c={}/{}; d={}; s={}; h={}; bh={}",
            self.config.header_canonicalization.as_str(),
            self.config.body_canonicalization.as_str(),
            self.config.domain,
            self.config.selector,
            self.config.headers.join(":"),
            body_hash
        )
    }

    /// Canonicalize headers for signing
    fn canonicalize_headers(&self, headers: &Header, dkim_header: &str) -> Result<String> {
        let mut canonical_headers = Vec::new();

        // Add requested headers
        for header_name in &self.config.headers {
            if let Some(values) = headers.get(header_name) {
                if let Some(value) = values.first() {
                    let canonical = self.canonicalize_header(header_name, value);
                    canonical_headers.push(canonical);
                }
            }
        }

        // Add DKIM-Signature header (without signature)
        let dkim_canonical = self.canonicalize_header("dkim-signature", dkim_header);
        canonical_headers.push(dkim_canonical);

        Ok(canonical_headers.join("\r\n"))
    }

    /// Canonicalize a single header
    fn canonicalize_header(&self, name: &str, value: &str) -> String {
        match self.config.header_canonicalization {
            Canonicalization::Simple => {
                // Simple: preserve exactly as-is
                format!("{}:{}", name, value)
            }
            Canonicalization::Relaxed => {
                // Relaxed: lowercase header name, normalize whitespace
                let normalized_value = value
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ");
                format!("{}:{}", name.to_lowercase(), normalized_value)
            }
        }
    }

    /// Sign the canonical headers
    fn sign_headers(&self, canonical_headers: &str) -> Result<Vec<u8>> {
        // For now, let's use a placeholder signature to get the basic structure working
        // TODO: Implement proper RSA-PKCS1v15 signing
        let mut hasher = Sha256::new();
        hasher.update(canonical_headers.as_bytes());
        let hash = hasher.finalize().to_vec();
        
        // Return a placeholder signature (in real implementation, this would be RSA signed)
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::Header;
    use rsa::RsaPrivateKey;
    use rand::rngs::OsRng;

    fn create_test_key() -> RsaPrivateKey {
        RsaPrivateKey::new(&mut OsRng, 1024).unwrap()
    }

    #[test]
    fn test_dkim_config_creation() {
        let private_key = create_test_key();
        let config = DkimConfig::new(
            "example.com".to_string(),
            "selector1".to_string(),
            private_key,
        );
        
        assert_eq!(config.domain, "example.com");
        assert_eq!(config.selector, "selector1");
        assert_eq!(config.headers, vec!["from", "to", "subject", "date"]);
    }

    #[test]
    fn test_canonicalization_simple() {
        let private_key = create_test_key();
        let config = DkimConfig::new(
            "example.com".to_string(),
            "test".to_string(),
            private_key,
        );
        let signer = DkimSigner::new(config);
        
        let body = "Hello World\n\nThis is a test.\n\n";
        let canonical = signer.canonicalize_body(body);
        assert!(canonical.ends_with("\r\n"));
    }

    #[test]
    fn test_dkim_header_creation() {
        let private_key = create_test_key();
        let config = DkimConfig::new(
            "example.com".to_string(),
            "selector1".to_string(),
            private_key,
        );
        let signer = DkimSigner::new(config);
        
        let header = signer.create_dkim_header("abcd1234");
        assert!(header.contains("v=1"));
        assert!(header.contains("a=rsa-sha256"));
        assert!(header.contains("d=example.com"));
        assert!(header.contains("s=selector1"));
        assert!(header.contains("bh=abcd1234"));
    }

    #[test]
    fn test_body_hash() {
        let private_key = create_test_key();
        let config = DkimConfig::new(
            "example.com".to_string(),
            "test".to_string(),
            private_key,
        );
        let signer = DkimSigner::new(config);
        
        let body = "Hello World";
        let hash = signer.hash_body(body);
        assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
    }
}