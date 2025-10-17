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
    /// No authentication
    None,
}

impl AuthMechanism {
    /// Get the SASL mechanism name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Login => "LOGIN",
            Self::None => "",
        }
    }

    /// Encode credentials for the authentication mechanism
    pub fn encode(&self, creds: &Credentials) -> Vec<String> {
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
            }
        }
        
        mechanisms
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
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
    fn test_parse_auth_mechanisms() {
        let response = "250-AUTH PLAIN LOGIN\r\n250 HELP";
        let mechanisms = AuthMechanism::from_ehlo_response(response);
        assert!(mechanisms.contains(&AuthMechanism::Plain));
        assert!(mechanisms.contains(&AuthMechanism::Login));
    }
}
