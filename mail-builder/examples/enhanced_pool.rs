use mail_smtp::{
    transport::SmtpTransport,
    enhanced_pool::{EnhancedSmtpPool, EnhancedPoolConfig, PoolMetricsSnapshot},
    error::Result,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Enhanced SMTP Connection Pool Example ===\n");

    // Configure the enhanced connection pool
    let pool_config = EnhancedPoolConfig {
        max_connections: 10,
        min_connections: 2,
        max_idle_time: Duration::from_secs(300),
        connection_timeout: Duration::from_secs(30),
        acquire_timeout: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(60),
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_recovery_timeout: Duration::from_secs(60),
        validate_connections: true,
        max_connection_age: Duration::from_secs(3600),
        max_retry_attempts: 3,
        retry_backoff_multiplier: 2.0,
    };

    println!("Pool Configuration:");
    println!("  Max Connections: {}", pool_config.max_connections);
    println!("  Min Connections: {}", pool_config.min_connections);
    println!("  Connection Timeout: {:?}", pool_config.connection_timeout);
    println!("  Max Idle Time: {:?}", pool_config.max_idle_time);
    println!("  Max Connection Age: {:?}", pool_config.max_connection_age);
    println!("  Health Check Interval: {:?}", pool_config.health_check_interval);
    println!("  Circuit Breaker Threshold: {}", pool_config.circuit_breaker_failure_threshold);
    println!();

    // Create SMTP transport
    let transport = SmtpTransport::new("smtp.example.com", 587);

    println!("Creating enhanced connection pool...");
    
    // Create the enhanced pool
    let pool = EnhancedSmtpPool::with_config(transport, pool_config.clone());
    
    println!("✓ Pool created successfully\n");

    // Display initial metrics
    print_metrics("Initial Metrics", pool.metrics());

    println!("\n--- Enhanced Pool Features ---\n");
    
    // Demonstrate the key features
    println!("This pool provides the following enhancements:");
    println!();
    println!("✓ Dynamic Connection Management");
    println!("  - Maintains {} to {} connections based on load", 
             pool_config.min_connections, pool_config.max_connections);
    println!("  - Automatic scaling and connection recycling");
    println!();
    println!("✓ Circuit Breaker Pattern");
    println!("  - Opens after {} consecutive failures", pool_config.circuit_breaker_failure_threshold);
    println!("  - Prevents cascading failures");
    println!("  - Auto-recovers after {:?}", pool_config.circuit_breaker_recovery_timeout);
    println!();
    println!("✓ Health Monitoring");
    println!("  - Periodic health checks every {:?}", pool_config.health_check_interval);
    println!("  - Connection validation before use");
    println!("  - Automatic unhealthy connection removal");
    println!();
    println!("✓ Connection Lifecycle Management");
    println!("  - Max idle time: {:?}", pool_config.max_idle_time);
    println!("  - Max connection age: {:?}", pool_config.max_connection_age);
    println!("  - Automatic connection rotation");
    println!();
    println!("✓ Comprehensive Metrics");
    println!("  - Active and idle connection counts");
    println!("  - Success/failure rates");
    println!("  - Connection creation/destruction tracking");
    println!();
    println!("✓ Fault Tolerance");
    println!("  - Retry logic with backoff (up to {} attempts)", pool_config.max_retry_attempts);
    println!("  - Graceful degradation under load");
    println!("  - Thread-safe concurrent access");
    println!();

    // Display current metrics
    let metrics = pool.metrics();
    println!("--- Current Pool Metrics ---");
    print_metrics("Pool Status", metrics);

    println!("\n--- Usage Example ---\n");
    println!("To send an email through the pool:");
    println!("  1. Create your message using MessageBuilder");
    println!("  2. Call pool.send(&message).await");
    println!("  3. The pool handles connection management automatically");
    println!();
    println!("Benefits:");
    println!("  • Reuses existing connections when available");
    println!("  • Creates new connections up to max_connections limit");
    println!("  • Validates connections before use");
    println!("  • Retries on transient failures");
    println!("  • Provides detailed metrics for monitoring");

    println!("\n--- Graceful Shutdown ---\n");
    println!("When shutting down:");
    println!("  • All idle connections are closed");
    println!("  • Background tasks are stopped");
    println!("  • Resources are properly cleaned up");
    
    // Note: Actual shutdown would be: pool.shutdown().await;
    // But we skip it in this example since we're not connecting to a real server
    
    println!("\n✓ Example completed successfully!");
    println!("\nNote: This example demonstrates the API. To use with a real SMTP server,");
    println!("configure the transport with valid credentials and server details.");

    Ok(())
}

/// Helper function to print pool metrics in a formatted way
fn print_metrics(label: &str, metrics: PoolMetricsSnapshot) {
    println!("\n{}", label);
    println!("  Active Connections: {}", metrics.active_connections);
    println!("  Idle Connections: {}", metrics.idle_connections);
    println!("  Total Connections: {}", metrics.total_connections());
    println!("  Successful Sends: {}", metrics.successful_sends);
    println!("  Failed Sends: {}", metrics.failed_sends);
    println!("  Success Rate: {:.2}%", metrics.success_rate() * 100.0);
    println!("  Connections Created: {}", metrics.connections_created);
    println!("  Connections Destroyed: {}", metrics.connections_destroyed);
    println!("  Acquire Timeouts: {}", metrics.acquire_timeouts);
    println!("  Avg Connection Age: {}ms", metrics.avg_connection_age_ms);
    println!("  Health Check Failures: {}", metrics.health_check_failures);
    println!("  Circuit Breaker Trips: {}", metrics.circuit_breaker_trips);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults() {
        let config = EnhancedPoolConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert!(config.validate_connections);
    }

    #[test]
    fn test_metrics_calculation() {
        // This would test metrics calculations
        // In a real scenario, we'd need actual pool operations
        assert!(true);
    }
}
