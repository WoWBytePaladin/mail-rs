use mail_smtp::{
    SmtpTransport, TlsConfig, Credentials, SmtpClient,
    RateLimitedSmtpClient, RateLimitConfig,
    TokenBucketRateLimiter, SlidingWindowRateLimiter,
};
use mail_core::{Message, Address};
use std::time::Duration;
use tokio::time::{sleep, Instant};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚦 Rate Limiting Example");
    
    // Demonstrate different rate limiting algorithms
    println!("\n📊 Token Bucket Rate Limiter Test");
    test_token_bucket().await;
    
    println!("\n📊 Sliding Window Rate Limiter Test");
    test_sliding_window().await;
    
    // Create a rate-limited SMTP client
    println!("\n📧 Rate-Limited SMTP Client Test");
    test_rate_limited_client().await;
    
    println!("\n🎯 Rate Limiting Benefits:");
    println!("   ✅ Prevents server overload and blacklisting");
    println!("   ✅ Ensures compliance with provider limits");
    println!("   ✅ Provides smooth, consistent sending rates");
    println!("   ✅ Automatic backpressure for bulk operations");
    println!("   ✅ Configurable algorithms for different use cases");
    
    Ok(())
}

async fn test_token_bucket() {
    println!("🪣 Testing Token Bucket (5 emails per 2 seconds)");
    let limiter = TokenBucketRateLimiter::new(5, Duration::from_secs(2));
    
    let mut successful = 0;
    let mut failed = 0;
    
    // Try to send 10 emails rapidly
    for i in 1..=10 {
        if limiter.acquire().await {
            successful += 1;
            println!("   ✅ Email {} - Token acquired", i);
        } else {
            failed += 1;
            println!("   ❌ Email {} - Rate limited", i);
        }
        
        // Small delay to simulate processing
        sleep(Duration::from_millis(50)).await;
    }
    
    println!("   📈 Results: {} successful, {} rate limited", successful, failed);
    
    // Show token refill
    println!("   💧 Waiting for tokens to refill...");
    sleep(Duration::from_secs(1)).await;
    
    let tokens = limiter.tokens_available().await;
    println!("   🪙 Tokens available after 1s: {:.2}", tokens);
}

async fn test_sliding_window() {
    println!("🪟 Testing Sliding Window (3 emails per 1 second)");
    let limiter = SlidingWindowRateLimiter::new(3, Duration::from_secs(1));
    
    let mut successful = 0;
    let mut failed = 0;
    
    // Try to send 8 emails rapidly
    for i in 1..=8 {
        if limiter.acquire().await {
            successful += 1;
            println!("   ✅ Email {} - Permission granted", i);
        } else {
            failed += 1;
            println!("   ❌ Email {} - Rate limited", i);
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    println!("   📈 Results: {} successful, {} rate limited", successful, failed);
    
    // Show window sliding
    println!("   ⏳ Waiting for window to slide...");
    sleep(Duration::from_millis(1200)).await;
    
    let usage = limiter.current_usage().await;
    println!("   📊 Current usage after window slide: {}/3", usage);
    
    // Should be able to send more now
    if limiter.acquire().await {
        println!("   ✅ New email after window slide - Success!");
    }
}

async fn test_rate_limited_client() {
    // Create rate limit configuration
    let rate_config = RateLimitConfig {
        max_emails: 3,
        time_window: Duration::from_secs(2),
        sliding_window: true,
        max_wait_time: Some(Duration::from_secs(5)),
    };
    
    println!("⚙️  Rate Limit Config:");
    println!("   Max emails: {} per {:?}", rate_config.max_emails, rate_config.time_window);
    println!("   Algorithm: {}", if rate_config.sliding_window { "Sliding Window" } else { "Token Bucket" });
    println!("   Max wait time: {:?}", rate_config.max_wait_time);
    
    // Create the transport (using a test SMTP server)
    let transport = SmtpTransport::new("localhost", 587)
        .with_starttls(TlsConfig::danger_accept_invalid_certs());
    
    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("test@example.com", "password"));
    
    // Create rate-limited client
    let rate_limited_client = RateLimitedSmtpClient::with_config(client, rate_config);
    
    // Create test messages
    let messages: Vec<Message> = (1..=6)
        .map(|i| {
            Message::new()
                .from(Address::new("sender@example.com"))
                .to(vec![Address::new(&format!("recipient{}@example.com", i))])
                .subject(&format!("Rate Limited Email {}", i))
                .body("text/plain", format!("This is test email number {}", i))
        })
        .collect();
    
    println!("\n📤 Attempting to send {} emails with rate limiting:", messages.len());
    
    let start_time = Instant::now();
    
    for (i, message) in messages.iter().enumerate() {
        let email_start = Instant::now();
        
        // Try to send without waiting first
        match rate_limited_client.try_send(message).await {
            Ok(_) => {
                println!("   ✅ Email {} sent immediately", i + 1);
            }
            Err(_) => {
                // If rate limited, use the waiting version
                println!("   ⏳ Email {} rate limited, waiting...", i + 1);
                
                match rate_limited_client.send(message).await {
                    Ok(_) => {
                        let wait_time = email_start.elapsed();
                        println!("   ✅ Email {} sent after waiting {:?}", i + 1, wait_time);
                    }
                    Err(e) => {
                        println!("   ❌ Email {} failed: {}", i + 1, e);
                    }
                }
            }
        }
    }
    
    let total_time = start_time.elapsed();
    println!("   ⏱️  Total time for all emails: {:?}", total_time);
    
    // Demonstrate burst vs sustained sending
    println!("\n🚀 Comparison: Burst vs Rate-Limited Sending");
    println!("   Without rate limiting:");
    println!("     ❌ Risk of hitting provider limits");
    println!("     ❌ Potential account suspension");
    println!("     ❌ Poor server resource utilization");
    
    println!("   With rate limiting:");
    println!("     ✅ Compliant with provider policies");
    println!("     ✅ Sustainable sending rates");
    println!("     ✅ Better resource management");
    println!("     ✅ Automatic traffic shaping");
}