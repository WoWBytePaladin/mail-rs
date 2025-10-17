use mail_core::{Message, Address};
use mail_smtp::{
    SmtpTransport, SmtpClient, Credentials, TlsConfig,
    OAuth2Config, OAuth2Client, TokenManager
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OAuth2 SMTP Authentication Example");
    println!("==================================");

    // Step 1: Configure OAuth2 for Gmail
    let oauth2_config = OAuth2Config::gmail(
        "your-client-id.apps.googleusercontent.com",
        "your-client-secret"
    );

    println!("✓ OAuth2 configuration created for Gmail");

    // Step 2: Get authorization URL for user consent
    let oauth2_client = OAuth2Client::new(oauth2_config.clone());
    let auth_url = oauth2_client.get_authorization_url(Some("random-state-123"));
    
    println!("\n📋 OAuth2 Authorization Process:");
    println!("1. Visit this URL to authorize the application:");
    println!("   {}", auth_url);
    println!("2. After authorization, you'll receive an authorization code");
    println!("3. Exchange the code for access token using the OAuth2 client");

    // Simulate OAuth2 flow with placeholder tokens
    // In a real application, you would:
    // 1. Open the auth_url in a browser
    // 2. User grants consent
    // 3. Capture the authorization code from redirect
    // 4. Exchange code for access token

    println!("\n🔄 OAuth2 Token Exchange (simulated):");
    
    // For demonstration, create a placeholder token
    // In real usage: let token = oauth2_client.exchange_code(&auth_code).await?;
    let token = mail_smtp::OAuth2Token::new(
        "ya29.example_access_token_here",
        "Bearer",
        Some(3600), // expires in 1 hour
        Some("1//example_refresh_token_here".to_string())
    );
    
    println!("✓ Access token obtained (simulated)");
    println!("  Token type: {}", token.token_type);
    println!("  Expires in: {} seconds", token.expires_in.unwrap_or(0));
    println!("  Has refresh token: {}", token.refresh_token.is_some());

    // Step 3: Create token manager for automatic refresh
    let mut token_manager = TokenManager::new(oauth2_config.clone());
    token_manager.set_token(token.clone());

    println!("✓ Token manager configured with automatic refresh");

    // Step 4: Create OAuth2 credentials
    let oauth2_credentials = Credentials::oauth2(token);

    // Step 5: Setup SMTP with OAuth2
    let transport = SmtpTransport::new("smtp.gmail.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(oauth2_credentials);

    println!("✓ SMTP client configured with OAuth2 authentication");

    // Step 6: Create and send email message
    let message = Message::new()
        .from(Address::new("your-email@gmail.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("OAuth2 Authenticated Email")
        .body("text/html", r#"
            <html>
                <body>
                    <h2>🔐 OAuth2 Authenticated Email</h2>
                    <p>This email was sent using OAuth2 authentication with Gmail SMTP.</p>
                    
                    <h3>Benefits of OAuth2 Authentication:</h3>
                    <ul>
                        <li>✓ No need to store passwords</li>
                        <li>✓ Scoped access permissions</li>
                        <li>✓ Token-based security</li>
                        <li>✓ Automatic token refresh</li>
                        <li>✓ Revocable access</li>
                    </ul>
                    
                    <h3>Supported Providers:</h3>
                    <ul>
                        <li>📧 Gmail / Google Workspace</li>
                        <li>📧 Microsoft Outlook / Office 365</li>
                        <li>📧 Custom OAuth2 providers</li>
                    </ul>
                    
                    <p><em>Sent with mail-rs OAuth2 support</em></p>
                </body>
            </html>
        "#);

    println!("\n📧 Sending OAuth2 authenticated email...");

    // Attempt to send (will fail in demo due to invalid credentials)
    match client.send(&message).await {
        Ok(_) => {
            println!("✅ Email sent successfully with OAuth2 authentication!");
        }
        Err(e) => {
            println!("❌ Failed to send email: {}", e);
            println!("   Note: This is expected in a demo environment with placeholder tokens");
        }
    }

    // Step 7: Demonstrate token refresh
    println!("\n🔄 Token Refresh Demonstration:");
    
    match token_manager.get_valid_token().await {
        Ok(valid_token) => {
            println!("✓ Valid token available");
            println!("  Authorization header: {}", valid_token.authorization_header());
        }
        Err(e) => {
            println!("❌ Token refresh needed: {}", e);
            println!("   In production, implement proper token refresh logic");
        }
    }

    // Step 8: Configuration examples for other providers
    println!("\n📋 OAuth2 Configuration Examples:");
    
    // Microsoft Outlook/Office 365
    let outlook_config = OAuth2Config::outlook(
        "your-outlook-client-id",
        "your-outlook-client-secret"
    );
    println!("✓ Outlook/Office 365 config: {}", outlook_config.auth_url);

    // Custom provider
    let custom_config = OAuth2Config::custom(
        "client-id",
        "client-secret", 
        "https://provider.com/oauth2/authorize",
        "https://provider.com/oauth2/token"
    ).with_scopes(vec!["email.send".to_string()]);
    println!("✓ Custom provider config: {}", custom_config.auth_url);

    println!("\n💡 OAuth2 Implementation Guide:");
    println!("1. Register your application with the email provider");
    println!("2. Configure OAuth2 client credentials"); 
    println!("3. Implement authorization code flow");
    println!("4. Exchange authorization code for access token");
    println!("5. Use TokenManager for automatic token refresh");
    println!("6. Create OAuth2 credentials for SMTP client");
    println!("7. Send emails with secure, token-based authentication");

    println!("\n🔒 Security Benefits:");
    println!("• No password storage required");
    println!("• Granular permission scopes");
    println!("• Token expiration and refresh");
    println!("• Audit trail of application access");
    println!("• User can revoke access anytime");

    Ok(())
}