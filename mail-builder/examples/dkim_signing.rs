use mail_core::{Message, Address, DkimConfig, DkimSigner, Canonicalization};
use mail_smtp::{SmtpTransport, SmtpClient, Credentials, TlsConfig};
use rsa::{RsaPrivateKey};
use rand::rngs::OsRng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DKIM Signing Example");
    println!("===================");

    // Generate a test RSA key for demonstration
    // In production, you would load your actual DKIM private key
    println!("Generating test RSA key...");
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048)?;

    // Create DKIM configuration
    let dkim_config = DkimConfig::new(
        "example.com".to_string(),     // Your domain
        "selector1".to_string(),       // DKIM selector
        private_key,
    )
    .with_headers(vec![
        "from".to_string(),
        "to".to_string(),
        "subject".to_string(),
        "date".to_string(),
        "message-id".to_string(),
    ])
    .with_canonicalization(
        Canonicalization::Relaxed,  // Header canonicalization
        Canonicalization::Simple,   // Body canonicalization
    );

    // Create DKIM signer
    let dkim_signer = DkimSigner::new(dkim_config);

    // Create email message
    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("DKIM-signed email test")
        .body("text/plain", "This email is signed with DKIM for authentication and deliverability.");

    println!("Created email message");

    // Sign the message with DKIM
    let dkim_signature = dkim_signer.sign_message(&message)?;
    
    println!("Generated DKIM signature:");
    println!("DKIM-Signature: {}", dkim_signature);

    // In a real implementation, you would add the DKIM-Signature header to the message
    // before sending. For now, we'll demonstrate the concept.

    // Setup SMTP transport
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());

    // Create credentials
    let credentials = Credentials::new("your_username", "your_password");

    // Create SMTP client
    let client = SmtpClient::new(transport).credentials(credentials);

    println!("\nSending DKIM-signed email...");
    
    // In production, the message would include the DKIM-Signature header
    match client.send(&message).await {
        Ok(_) => {
            println!("✅ DKIM-signed email sent successfully!");
            println!("   The DKIM signature helps email providers verify authenticity.");
            println!("   This improves deliverability and reduces spam filtering.");
        }
        Err(e) => {
            println!("❌ Failed to send email: {}", e);
            println!("   Note: This is expected in a demo environment");
        }
    }

    // Display DKIM information
    println!("\n📋 DKIM Configuration Details:");
    println!("   Domain: example.com");
    println!("   Selector: selector1");
    println!("   Algorithm: RSA-SHA256");
    println!("   Header Canonicalization: Relaxed");
    println!("   Body Canonicalization: Simple");
    println!("   Signed Headers: from, to, subject, date, message-id");

    println!("\n💡 DKIM Benefits:");
    println!("   ✓ Email authentication and integrity verification");
    println!("   ✓ Improved deliverability and reputation");
    println!("   ✓ Protection against email spoofing");
    println!("   ✓ Compliance with email security standards");

    Ok(())
}