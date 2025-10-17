use mail_smtp::{SmtpPool, PoolConfig, SmtpTransport, TlsConfig, Credentials};
use mail_core::{Message, Address};
use std::time::Duration;
use std::sync::Arc;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Connection Pool Example");
    
    // Create a custom pool configuration
    let pool_config = PoolConfig {
        max_connections: 5,
        min_connections: 2,
        max_idle_time: Duration::from_secs(300), // 5 minutes
        connection_timeout: Duration::from_secs(30),
        acquire_timeout: Duration::from_secs(10),
    };
    
    // Create the transport (using a test SMTP server or localhost)
    let transport = SmtpTransport::new("localhost", 587)
        .with_starttls(TlsConfig::danger_accept_invalid_certs());
    
    // Create the connection pool
    let pool = Arc::new(SmtpPool::with_config(transport, pool_config)
        .credentials(Credentials::new("test@example.com", "password")));
    
    println!("📊 Initializing pool with minimum connections...");
    // Initialize the pool (this would fail with real SMTP in this example)
    // pool.initialize().await?;
    
    // Create multiple email messages
    let messages = vec![
        Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient1@example.com")])
            .subject("Bulk Email 1")
            .body("text/plain", "This is the first bulk email."),
        
        Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient2@example.com")])
            .subject("Bulk Email 2")
            .body("text/plain", "This is the second bulk email."),
        
        Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient3@example.com")])
            .subject("Bulk Email 3")
            .body("text/plain", "This is the third bulk email."),
    ];
    
    println!("📧 Sending {} emails using connection pool...", messages.len());
    let start_time = Instant::now();
    
    // Send emails concurrently using the pool
    let mut tasks = Vec::new();
    for (i, message) in messages.into_iter().enumerate() {
        let pool_clone = Arc::clone(&pool);
        let task = tokio::spawn(async move {
            println!("📤 Sending email {} via pool", i + 1);
            
            // This would fail in this example since we're using localhost without a real SMTP server
            // In a real scenario, this would work with proper SMTP credentials
            match pool_clone.send(&message).await {
                Ok(_) => println!("✅ Email {} sent successfully", i + 1),
                Err(e) => println!("❌ Email {} failed: {}", i + 1, e),
            }
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
    
    let elapsed = start_time.elapsed();
    println!("⏱️  Total time: {:?}", elapsed);
    
    // Get pool statistics
    let stats = pool.stats().await;
    println!("📊 Pool Statistics:");
    println!("   {}", stats);
    
    // Close the pool
    pool.close().await;
    println!("🔌 Pool closed successfully");
    
    // Demonstrate the difference between pool and individual clients
    println!("\n🔄 Comparison with individual connections:");
    println!("With connection pool:");
    println!("   ✅ Reuses connections for better performance");
    println!("   ✅ Limits concurrent connections to prevent server overload");
    println!("   ✅ Automatic connection cleanup when idle");
    println!("   ✅ Connection sharing across multiple send operations");
    
    println!("\nWithout connection pool:");
    println!("   ❌ Creates new connection for each email");
    println!("   ❌ Higher latency due to connection establishment");
    println!("   ❌ More resource usage on both client and server");
    println!("   ❌ Potential server connection limits issues");
    
    Ok(())
}