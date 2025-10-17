use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use mail_core::Message;
use crate::{
    client::SmtpClient,
    pool::SmtpPool,
    error::{Error, Result},
};

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of emails per time window
    pub max_emails: usize,
    /// Duration of the time window
    pub time_window: Duration,
    /// Whether to use sliding window (true) or fixed window (false)
    pub sliding_window: bool,
    /// Maximum time to wait for rate limit to clear
    pub max_wait_time: Option<Duration>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_emails: 100,
            time_window: Duration::from_secs(60), // 100 emails per minute
            sliding_window: true,
            max_wait_time: Some(Duration::from_secs(30)),
        }
    }
}

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct TokenBucketRateLimiter {
    capacity: usize,
    tokens: Arc<Mutex<f64>>,
    refill_rate: f64, // tokens per second
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucketRateLimiter {
    /// Create a new token bucket rate limiter
    pub fn new(max_emails: usize, time_window: Duration) -> Self {
        let refill_rate = max_emails as f64 / time_window.as_secs_f64();
        
        Self {
            capacity: max_emails,
            tokens: Arc::new(Mutex::new(max_emails as f64)),
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Try to acquire a token (returns true if successful)
    pub async fn acquire(&self) -> bool {
        self.refill_tokens().await;
        
        let mut tokens = self.tokens.lock().await;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Wait until a token is available
    pub async fn acquire_wait(&self, max_wait: Option<Duration>) -> Result<()> {
        let start = Instant::now();
        
        loop {
            if self.acquire().await {
                return Ok(());
            }
            
            if let Some(max_wait) = max_wait {
                if start.elapsed() > max_wait {
                    return Err(Error::Custom("Rate limit wait timeout".to_string()));
                }
            }
            
            // Wait a short time before trying again
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.lock().await;
        let time_passed = now.duration_since(*last_refill).as_secs_f64();
        
        if time_passed > 0.0 {
            let mut tokens = self.tokens.lock().await;
            let new_tokens = *tokens + (time_passed * self.refill_rate);
            *tokens = new_tokens.min(self.capacity as f64);
            *last_refill = now;
        }
    }

    /// Get current token count
    pub async fn tokens_available(&self) -> f64 {
        self.refill_tokens().await;
        *self.tokens.lock().await
    }
}

/// Rate limiter using sliding window algorithm
#[derive(Debug)]
pub struct SlidingWindowRateLimiter {
    max_emails: usize,
    time_window: Duration,
    timestamps: Arc<Mutex<VecDeque<Instant>>>,
}

impl SlidingWindowRateLimiter {
    /// Create a new sliding window rate limiter
    pub fn new(max_emails: usize, time_window: Duration) -> Self {
        Self {
            max_emails,
            time_window,
            timestamps: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Try to acquire permission (returns true if successful)
    pub async fn acquire(&self) -> bool {
        let now = Instant::now();
        let mut timestamps = self.timestamps.lock().await;
        
        // Remove old timestamps outside the window
        while let Some(&front) = timestamps.front() {
            if now.duration_since(front) > self.time_window {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we can add a new timestamp
        if timestamps.len() < self.max_emails {
            timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    /// Wait until permission is available
    pub async fn acquire_wait(&self, max_wait: Option<Duration>) -> Result<()> {
        let start = Instant::now();
        
        loop {
            if self.acquire().await {
                return Ok(());
            }
            
            if let Some(max_wait) = max_wait {
                if start.elapsed() > max_wait {
                    return Err(Error::Custom("Rate limit wait timeout".to_string()));
                }
            }
            
            // Calculate when the next slot will be available
            let timestamps = self.timestamps.lock().await;
            if timestamps.front().is_some() {
                let wait_time = self.time_window - start.elapsed().min(self.time_window);
                drop(timestamps);
                sleep(wait_time.min(Duration::from_millis(100))).await;
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    /// Get current usage count
    pub async fn current_usage(&self) -> usize {
        let now = Instant::now();
        let mut timestamps = self.timestamps.lock().await;
        
        // Remove old timestamps
        while let Some(&front) = timestamps.front() {
            if now.duration_since(front) > self.time_window {
                timestamps.pop_front();
            } else {
                break;
            }
        }
        
        timestamps.len()
    }
}

/// Rate-limited SMTP client
pub struct RateLimitedSmtpClient {
    client: SmtpClient,
    rate_limiter: Box<dyn RateLimiter + Send + Sync>,
    config: RateLimitConfig,
}

/// Trait for rate limiting implementations
#[async_trait::async_trait]
pub trait RateLimiter {
    async fn acquire(&self) -> bool;
    async fn acquire_wait(&self, max_wait: Option<Duration>) -> Result<()>;
}

#[async_trait::async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn acquire(&self) -> bool {
        self.acquire().await
    }

    async fn acquire_wait(&self, max_wait: Option<Duration>) -> Result<()> {
        self.acquire_wait(max_wait).await
    }
}

#[async_trait::async_trait]
impl RateLimiter for SlidingWindowRateLimiter {
    async fn acquire(&self) -> bool {
        self.acquire().await
    }

    async fn acquire_wait(&self, max_wait: Option<Duration>) -> Result<()> {
        self.acquire_wait(max_wait).await
    }
}

impl RateLimitedSmtpClient {
    /// Create a new rate-limited SMTP client with default configuration
    pub fn new(client: SmtpClient) -> Self {
        let config = RateLimitConfig::default();
        Self::with_config(client, config)
    }

    /// Create a new rate-limited SMTP client with custom configuration
    pub fn with_config(client: SmtpClient, config: RateLimitConfig) -> Self {
        let rate_limiter: Box<dyn RateLimiter + Send + Sync> = if config.sliding_window {
            Box::new(SlidingWindowRateLimiter::new(config.max_emails, config.time_window))
        } else {
            Box::new(TokenBucketRateLimiter::new(config.max_emails, config.time_window))
        };

        Self {
            client,
            rate_limiter,
            config,
        }
    }

    /// Send an email with rate limiting
    pub async fn send(&self, message: &Message) -> Result<()> {
        // Wait for rate limit clearance
        self.rate_limiter.acquire_wait(self.config.max_wait_time).await?;
        
        // Send the email
        self.client.send(message).await
    }

    /// Try to send an email without waiting (returns error if rate limited)
    pub async fn try_send(&self, message: &Message) -> Result<()> {
        if !self.rate_limiter.acquire().await {
            return Err(Error::Custom("Rate limit exceeded".to_string()));
        }
        
        self.client.send(message).await
    }
}

/// Rate-limited SMTP pool
pub struct RateLimitedSmtpPool {
    pool: SmtpPool,
    rate_limiter: Box<dyn RateLimiter + Send + Sync>,
    config: RateLimitConfig,
}

impl RateLimitedSmtpPool {
    /// Create a new rate-limited SMTP pool with default configuration
    pub fn new(pool: SmtpPool) -> Self {
        let config = RateLimitConfig::default();
        Self::with_config(pool, config)
    }

    /// Create a new rate-limited SMTP pool with custom configuration
    pub fn with_config(pool: SmtpPool, config: RateLimitConfig) -> Self {
        let rate_limiter: Box<dyn RateLimiter + Send + Sync> = if config.sliding_window {
            Box::new(SlidingWindowRateLimiter::new(config.max_emails, config.time_window))
        } else {
            Box::new(TokenBucketRateLimiter::new(config.max_emails, config.time_window))
        };

        Self {
            pool,
            rate_limiter,
            config,
        }
    }

    /// Send an email with rate limiting
    pub async fn send(&self, message: &Message) -> Result<()> {
        // Wait for rate limit clearance
        self.rate_limiter.acquire_wait(self.config.max_wait_time).await?;
        
        // Send the email
        self.pool.send(message).await
    }

    /// Try to send an email without waiting (returns error if rate limited)
    pub async fn try_send(&self, message: &Message) -> Result<()> {
        if !self.rate_limiter.acquire().await {
            return Err(Error::Custom("Rate limit exceeded".to_string()));
        }
        
        self.pool.send(message).await
    }

    /// Get pool statistics
    pub async fn stats(&self) -> crate::pool::PoolStats {
        self.pool.stats().await
    }

    /// Initialize the pool
    pub async fn initialize(&self) -> Result<()> {
        self.pool.initialize().await
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await
    }
}

/// Statistics for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub max_emails: usize,
    pub time_window: Duration,
    pub current_usage: usize,
    pub available_slots: usize,
}

impl std::fmt::Display for RateLimitStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate Limit: {}/{} emails in {:?} window, {} slots available",
            self.current_usage,
            self.max_emails,
            self.time_window,
            self.available_slots
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_token_bucket_rate_limiter() {
        let limiter = TokenBucketRateLimiter::new(2, Duration::from_secs(1));
        
        // Should be able to acquire 2 tokens immediately
        assert!(limiter.acquire().await);
        assert!(limiter.acquire().await);
        
        // Third acquisition should fail
        assert!(!limiter.acquire().await);
        
        // Wait a bit and try again (should have some tokens refilled)
        sleep(Duration::from_millis(600)).await;
        assert!(limiter.acquire().await);
    }

    #[tokio::test]
    async fn test_sliding_window_rate_limiter() {
        let limiter = SlidingWindowRateLimiter::new(2, Duration::from_millis(100));
        
        // Should be able to acquire 2 permits immediately
        assert!(limiter.acquire().await);
        assert!(limiter.acquire().await);
        
        // Third acquisition should fail
        assert!(!limiter.acquire().await);
        
        // Wait for window to slide
        sleep(Duration::from_millis(110)).await;
        
        // Should be able to acquire again
        assert!(limiter.acquire().await);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_emails, 100);
        assert_eq!(config.time_window, Duration::from_secs(60));
        assert!(config.sliding_window);
    }
}