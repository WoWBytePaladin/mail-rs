//! Using TLS (port 465) instead of STARTTLS

use mail_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Email via TLS")
        .body("text/plain", "This email was sent using direct TLS connection.");

    // Use port 465 for direct TLS
    let transport = SmtpTransport::new("smtp.example.com", 465)
        .with_tls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"));

    client.send(&message).await?;

    println!("Email sent via TLS!");
    Ok(())
}
