//! Embedded images in HTML email

use mail_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTML email with embedded image
    let html_body = r#"
        <html>
        <body>
            <h1>Welcome!</h1>
            <p>Here is our logo:</p>
            <img src="cid:logo.png" alt="Company Logo" />
            <p>Best regards,<br/>The Team</p>
        </body>
        </html>
    "#;

    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Welcome Email with Logo")
        .body("text/html", html_body)
        .embed("logo.png", b"PNG image data...".to_vec());

    // Configure SMTP
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"));

    client.send(&message).await?;

    println!("Email with embedded image sent!");
    Ok(())
}
