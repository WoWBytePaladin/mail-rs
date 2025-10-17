//! Bulk email sending with connection reuse

use mail_builder::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // List of recipients
    let recipients = vec![
        ("alice@example.com", "Alice"),
        ("bob@example.com", "Bob"),
        ("charlie@example.com", "Charlie"),
    ];

    // Configure SMTP
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"))
        .timeout(Duration::from_secs(60));

    // Send personalized emails
    for (email, name) in recipients {
        let message = Message::new()
            .from(Address::new("newsletter@example.com"))
            .to(vec![Address::with_name(email, name)])
            .subject(format!("Hello, {}!", name))
            .body("text/plain", format!("Dear {},\n\nThank you for subscribing!", name));

        match client.send(&message).await {
            Ok(_) => println!("✓ Sent to {}", email),
            Err(e) => eprintln!("Failed to send to {}: {}", email, e),
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\nBulk send completed!");
    Ok(())
}
