use crate::{Error, Result};
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};

/// OAuth2 configuration for SMTP authentication
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// Client ID from the OAuth2 provider
    pub client_id: String,
    /// Client secret from the OAuth2 provider  
    pub client_secret: String,
    /// Authorization server URL
    pub auth_url: String,
    /// Token endpoint URL
    pub token_url: String,
    /// Redirect URI for authorization code flow
    pub redirect_uri: String,
    /// OAuth2 scopes required for email access
    pub scopes: Vec<String>,
}

impl OAuth2Config {
    /// Create OAuth2 config for Gmail/Google Workspace
    pub fn gmail(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "urn:ietf:wg:oauth:2.0:oob".to_string(),
            scopes: vec!["https://mail.google.com/".to_string()],
        }
    }

    /// Create OAuth2 config for Microsoft Outlook/Office 365
    pub fn outlook(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["https://outlook.office.com/SMTP.Send".to_string()],
        }
    }

    /// Create custom OAuth2 configuration
    pub fn custom(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: auth_url.into(),
            token_url: token_url.into(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: Vec::new(),
        }
    }

    /// Add OAuth2 scopes
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Set redirect URI
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = redirect_uri.into();
        self
    }

    /// Generate authorization URL for user consent
    pub fn get_authorization_url(&self, state: Option<&str>) -> String {
        let scope_string = self.scopes.join(" ");
        let mut params = vec![
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("scope", &scope_string),
        ];

        if let Some(state_value) = state {
            params.push(("state", state_value));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.auth_url, query_string)
    }
}

/// OAuth2 token response from authorization server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    /// Access token for API requests
    pub access_token: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Token expiration time in seconds
    pub expires_in: Option<u64>,
    /// Refresh token for getting new access tokens
    pub refresh_token: Option<String>,
    /// OAuth2 scopes granted
    pub scope: Option<String>,
    /// When the token was issued (for expiration calculation)
    #[serde(skip, default = "SystemTime::now")]
    pub issued_at: SystemTime,
}

impl OAuth2Token {
    /// Create a new OAuth2 token
    pub fn new(
        access_token: impl Into<String>,
        token_type: impl Into<String>,
        expires_in: Option<u64>,
        refresh_token: Option<String>,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            token_type: token_type.into(),
            expires_in,
            refresh_token,
            scope: None,
            issued_at: SystemTime::now(),
        }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_in) = self.expires_in {
            let expiry_time = self.issued_at + Duration::from_secs(expires_in);
            SystemTime::now() > expiry_time
        } else {
            false
        }
    }

    /// Check if token expires within the given duration
    pub fn expires_within(&self, duration: Duration) -> bool {
        if let Some(expires_in) = self.expires_in {
            let expiry_time = self.issued_at + Duration::from_secs(expires_in);
            let check_time = SystemTime::now() + duration;
            check_time >= expiry_time
        } else {
            false
        }
    }

    /// Get the authorization header value
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// OAuth2 client for managing authentication flow
#[derive(Debug)]
pub struct OAuth2Client {
    config: OAuth2Config,
    http_client: reqwest::Client,
}

impl OAuth2Client {
    /// Create a new OAuth2 client
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<OAuth2Token> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Failed to exchange code: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Custom(format!("Token exchange failed: {}", error_text)));
        }

        let mut token: OAuth2Token = response.json().await
            .map_err(|e| Error::Custom(format!("Failed to parse token response: {}", e)))?;
        
        token.issued_at = SystemTime::now();
        Ok(token)
    }

    /// Refresh an access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuth2Token> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self.http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Failed to refresh token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Custom(format!("Token refresh failed: {}", error_text)));
        }

        let mut token: OAuth2Token = response.json().await
            .map_err(|e| Error::Custom(format!("Failed to parse refresh response: {}", e)))?;
        
        token.issued_at = SystemTime::now();
        Ok(token)
    }

    /// Get authorization URL for user consent
    pub fn get_authorization_url(&self, state: Option<&str>) -> String {
        self.config.get_authorization_url(state)
    }
}

