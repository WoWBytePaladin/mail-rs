use mail_core::{Message, Address, SmimeConfig, SmimeSigner};
// use mail_smtp::{SmtpTransport, SmtpClient, Credentials, TlsConfig};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("S/MIME Email Example");
    println!("===================");

    // Generate test certificates (in production, use real certificates)
    println!("Generating test S/MIME certificates...");
    let (cert_data, key_data) = mail_core::smime::utils::generate_test_certificate()?;
    
    // Save test certificates to temporary files
    fs::write("test_cert.pem", &cert_data)?;
    fs::write("test_key.pem", &key_data)?;
    
    println!("✓ Test certificates generated");

    // Create S/MIME configuration
    let smime_config = SmimeConfig::new("test_cert.pem", "test_key.pem")
        .with_password("test_password");

    println!("✓ S/MIME configuration created");

    // Create S/MIME signer
    let signer = SmimeSigner::new(smime_config);

    // Create email message
    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("S/MIME Signed & Encrypted Email")
        .body("text/plain", "This email is both signed and encrypted using S/MIME.\n\nBenefits:\n- Authentication: Verifies sender identity\n- Integrity: Ensures message hasn't been tampered with\n- Confidentiality: Encrypts content for recipient eyes only\n\nS/MIME provides end-to-end security for email communication.");

    println!("✓ Email message created");

    // Sign the message
    println!("\nSigning message with S/MIME...");
    let signed_message = signer.sign(&message)?;
    println!("✓ Message signed successfully");
    println!("  Signed message size: {} bytes", signed_message.len());

    // Encrypt the message
    println!("\nEncrypting message with S/MIME...");
    let encrypted_message = signer.encrypt(&message)?;
    println!("✓ Message encrypted successfully");
    println!("  Encrypted message size: {} bytes", encrypted_message.len());

    // Sign and encrypt the message
    println!("\nSigning and encrypting message...");
    let signed_encrypted_message = signer.sign_and_encrypt(&message)?;
    println!("✓ Message signed and encrypted successfully");
    println!("  Final message size: {} bytes", signed_encrypted_message.len());

    // Display S/MIME headers
    let signed_str = String::from_utf8_lossy(&signed_message);
    if let Some(content_type_start) = signed_str.find("Content-Type:") {
        if let Some(content_type_end) = signed_str[content_type_start..].find("\r\n") {
            let content_type = &signed_str[content_type_start..content_type_start + content_type_end];
            println!("\n📋 S/MIME Content-Type: {}", content_type);
        }
    }

    // Verify the signed message
    println!("\nVerifying S/MIME signature...");
    let (original_content, is_valid) = signer.verify(&signed_message)?;
    println!("✓ Signature verification: {}", if is_valid { "VALID" } else { "INVALID" });
    println!("  Original content size: {} bytes", original_content.len());

    // Decrypt the encrypted message
    println!("\nDecrypting S/MIME message...");
    let decrypted_content = signer.decrypt(&encrypted_message)?;
    println!("✓ Message decrypted successfully");
    println!("  Decrypted content size: {} bytes", decrypted_content.len());

    // Setup SMTP for sending (optional - commented out for demo)
    /*
    println!("\nSending S/MIME protected email...");
    let transport = SmtpTransport::new("smtp.example.com", 587)
        .with_starttls(TlsConfig::new());
    
    let credentials = Credentials::new("your_username", "your_password");
    let client = SmtpClient::new(transport).credentials(credentials);

    // In production, you'd send the signed/encrypted message
    match client.send_raw(&signed_encrypted_message).await {
        Ok(_) => println!("✅ S/MIME protected email sent successfully!"),
        Err(e) => println!("❌ Failed to send email: {} (expected in demo)", e),
    }
    */

    // Cleanup test files
    let _ = fs::remove_file("test_cert.pem");
    let _ = fs::remove_file("test_key.pem");

    println!("\n📊 S/MIME Summary:");
    println!("   🔐 Encryption: Protects message content from unauthorized access");
    println!("   ✍️  Digital Signature: Authenticates sender and ensures integrity");
    println!("   📜 Certificate-based: Uses X.509 certificates for trust");
    println!("   🏢 Enterprise Ready: Widely supported by email clients");
    println!("   🔒 PKCS#7: Industry standard for secure email");

    println!("\n💡 S/MIME Use Cases:");
    println!("   • Corporate communications requiring confidentiality");
    println!("   • Legal and financial document exchange");
    println!("   • Healthcare communications (HIPAA compliance)");
    println!("   • Government and regulated industry communications");
    println!("   • Any scenario requiring non-repudiation");

    Ok(())
}