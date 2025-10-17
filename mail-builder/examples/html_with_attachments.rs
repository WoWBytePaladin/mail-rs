//! Email with HTML and attachments

use mail_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an HTML email with plain text alternative
    let message = Message::new()
        .from(Address::with_name("sender@example.com", "Company Newsletter"))
        .to(vec![
            Address::with_name("user1@example.com", "User One"),
            Address::with_name("user2@example.com", "User Two"),
        ])
        .cc(vec![Address::new("cc@example.com")])
        .subject("Monthly Newsletter - October 2025")
        .body("text/plain", "Hello,\n\nThis is our monthly newsletter.")
        .alternative("text/html", "<html><body><h1>Hello!</h1><p>This is our <b>monthly newsletter</b>.</p></body></html>")
        .attach("report.pdf", b"PDF content here".to_vec())
        .attach("data.csv", b"Name,Email\nJohn,john@example.com".to_vec());

    // Configure SMTP
    let transport = SmtpTransport::new("smtp.gmail.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("your-email@gmail.com", "your-app-password"));

    // Send
    client.send(&message).await?;

    println!("Newsletter sent successfully!");
    Ok(())
}
