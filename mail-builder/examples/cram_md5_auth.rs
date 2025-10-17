use mail_core::{Message, Address};
use mail_smtp::{SmtpTransport, SmtpClient, Credentials, AuthMechanism, TlsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CRAM-MD5 Authentication Example");
    println!("================================");

    // Create email message
    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Test email with CRAM-MD5 authentication")
        .body("text/plain", "This email was sent using CRAM-MD5 authentication.");

    // Create transport with STARTTLS
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    // Create credentials
    let credentials = Credentials::new("your_username", "your_password");

    // Create client
    let client = SmtpClient::new(transport);
    
    println!("Connecting to SMTP server...");
    
    // You can specify the authentication mechanism explicitly
    match client.send_with_auth(&message, &credentials, AuthMechanism::CramMd5).await {
        Ok(_) => {
            println!("✅ Email sent successfully using CRAM-MD5 authentication!");
        }
        Err(e) => {
            println!("❌ Failed to send email: {}", e);
            
            // Fallback to other auth methods if CRAM-MD5 is not supported
            println!("Trying with LOGIN authentication as fallback...");
            match client.send_with_auth(&message, &credentials, AuthMechanism::Login).await {
                Ok(_) => println!("✅ Email sent successfully using LOGIN authentication!"),
                Err(e) => println!("❌ Failed with LOGIN: {}", e),
            }
        }
    }

    // Demonstrate using client with pre-configured credentials
    println!("\nDemonstrating pre-configured credentials:");
    let transport2 = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());
    let client_with_creds = SmtpClient::new(transport2).credentials(credentials.clone());
    match client_with_creds.send(&message).await {
        Ok(_) => println!("✅ Email sent with pre-configured credentials (auto-auth with CRAM-MD5 preference)!"),
        Err(e) => println!("❌ Pre-configured auth failed: {}", e),
    }

    Ok(())
}