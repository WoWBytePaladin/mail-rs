use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use mail_core::Message;
use crate::{
    client::{SmtpClient, SmtpConnection},
    transport::SmtpTransport,
    auth::Credentials,
    error::{Error, Result},
};

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum idle time before closing a connection
    pub max_idle_time: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Maximum time to wait for a connection from the pool
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            max_idle_time: Duration::from_secs(300), // 5 minutes
            connection_timeout: Duration::from_secs(30),
            acquire_timeout: Duration::from_secs(10),
        }
    }
}

/// A pooled SMTP connection wrapper
#[derive(Debug)]
struct PooledConnection {
    connection: SmtpConnection,
    created_at: Instant,
    last_used: Instant,
}

impl PooledConnection {
    fn new(connection: SmtpConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
        }
    }

    fn is_idle_expired(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }

    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }
}

/// SMTP connection pool for reusing connections
pub struct SmtpPool {
    config: PoolConfig,
    transport: SmtpTransport,
    credentials: Option<Credentials>,
    pool: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    active_connections: Arc<Mutex<usize>>,
}

impl SmtpPool {
    /// Create a new SMTP connection pool
    pub fn new(transport: SmtpTransport) -> Self {
        let config = PoolConfig::default();
        Self::with_config(transport, config)
    }

    /// Create a new SMTP connection pool with custom configuration
    pub fn with_config(transport: SmtpTransport, config: PoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            transport,
            credentials: None,
            pool: Arc::new(Mutex::new(VecDeque::new())),
            active_connections: Arc::new(Mutex::new(0)),
            config,
        }
    }

    /// Set authentication credentials for all connections
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Send an email using a pooled connection
    pub async fn send(&self, message: &Message) -> Result<()> {
        let mut connection = self.acquire_connection().await?;
        
        // Get sender and recipients
        let from_email = message.from_address()
            .ok_or_else(|| Error::Custom("Missing From header".to_string()))?;
        
        let recipients = message.recipients();
        if recipients.is_empty() {
            return Err(Error::Custom("No recipients specified".to_string()));
        }

        // Send email using the connection
        let result = self.send_with_connection(&mut connection, message, &from_email, &recipients).await;
        
        // Return connection to pool
        self.release_connection(connection).await;
        
        result
    }

    /// Acquire a connection from the pool or create a new one
    async fn acquire_connection(&self) -> Result<SmtpConnection> {
        // Wait for semaphore permit
        let _permit = timeout(self.config.acquire_timeout, self.semaphore.acquire())
            .await
            .map_err(|_| Error::Custom("Timeout waiting for connection".to_string()))?
            .map_err(|_| Error::Custom("Semaphore closed".to_string()))?;

        // Try to get a connection from the pool
        if let Some(mut pooled_conn) = self.pop_available_connection().await {
            pooled_conn.update_last_used();
            return Ok(pooled_conn.connection);
        }

        // Create a new connection
        self.create_new_connection().await
    }

    /// Pop an available connection from the pool
    async fn pop_available_connection(&self) -> Option<PooledConnection> {
        let mut pool = self.pool.lock().await;
        
        // Remove expired connections
        while let Some(conn) = pool.front() {
            if conn.is_idle_expired(self.config.max_idle_time) {
                pool.pop_front();
                let mut active = self.active_connections.lock().await;
                *active -= 1;
            } else {
                break;
            }
        }

        pool.pop_front()
    }

    /// Create a new SMTP connection
    async fn create_new_connection(&self) -> Result<SmtpConnection> {
        let client = SmtpClient::new(self.transport.clone())
            .timeout(self.config.connection_timeout);

        let client = if let Some(ref creds) = self.credentials {
            client.credentials(creds.clone())
        } else {
            client
        };

        let connection = client.connect().await?;
        
        let mut active = self.active_connections.lock().await;
        *active += 1;

        Ok(connection)
    }

    /// Release a connection back to the pool
    async fn release_connection(&self, connection: SmtpConnection) {
        let mut pool = self.pool.lock().await;
        
        // Only keep the connection if we haven't exceeded min_connections
        if pool.len() < self.config.min_connections {
            pool.push_back(PooledConnection::new(connection));
        } else {
            // Close the connection
            let mut active = self.active_connections.lock().await;
            *active -= 1;
        }
    }

    /// Send email using an existing connection
    async fn send_with_connection(
        &self,
        connection: &mut SmtpConnection,
        message: &Message,
        from_email: &str,
        recipients: &[String],
    ) -> Result<()> {
        // Send MAIL FROM
        connection.mail_from(from_email).await?;

        // Send RCPT TO for each recipient
        for recipient in recipients {
            connection.rcpt_to(recipient).await?;
        }

        // Send DATA command
        connection.data().await?;

        // Send message content
        let message_data = message.format().map_err(|e| Error::Custom(e.to_string()))?;
        connection.write_data(&message_data).await?;

        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().await;
        let active = self.active_connections.lock().await;
        
        PoolStats {
            active_connections: *active,
            idle_connections: pool.len(),
            total_connections: *active + pool.len(),
            max_connections: self.config.max_connections,
        }
    }

    /// Initialize the pool with minimum connections
    pub async fn initialize(&self) -> Result<()> {
        for _ in 0..self.config.min_connections {
            let connection = self.create_new_connection().await?;
            self.release_connection(connection).await;
        }
        Ok(())
    }

    /// Close all connections in the pool
    pub async fn close(&self) {
        let mut pool = self.pool.lock().await;
        pool.clear();
        
        let mut active = self.active_connections.lock().await;
        *active = 0;
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_connections: usize,
    pub max_connections: usize,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool Stats: {}/{} active, {} idle, {} total",
            self.active_connections,
            self.max_connections,
            self.idle_connections,
            self.total_connections
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::SmtpTransport;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_pool_creation() {
        let transport = SmtpTransport::new("smtp.test.com", 587);
        let _pool = SmtpPool::new(transport);
    }

    #[test]
    fn test_pooled_connection_idle_logic() {
        use std::time::{Duration, Instant};
        
        // Test the idle time calculation logic
        let now = Instant::now();
        let past = now - Duration::from_secs(2);
        
        assert!(past.elapsed() > Duration::from_secs(1));
        assert!(now.elapsed() < Duration::from_secs(1));
    }
}