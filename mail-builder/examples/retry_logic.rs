use mail_smtp::{
    SmtpTransport, TlsConfig, Credentials, SmtpClient,
    RetryableSmtpClient, RetryConfig, send_with_retry,
};
use mail_core::{Message, Address};
use std::time::Duration;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Retry Logic Example");
    
    // Create a custom retry configuration
    let retry_config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        retry_on_connection_error: true,
        retry_on_auth_error: false, // Don't retry auth errors
        retry_on_server_error: true, // Retry server errors (5xx)
    };
    
    println!("📋 Retry Configuration:");
    println!("   Max attempts: {}", retry_config.max_attempts);
    println!("   Initial delay: {:?}", retry_config.initial_delay);
    println!("   Max delay: {:?}", retry_config.max_delay);
    println!("   Backoff multiplier: {}", retry_config.backoff_multiplier);
    println!("   Retry on connection error: {}", retry_config.retry_on_connection_error);
    println!("   Retry on auth error: {}", retry_config.retry_on_auth_error);
    println!("   Retry on server error: {}", retry_config.retry_on_server_error);
    
    // Create the transport (using a test SMTP server)
    let transport = SmtpTransport::new("localhost", 587)
        .with_starttls(TlsConfig::danger_accept_invalid_certs());
    
    // Create a regular SMTP client
    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("test@example.com", "password"));
    
    // Wrap it with retry functionality
    let retryable_client = RetryableSmtpClient::with_config(client, retry_config.clone());
    
    // Create a test email message
    let message = Message::new()
        .from(Address::new("sender@example.com"))
        .to(vec![Address::new("recipient@example.com")])
        .subject("Retry Test Email")
        .body("text/plain", "This email tests the retry functionality.");
    
    println!("\n📧 Testing Retry with Client Wrapper");
    let start_time = Instant::now();
    
    // This will fail since we're using localhost without a real SMTP server
    // but it demonstrates the retry behavior
    match retryable_client.send(&message).await {
        Ok(_) => println!("✅ Email sent successfully with retries"),
        Err(e) => println!("❌ Email failed after retries: {}", e),
    }
    
    let elapsed = start_time.elapsed();
    println!("⏱️  Total time with retries: {:?}", elapsed);
    
    // Demonstrate the generic retry function
    println!("\n🔄 Testing Generic Retry Function");
    
    let mut attempt_count = 0;
    let start_time = Instant::now();
    
    let result = send_with_retry(&retry_config, || {
        attempt_count += 1;
        println!("📤 Attempt {} via generic retry function", attempt_count);
        
        async move {
            // Simulate an operation that might fail
            if attempt_count < 3 {
                Err(mail_smtp::Error::Connection("Simulated connection error".to_string()))
            } else {
                Ok(())
            }
        }
    }).await;
    
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(_) => println!("✅ Operation succeeded after {} attempts", attempt_count),
        Err(e) => println!("❌ Operation failed after {} attempts: {}", attempt_count, e),
    }
    
    println!("⏱️  Total time: {:?}", elapsed);
    
    // Demonstrate delay calculation
    println!("\n⏳ Delay Calculation Examples:");
    for attempt in 0..5 {
        let delay = retry_config.calculate_delay(attempt);
        println!("   Attempt {}: delay = {:?}", attempt + 1, delay);
    }
    
    // Show what errors would be retried
    println!("\n🔍 Error Retry Policy:");
    let test_errors = vec![
        mail_smtp::Error::Connection("Connection refused".to_string()),
        mail_smtp::Error::Authentication("Invalid credentials".to_string()),
        mail_smtp::Error::Command("500 Internal server error".to_string()),
        mail_smtp::Error::Command("421 Service not available".to_string()),
        mail_smtp::Error::Timeout,
        mail_smtp::Error::Tls("TLS handshake failed".to_string()),
        mail_smtp::Error::Custom("Custom error".to_string()),
    ];
    
    for error in test_errors {
        let will_retry = retry_config.should_retry(&error);
        println!("   {} -> Retry: {}", error, will_retry);
    }
    
    println!("\n🎯 Benefits of Retry Logic:");
    println!("   ✅ Handles transient network issues automatically");
    println!("   ✅ Exponential backoff prevents server overload");
    println!("   ✅ Configurable retry policies for different error types");
    println!("   ✅ Graceful handling of temporary server unavailability");
    println!("   ✅ Improves reliability without manual intervention");
    
    Ok(())
}