use std::time::Duration;
use tokio::time::sleep;
use mail_core::Message;
use crate::{
    client::SmtpClient,
    pool::SmtpPool,
    error::{Error, Result},
};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to retry on connection errors
    pub retry_on_connection_error: bool,
    /// Whether to retry on authentication errors
    pub retry_on_auth_error: bool,
    /// Whether to retry on server errors (5xx responses)
    pub retry_on_server_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retry_on_connection_error: true,
            retry_on_auth_error: false, // Don't retry auth errors by default
            retry_on_server_error: true,
        }
    }
}

/// Retry policy for determining if an error should trigger a retry
impl RetryConfig {
    /// Check if an error should trigger a retry
    pub fn should_retry(&self, error: &Error) -> bool {
        match error {
            Error::Connection(_) => self.retry_on_connection_error,
            Error::Authentication(_) => self.retry_on_auth_error,
            Error::Command(msg) => {
                // Try to parse SMTP response code from command error
                if let Some(code_str) = msg.split_whitespace().next() {
                    if let Ok(code) = code_str.parse::<u16>() {
                        self.retry_on_server_error && code >= 500
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Error::Tls(_) => self.retry_on_connection_error,
            Error::Timeout => true, // Always retry timeouts
            Error::InvalidResponse(_) => false,
            Error::Io(_) => self.retry_on_connection_error,
            Error::MailCore(_) => false,
            Error::Custom(_) => false, // Don't retry custom errors by default
        }
    }

    /// Calculate the delay for a given attempt number
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        
        let delay = Duration::from_millis(delay_ms as u64);
        
        // Cap at max_delay
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

/// A wrapper around SmtpClient that provides retry functionality
pub struct RetryableSmtpClient {
    client: SmtpClient,
    retry_config: RetryConfig,
}

impl RetryableSmtpClient {
    /// Create a new retryable SMTP client
    pub fn new(client: SmtpClient) -> Self {
        Self {
            client,
            retry_config: RetryConfig::default(),
        }
    }

    /// Create a new retryable SMTP client with custom retry configuration
    pub fn with_config(client: SmtpClient, retry_config: RetryConfig) -> Self {
        Self {
            client,
            retry_config,
        }
    }

    /// Send an email with retry logic
    pub async fn send(&self, message: &Message) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..self.retry_config.max_attempts {
            match self.client.send(message).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    // Check if we should retry this error
                    if !self.retry_config.should_retry(&error) {
                        return Err(error);
                    }

                    // Don't delay after the last attempt
                    if attempt < self.retry_config.max_attempts - 1 {
                        let delay = self.retry_config.calculate_delay(attempt);
                        log::debug!(
                            "Attempt {} failed with error: {}. Retrying in {:?}",
                            attempt + 1,
                            error,
                            delay
                        );
                        sleep(delay).await;
                    } else {
                        log::debug!(
                            "Attempt {} failed with error: {}. No more retries.",
                            attempt + 1,
                            error
                        );
                    }
                }
            }
        }

        // All retry attempts failed
        Err(last_error.unwrap_or_else(|| Error::Custom("No attempts made".to_string())))
    }
}

/// A wrapper around SmtpPool that provides retry functionality
pub struct RetryableSmtpPool {
    pool: SmtpPool,
    retry_config: RetryConfig,
}

impl RetryableSmtpPool {
    /// Create a new retryable SMTP pool
    pub fn new(pool: SmtpPool) -> Self {
        Self {
            pool,
            retry_config: RetryConfig::default(),
        }
    }

    /// Create a new retryable SMTP pool with custom retry configuration
    pub fn with_config(pool: SmtpPool, retry_config: RetryConfig) -> Self {
        Self {
            pool,
            retry_config,
        }
    }

    /// Send an email with retry logic
    pub async fn send(&self, message: &Message) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..self.retry_config.max_attempts {
            match self.pool.send(message).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    // Check if we should retry this error
                    if !self.retry_config.should_retry(&error) {
                        return Err(error);
                    }

                    // Don't delay after the last attempt
                    if attempt < self.retry_config.max_attempts - 1 {
                        let delay = self.retry_config.calculate_delay(attempt);
                        log::debug!(
                            "Pool attempt {} failed with error: {}. Retrying in {:?}",
                            attempt + 1,
                            error,
                            delay
                        );
                        sleep(delay).await;
                    } else {
                        log::debug!(
                            "Pool attempt {} failed with error: {}. No more retries.",
                            attempt + 1,
                            error
                        );
                    }
                }
            }
        }

        // All retry attempts failed
        Err(last_error.unwrap_or_else(|| Error::Custom("No attempts made".to_string())))
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

/// A future that resolves after attempting to send an email with retries
pub async fn send_with_retry<F, Fut>(
    retry_config: &RetryConfig,
    mut send_fn: F,
) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut last_error = None;

    for attempt in 0..retry_config.max_attempts {
        match send_fn().await {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error.clone());
                
                // Check if we should retry this error
                if !retry_config.should_retry(&error) {
                    return Err(error);
                }

                // Don't delay after the last attempt
                if attempt < retry_config.max_attempts - 1 {
                    let delay = retry_config.calculate_delay(attempt);
                    log::debug!(
                        "Generic retry attempt {} failed with error: {}. Retrying in {:?}",
                        attempt + 1,
                        error,
                        delay
                    );
                    sleep(delay).await;
                }
            }
        }
    }

    // All retry attempts failed
    Err(last_error.unwrap_or_else(|| Error::Custom("No attempts made".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert!(config.retry_on_connection_error);
        assert!(!config.retry_on_auth_error);
    }

    #[test]
    fn test_should_retry() {
        let config = RetryConfig::default();
        
        // Should retry connection errors
        assert!(config.should_retry(&Error::Connection("test".to_string())));
        
        // Should not retry auth errors by default
        assert!(!config.should_retry(&Error::Authentication("test".to_string())));
        
        // Should retry server errors (5xx) - test command parsing
        assert!(config.should_retry(&Error::Command("500 Server error".to_string())));
        assert!(config.should_retry(&Error::Command("502 Bad gateway".to_string())));
        
        // Should not retry client errors (4xx)
        assert!(!config.should_retry(&Error::Command("400 Bad request".to_string())));
        assert!(!config.should_retry(&Error::Command("450 Mailbox busy".to_string())));
        
        // Should retry timeouts
        assert!(config.should_retry(&Error::Timeout));
        
        // Should not retry custom errors
        assert!(!config.should_retry(&Error::Custom("test".to_string())));
        
        // Should not retry invalid responses
        assert!(!config.should_retry(&Error::InvalidResponse("test".to_string())));
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            ..Default::default()
        };
        
        // First retry (attempt 0)
        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        
        // Second retry (attempt 1)
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        
        // Third retry (attempt 2)
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        
        // Test max delay cap
        let config_high = RetryConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(2),
            backoff_multiplier: 10.0,
            ..Default::default()
        };
        
        assert_eq!(config_high.calculate_delay(3), Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_retry_function() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1), // Very short for testing
            ..Default::default()
        };
        
        let mut attempt_count = 0;
        let result = send_with_retry(&config, || {
            attempt_count += 1;
            async move {
                if attempt_count < 3 {
                    Err(Error::Connection("test".to_string()))
                } else {
                    Ok(())
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(attempt_count, 3);
    }

    #[tokio::test]
    async fn test_retry_function_no_retry() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        
        let mut attempt_count = 0;
        let result = send_with_retry(&config, || {
            attempt_count += 1;
            async move {
                Err(Error::Authentication("no retry".to_string()))
            }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(attempt_count, 1); // Should not retry auth errors
    }
}