/// OAuth2 token manager for automatic refresh
#[derive(Debug)]
pub struct TokenManager {
    client: OAuth2Client,
    current_token: Option<OAuth2Token>,
    refresh_threshold: Duration,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            client: OAuth2Client::new(config),
            current_token: None,
            refresh_threshold: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Set the refresh threshold (how early to refresh before expiry)
    pub fn with_refresh_threshold(mut self, threshold: Duration) -> Self {
        self.refresh_threshold = threshold;
        self
    }

    /// Set the current token
    pub fn set_token(&mut self, token: OAuth2Token) {
        self.current_token = Some(token);
    }

    /// Get a valid access token, refreshing if necessary
    pub async fn get_valid_token(&mut self) -> Result<&OAuth2Token> {
        // Check if current token is still valid
        let needs_refresh = if let Some(ref token) = self.current_token {
            token.expires_within(self.refresh_threshold)
        } else {
            return Err(Error::Custom("No token available".to_string()));
        };

        if needs_refresh {
            // Token needs refresh, try to refresh it
            if let Some(ref token) = self.current_token.clone() {
                if let Some(ref refresh_token) = token.refresh_token {
                    match self.client.refresh_token(refresh_token).await {
                        Ok(new_token) => {
                            self.current_token = Some(new_token);
                        }
                        Err(e) => {
                            return Err(Error::Custom(format!("Failed to refresh token: {}", e)));
                        }
                    }
                } else {
                    return Err(Error::Custom("No refresh token available".to_string()));
                }
            }
        }

        // Return the valid token
        self.current_token.as_ref()
            .ok_or_else(|| Error::Custom("No valid token available".to_string()))
    }

    /// Get the OAuth2 client for initial authorization
    pub fn client(&self) -> &OAuth2Client {
        &self.client
    }

    /// Check if we have a valid token
    pub fn has_valid_token(&self) -> bool {
        if let Some(ref token) = self.current_token {
            !token.expires_within(self.refresh_threshold)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_config_gmail() {
        let config = OAuth2Config::gmail("client123", "secret456");
        
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret456");
        assert!(config.auth_url.contains("google.com"));
        assert!(config.token_url.contains("googleapis.com"));
        assert_eq!(config.scopes, vec!["https://mail.google.com/"]);
    }

    #[test]
    fn test_oauth2_config_outlook() {
        let config = OAuth2Config::outlook("client123", "secret456");
        
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret456");
        assert!(config.auth_url.contains("microsoftonline.com"));
        assert!(config.token_url.contains("microsoftonline.com"));
        assert_eq!(config.scopes, vec!["https://outlook.office.com/SMTP.Send"]);
    }

    #[test]
    fn test_oauth2_token_expiry() {
        let token = OAuth2Token::new("access123", "Bearer", Some(3600), None);
        
        assert!(!token.is_expired());
        assert!(!token.expires_within(Duration::from_secs(100)));
        assert!(token.expires_within(Duration::from_secs(7200))); // 2 hours
    }

    #[test]
    fn test_authorization_header() {
        let token = OAuth2Token::new("access123", "Bearer", Some(3600), None);
        assert_eq!(token.authorization_header(), "Bearer access123");
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuth2Config::gmail("client123", "secret456");
        let url = config.get_authorization_url(Some("state123"));
        
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope="));
    }

    #[test]
    fn test_token_manager_creation() {
        let config = OAuth2Config::gmail("client123", "secret456");
        let manager = TokenManager::new(config);
        
        assert!(!manager.has_valid_token());
    }

    #[test]
    fn test_token_manager_with_token() {
        let config = OAuth2Config::gmail("client123", "secret456");
        let mut manager = TokenManager::new(config);
        
        let token = OAuth2Token::new("access123", "Bearer", Some(3600), None);
        manager.set_token(token);
        
        assert!(manager.has_valid_token());
    }
}