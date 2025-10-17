use crate::oauth2::OAuth2Token;

/// SMTP authentication credentials
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Username and password authentication
    Basic { username: String, password: String },
    /// OAuth2 token authentication
    OAuth2(OAuth2Token),
}

impl Credentials {
    /// Create new basic credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create OAuth2 credentials
    pub fn oauth2(token: OAuth2Token) -> Self {
        Self::OAuth2(token)
    }

    /// Get username (for basic auth only)
    pub fn username(&self) -> Option<&str> {
        match self {
            Self::Basic { username, .. } => Some(username),
            Self::OAuth2(_) => None,
        }
    }

    /// Get password (for basic auth only) 
    pub fn password(&self) -> Option<&str> {
        match self {
            Self::Basic { password, .. } => Some(password),
            Self::OAuth2(_) => None,
        }
    }

    /// Get OAuth2 token
    pub fn oauth2_token(&self) -> Option<&OAuth2Token> {
        match self {
            Self::OAuth2(token) => Some(token),
            Self::Basic { .. } => None,
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
    /// OAuth2 XOAUTH2 mechanism
    XOAuth2,
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
            Self::XOAuth2 => "XOAUTH2",
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
                match creds {
                    Credentials::Basic { username, password } => {
                        let mut buf = Vec::new();
                        buf.push(0);
                        buf.extend_from_slice(username.as_bytes());
                        buf.push(0);
                        buf.extend_from_slice(password.as_bytes());
                        vec![base64_encode(&buf)]
                    }
                    Credentials::OAuth2(_) => vec![], // OAuth2 doesn't use PLAIN
                }
            }
            Self::Login => {
                // LOGIN: username and password separately
                match creds {
                    Credentials::Basic { username, password } => {
                        vec![
                            base64_encode(username.as_bytes()),
                            base64_encode(password.as_bytes()),
                        ]
                    }
                    Credentials::OAuth2(_) => vec![], // OAuth2 doesn't use LOGIN
                }
            }
            Self::CramMd5 => {
                // CRAM-MD5: username + space + HMAC-MD5(password, challenge)
                match creds {
                    Credentials::Basic { username, password } => {
                        if let Some(challenge_str) = challenge {
                            let challenge_bytes = base64_decode(challenge_str).unwrap_or_default();
                            let digest = hmac_md5(password.as_bytes(), &challenge_bytes);
                            let response = format!("{} {}", username, hex_encode(&digest));
                            vec![base64_encode(response.as_bytes())]
                        } else {
                            vec![] // CRAM-MD5 requires a challenge
                        }
                    }
                    Credentials::OAuth2(_) => vec![], // OAuth2 doesn't use CRAM-MD5
                }
            }
            Self::XOAuth2 => {
                // XOAUTH2: user=username^Aauth=Bearer token^A^A
                match creds {
                    Credentials::OAuth2(token) => {
                        if let Some(username) = creds.username() {
                            let auth_string = format!("user={}\x01auth={}\x01\x01", username, token.authorization_header());
                            vec![base64_encode(auth_string.as_bytes())]
                        } else {
                            // Extract username from token if available, or use email from scope
                            let auth_string = format!("user=user\x01auth={}\x01\x01", token.authorization_header());
                            vec![base64_encode(auth_string.as_bytes())]
                        }
                    }
                    Credentials::Basic { .. } => vec![], // Basic auth doesn't use XOAUTH2
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
                if auth_line.contains("XOAUTH2") {
                    mechanisms.push(Self::XOAuth2);
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
        assert_eq!(creds.username(), Some("user"));
        assert_eq!(creds.password(), Some("pass"));
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
    fn test_oauth2_credentials() {
        let token = OAuth2Token::new("access123", "Bearer", Some(3600), None);
        let creds = Credentials::oauth2(token);
        
        assert!(creds.oauth2_token().is_some());
        assert!(creds.username().is_none());
        assert!(creds.password().is_none());
    }

    #[test]
    fn test_xoauth2_auth() {
        let token = OAuth2Token::new("access123", "Bearer", Some(3600), None);
        let creds = Credentials::oauth2(token);
        let encoded = AuthMechanism::XOAuth2.encode(&creds);
        
        assert_eq!(encoded.len(), 1);
        // Should contain the OAuth2 token in XOAUTH2 format
        assert!(encoded[0].len() > 0);
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
