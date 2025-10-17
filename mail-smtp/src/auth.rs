/// SMTP authentication credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Supported authentication mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMechanism {
    /// PLAIN authentication (RFC 4616)
    Plain,
    /// LOGIN authentication
    Login,
    /// CRAM-MD5 authentication (RFC 2195)
    CramMd5,
    /// No authentication
    None,
}

impl AuthMechanism {
    /// Get the SASL mechanism name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Login => "LOGIN",
            Self::CramMd5 => "CRAM-MD5",
            Self::None => "",
        }
    }

    /// Encode credentials for the authentication mechanism
    /// For CRAM-MD5, challenge parameter should contain the server challenge
    pub fn encode(&self, creds: &Credentials) -> Vec<String> {
        self.encode_with_challenge(creds, None)
    }

    /// Encode credentials with optional challenge (for CRAM-MD5)
    pub fn encode_with_challenge(&self, creds: &Credentials, challenge: Option<&str>) -> Vec<String> {
        match self {
            Self::Plain => {
                // PLAIN: \0username\0password
                let mut buf = Vec::new();
                buf.push(0);
                buf.extend_from_slice(creds.username.as_bytes());
                buf.push(0);
                buf.extend_from_slice(creds.password.as_bytes());
                vec![base64_encode(&buf)]
            }
            Self::Login => {
                // LOGIN: username and password separately
                vec![
                    base64_encode(creds.username.as_bytes()),
                    base64_encode(creds.password.as_bytes()),
                ]
            }
            Self::CramMd5 => {
                // CRAM-MD5: username + space + HMAC-MD5(password, challenge)
                if let Some(challenge_str) = challenge {
                    let challenge_bytes = base64_decode(challenge_str).unwrap_or_default();
                    let digest = hmac_md5(creds.password.as_bytes(), &challenge_bytes);
                    let response = format!("{} {}", creds.username, hex_encode(&digest));
                    vec![base64_encode(response.as_bytes())]
                } else {
                    vec![] // CRAM-MD5 requires a challenge
                }
            }
            Self::None => vec![],
        }
    }

    /// Parse supported mechanisms from EHLO response
    pub fn from_ehlo_response(response: &str) -> Vec<Self> {
        let mut mechanisms = Vec::new();
        
        for line in response.lines() {
            if line.starts_with("250-AUTH ") || line.starts_with("250 AUTH ") {
                let auth_line = &line[9..];
                if auth_line.contains("PLAIN") {
                    mechanisms.push(Self::Plain);
                }
                if auth_line.contains("LOGIN") {
                    mechanisms.push(Self::Login);
                }
                if auth_line.contains("CRAM-MD5") {
                    mechanisms.push(Self::CramMd5);
                }
            }
        }
        
        mechanisms
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(data)
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hmac_md5(key: &[u8], data: &[u8]) -> [u8; 16] {
    // Prepare key
    let key = if key.len() > 64 {
        let hash = md5::compute(key);
        let mut result = [0u8; 64];
        result[..16].copy_from_slice(&hash[..]);
        result
    } else {
        let mut result = [0u8; 64];
        result[..key.len()].copy_from_slice(key);
        result
    };
    
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    
    // XOR key with ipad and opad
    for i in 0..64 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }
    
    // Inner hash: MD5(key XOR ipad || data)
    let mut inner_data = Vec::new();
    inner_data.extend_from_slice(&ipad);
    inner_data.extend_from_slice(data);
    let inner_hash = md5::compute(&inner_data);
    
    // Outer hash: MD5(key XOR opad || inner_hash)
    let mut outer_data = Vec::new();
    outer_data.extend_from_slice(&opad);
    outer_data.extend_from_slice(&inner_hash[..]);
    let outer_hash = md5::compute(&outer_data);
    
    outer_hash.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials() {
        let creds = Credentials::new("user", "pass");
        assert_eq!(creds.username, "user");
        assert_eq!(creds.password, "pass");
    }

    #[test]
    fn test_plain_auth() {
        let creds = Credentials::new("user", "pass");
        let encoded = AuthMechanism::Plain.encode(&creds);
        assert_eq!(encoded.len(), 1);
        // \0user\0pass encoded in base64
        assert_eq!(encoded[0], "AHVzZXIAcGFzcw==");
    }

    #[test]
    fn test_login_auth() {
        let creds = Credentials::new("user", "pass");
        let encoded = AuthMechanism::Login.encode(&creds);
        assert_eq!(encoded.len(), 2);
        assert_eq!(encoded[0], "dXNlcg=="); // "user"
        assert_eq!(encoded[1], "cGFzcw=="); // "pass"
    }

    #[test]
    fn test_cram_md5_auth() {
        let creds = Credentials::new("user", "password");
        let challenge = "PDE4OTYuNjk3MTcwNzUyQHBvc3RvZmZpY2UucmVzdG9uLm1jaS5uZXQ+";
        let encoded = AuthMechanism::CramMd5.encode_with_challenge(&creds, Some(challenge));
        assert_eq!(encoded.len(), 1);
        // Should contain username and HMAC-MD5 digest
        let decoded = base64_decode(&encoded[0]).unwrap();
        let response = String::from_utf8(decoded).unwrap();
        assert!(response.starts_with("user "));
        assert!(response.len() > "user ".len());
    }

    #[test]
    fn test_cram_md5_without_challenge() {
        let creds = Credentials::new("user", "password");
        let encoded = AuthMechanism::CramMd5.encode_with_challenge(&creds, None);
        assert_eq!(encoded.len(), 0); // Should return empty without challenge
    }

    #[test]
    fn test_parse_auth_mechanisms_with_cram_md5() {
        let response = "250-AUTH PLAIN LOGIN CRAM-MD5\r\n250 HELP";
        let mechanisms = AuthMechanism::from_ehlo_response(response);
        assert!(mechanisms.contains(&AuthMechanism::Plain));
        assert!(mechanisms.contains(&AuthMechanism::Login));
        assert!(mechanisms.contains(&AuthMechanism::CramMd5));
    }
}
