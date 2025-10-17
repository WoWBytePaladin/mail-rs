//! Simple email sending example

use mail_builder::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a message
    let message = Message::new()
        .from(Address::with_name("sender@example.com", "Sender Name"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Hello from mail-rs!")
        .body("text/plain", "This is a test email sent using mail-rs.");

    // Configure SMTP transport
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    // Create SMTP client with credentials
    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"));

    // Send the email
    client.send(&message).await?;

    println!("Email sent successfully!");
    Ok(())
}
