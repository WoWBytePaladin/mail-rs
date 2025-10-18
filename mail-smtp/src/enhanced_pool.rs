use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore, broadcast};
use tokio::time::{timeout, interval};
use mail_core::Message;
use crate::{
    client::{SmtpClient, SmtpConnection},
    transport::SmtpTransport,
    auth::Credentials,
    error::{Error, Result},
};

/// Enhanced connection pool configuration
#[derive(Debug, Clone)]
pub struct EnhancedPoolConfig {
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
    /// Health check interval
    pub health_check_interval: Duration,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: usize,
    /// Circuit breaker recovery timeout
    pub circuit_breaker_recovery_timeout: Duration,
    /// Enable connection validation
    pub validate_connections: bool,
    /// Maximum connection age before replacement
    pub max_connection_age: Duration,
    /// Connection retry attempts
    pub max_retry_attempts: usize,
    /// Backoff multiplier for retries
    pub retry_backoff_multiplier: f64,
}

impl Default for EnhancedPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            max_idle_time: Duration::from_secs(300), // 5 minutes
            connection_timeout: Duration::from_secs(30),
            acquire_timeout: Duration::from_secs(15),
            health_check_interval: Duration::from_secs(60), // 1 minute
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_timeout: Duration::from_secs(60),
            validate_connections: true,
            max_connection_age: Duration::from_secs(3600), // 1 hour
            max_retry_attempts: 3,
            retry_backoff_multiplier: 2.0,
        }
    }
}

/// Connection health status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Rejecting requests
    HalfOpen,  // Testing recovery
}

/// Enhanced pooled connection with health tracking
#[derive(Debug)]
struct EnhancedPooledConnection {
    connection: SmtpConnection,
    created_at: Instant,
    last_used: Instant,
    last_health_check: Instant,
    health_status: ConnectionHealth,
    failure_count: usize,
    success_count: usize,
    #[allow(dead_code)]
    id: String,
}

impl EnhancedPooledConnection {
    fn new(connection: SmtpConnection) -> Self {
        let now = Instant::now();
        let id = format!("conn-{}", now.elapsed().as_nanos());
        
        Self {
            connection,
            created_at: now,
            last_used: now,
            last_health_check: now,
            health_status: ConnectionHealth::Healthy,
            failure_count: 0,
            success_count: 0,
            id,
        }
    }

    fn is_idle_expired(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }

    fn is_aged(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() > max_age
    }

    fn needs_health_check(&self, check_interval: Duration) -> bool {
        self.last_health_check.elapsed() > check_interval
    }

    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }

    fn update_health_check(&mut self) {
        self.last_health_check = Instant::now();
    }

    fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0; // Reset failure count on success
        self.health_status = ConnectionHealth::Healthy;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= 3 {
            self.health_status = ConnectionHealth::Unhealthy;
        } else if self.failure_count >= 1 {
            self.health_status = ConnectionHealth::Degraded;
        }
    }

    fn is_healthy(&self) -> bool {
        matches!(self.health_status, ConnectionHealth::Healthy)
    }
}

/// Pool performance metrics
#[derive(Debug, Default)]
pub struct PoolMetrics {
    /// Total connections created
    pub connections_created: AtomicU64,
    /// Total connections destroyed
    pub connections_destroyed: AtomicU64,
    /// Currently active connections
    pub active_connections: AtomicUsize,
    /// Currently idle connections
    pub idle_connections: AtomicUsize,
    /// Total successful sends
    pub successful_sends: AtomicU64,
    /// Total failed sends
    pub failed_sends: AtomicU64,
    /// Total acquire timeouts
    pub acquire_timeouts: AtomicU64,
    /// Average connection age
    pub avg_connection_age_ms: AtomicU64,
    /// Health check failures
    pub health_check_failures: AtomicU64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: AtomicU64,
}

impl PoolMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> PoolMetricsSnapshot {
        PoolMetricsSnapshot {
            connections_created: self.connections_created.load(Ordering::Relaxed),
            connections_destroyed: self.connections_destroyed.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            idle_connections: self.idle_connections.load(Ordering::Relaxed),
            successful_sends: self.successful_sends.load(Ordering::Relaxed),
            failed_sends: self.failed_sends.load(Ordering::Relaxed),
            acquire_timeouts: self.acquire_timeouts.load(Ordering::Relaxed),
            avg_connection_age_ms: self.avg_connection_age_ms.load(Ordering::Relaxed),
            health_check_failures: self.health_check_failures.load(Ordering::Relaxed),
            circuit_breaker_trips: self.circuit_breaker_trips.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of pool metrics at a point in time
#[derive(Debug, Clone)]
pub struct PoolMetricsSnapshot {
    pub connections_created: u64,
    pub connections_destroyed: u64,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub successful_sends: u64,
    pub failed_sends: u64,
    pub acquire_timeouts: u64,
    pub avg_connection_age_ms: u64,
    pub health_check_failures: u64,
    pub circuit_breaker_trips: u64,
}

impl PoolMetricsSnapshot {
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_sends + self.failed_sends;
        if total == 0 {
            1.0
        } else {
            self.successful_sends as f64 / total as f64
        }
    }

    pub fn total_connections(&self) -> usize {
        self.active_connections + self.idle_connections
    }
}

/// Circuit breaker for handling connection failures
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: usize,
    last_failure_time: Option<Instant>,
    config: EnhancedPoolConfig,
}

impl CircuitBreaker {
    fn new(config: EnhancedPoolConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            config,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn record_failure(&mut self) -> bool {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.config.circuit_breaker_failure_threshold {
            self.state = CircuitBreakerState::Open;
            true // Circuit breaker tripped
        } else {
            false
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.circuit_breaker_recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn get_state(&self) -> CircuitBreakerState {
        self.state.clone()
    }
}

/// Enhanced SMTP connection pool with health checks and circuit breaker
pub struct EnhancedSmtpPool {
    config: EnhancedPoolConfig,
    transport: SmtpTransport,
    credentials: Option<Credentials>,
    pool: Arc<Mutex<VecDeque<EnhancedPooledConnection>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<PoolMetrics>,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl EnhancedSmtpPool {
    /// Create a new enhanced SMTP connection pool
    pub fn new(transport: SmtpTransport) -> Self {
        let config = EnhancedPoolConfig::default();
        Self::with_config(transport, config)
    }

    /// Create a new enhanced SMTP connection pool with custom configuration
    pub fn with_config(transport: SmtpTransport, config: EnhancedPoolConfig) -> Self {
        let pool = Self {
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            transport,
            credentials: None,
            pool: Arc::new(Mutex::new(VecDeque::new())),
            metrics: Arc::new(PoolMetrics::new()),
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(config.clone()))),
            shutdown_tx: Arc::new(Mutex::new(None)),
            config,
        };

        // Start background tasks
        pool.start_background_tasks();
        pool
    }

    /// Set authentication credentials for all connections
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Send an email using a pooled connection with retry logic
    pub async fn send(&self, message: &Message) -> Result<()> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.lock().await;
            if !cb.can_execute() {
                self.metrics.failed_sends.fetch_add(1, Ordering::Relaxed);
                return Err(Error::Custom("Circuit breaker is open".to_string()));
            }
        }

        let mut last_error = None;
        
        for attempt in 0..self.config.max_retry_attempts {
            match self.send_attempt(message).await {
                Ok(()) => {
                    // Record success in circuit breaker
                    {
                        let mut cb = self.circuit_breaker.lock().await;
                        cb.record_success();
                    }
                    self.metrics.successful_sends.fetch_add(1, Ordering::Relaxed);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retry_attempts - 1 {
                        let backoff = Duration::from_millis(
                            (1000.0 * self.config.retry_backoff_multiplier.powi(attempt as i32)) as u64
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        // Record failure in circuit breaker
        {
            let mut cb = self.circuit_breaker.lock().await;
            if cb.record_failure() {
                self.metrics.circuit_breaker_trips.fetch_add(1, Ordering::Relaxed);
            }
        }

        self.metrics.failed_sends.fetch_add(1, Ordering::Relaxed);
        Err(last_error.unwrap_or_else(|| Error::Custom("Unknown error".to_string())))
    }

    /// Single send attempt
    async fn send_attempt(&self, message: &Message) -> Result<()> {
        let mut connection = self.acquire_connection().await?;
        
        // Get sender and recipients
        let from_email = message.from_address()
            .ok_or_else(|| Error::Custom("Missing From header".to_string()))?;
        
        let recipients = message.recipients();
        if recipients.is_empty() {
            return Err(Error::Custom("No recipients specified".to_string()));
        }

        // Send email using the connection
        let result = self.send_with_connection(&mut connection.connection, message, &from_email, &recipients).await;
        
        // Update connection health based on result
        match &result {
            Ok(()) => connection.record_success(),
            Err(_) => connection.record_failure(),
        }

        // Return connection to pool
        self.release_connection(connection).await;
        
        result
    }

    /// Acquire a connection from the pool or create a new one
    async fn acquire_connection(&self) -> Result<EnhancedPooledConnection> {
        // Wait for semaphore permit
        let _permit = match timeout(self.config.acquire_timeout, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(Error::Custom("Semaphore closed".to_string())),
            Err(_) => {
                self.metrics.acquire_timeouts.fetch_add(1, Ordering::Relaxed);
                return Err(Error::Custom("Timeout waiting for connection".to_string()));
            }
        };

        // Try to get a healthy connection from the pool
        if let Some(mut pooled_conn) = self.pop_healthy_connection().await {
            pooled_conn.update_last_used();
            self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
            self.metrics.idle_connections.fetch_sub(1, Ordering::Relaxed);
            return Ok(pooled_conn);
        }

        // Create a new connection
        match self.create_new_connection().await {
            Ok(connection) => {
                let pooled_conn = EnhancedPooledConnection::new(connection);
                self.metrics.connections_created.fetch_add(1, Ordering::Relaxed);
                self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
                Ok(pooled_conn)
            }
            Err(e) => Err(e),
        }
    }

    /// Pop a healthy connection from the pool
    async fn pop_healthy_connection(&self) -> Option<EnhancedPooledConnection> {
        let mut pool = self.pool.lock().await;
        let mut removed_count = 0;
        
        // Find first healthy connection, removing expired/unhealthy ones
        while let Some(conn) = pool.pop_front() {
            if conn.is_idle_expired(self.config.max_idle_time) || 
               conn.is_aged(self.config.max_connection_age) ||
               !conn.is_healthy() {
                removed_count += 1;
                continue;
            }
            
            // Found a good connection
            if removed_count > 0 {
                self.metrics.connections_destroyed.fetch_add(removed_count, Ordering::Relaxed);
                self.metrics.idle_connections.fetch_sub(removed_count as usize, Ordering::Relaxed);
            }
            return Some(conn);
        }

        // Update metrics for removed connections
        if removed_count > 0 {
            self.metrics.connections_destroyed.fetch_add(removed_count, Ordering::Relaxed);
            self.metrics.idle_connections.fetch_sub(removed_count as usize, Ordering::Relaxed);
        }

        None
    }

    /// Create a new SMTP connection
    async fn create_new_connection(&self) -> Result<SmtpConnection> {
        let mut client = SmtpClient::new(self.transport.clone());
        
        if let Some(ref creds) = self.credentials {
            client = client.credentials(creds.clone());
        }

        timeout(self.config.connection_timeout, client.connect())
            .await
            .map_err(|_| Error::Custom("Connection timeout".to_string()))?
    }

    /// Release a connection back to the pool
    async fn release_connection(&self, mut connection: EnhancedPooledConnection) {
        // Update last used time
        connection.update_last_used();
        
        // Check if pool has space and connection is healthy
        let mut pool = self.pool.lock().await;
        if pool.len() < self.config.max_connections && connection.is_healthy() {
            pool.push_back(connection);
            self.metrics.idle_connections.fetch_add(1, Ordering::Relaxed);
        } else {
            // Connection will be dropped
            self.metrics.connections_destroyed.fetch_add(1, Ordering::Relaxed);
        }
        
        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Send email with a specific connection
    async fn send_with_connection(
        &self,
        connection: &mut SmtpConnection,
        message: &Message,
        from_email: &str,
        recipients: &[String],
    ) -> Result<()> {
        // Send MAIL FROM command
        connection.mail_from(from_email).await?;

        // Send RCPT TO commands
        for recipient in recipients {
            connection.rcpt_to(recipient).await?;
        }

        // Send DATA command and message content
        connection.data().await?;
        let message_data = message.format()?;
        connection.write_data(&message_data).await?;

        Ok(())
    }

    /// Get current pool metrics
    pub fn metrics(&self) -> PoolMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Get circuit breaker state
    pub async fn circuit_breaker_state(&self) -> CircuitBreakerState {
        let cb = self.circuit_breaker.lock().await;
        cb.get_state()
    }

    /// Perform health check on all connections
    pub async fn health_check(&self) -> Result<()> {
        let mut pool = self.pool.lock().await;
        let mut healthy_connections = VecDeque::new();
        let mut removed_count = 0;

        while let Some(mut conn) = pool.pop_front() {
            if self.config.validate_connections && conn.needs_health_check(self.config.health_check_interval) {
                // Perform basic health check (ping)
                match self.validate_connection(&mut conn.connection).await {
                    Ok(()) => {
                        conn.update_health_check();
                        conn.record_success();
                        healthy_connections.push_back(conn);
                    }
                    Err(_) => {
                        conn.record_failure();
                        if conn.is_healthy() {
                            healthy_connections.push_back(conn);
                        } else {
                            removed_count += 1;
                            self.metrics.health_check_failures.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            } else {
                healthy_connections.push_back(conn);
            }
        }

        *pool = healthy_connections;
        
        if removed_count > 0 {
            self.metrics.connections_destroyed.fetch_add(removed_count, Ordering::Relaxed);
            self.metrics.idle_connections.fetch_sub(removed_count as usize, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Validate a connection by sending a NOOP command
    async fn validate_connection(&self, connection: &mut SmtpConnection) -> Result<()> {
        let is_alive = timeout(Duration::from_secs(5), connection.is_alive())
            .await
            .map_err(|_| Error::Custom("Health check timeout".to_string()))?;
        
        if is_alive {
            Ok(())
        } else {
            Err(Error::Custom("Connection validation failed".to_string()))
        }
    }

    /// Start background maintenance tasks
    fn start_background_tasks(&self) {
        let (shutdown_tx, _) = broadcast::channel(1);
        
        // Store shutdown sender
        {
            let mut tx = self.shutdown_tx.blocking_lock();
            *tx = Some(shutdown_tx.clone());
        }

        // Health check task
        let pool_clone = self.pool.clone();
        let config_clone = self.config.clone();
        let metrics_clone = self.metrics.clone();
        let transport_clone = self.transport.clone();
        let credentials_clone = self.credentials.clone();
        let mut shutdown_rx1 = shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            let mut interval = interval(config_clone.health_check_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform health checks and maintenance
                        let health_check_task = Self::background_health_check(
                            pool_clone.clone(),
                            config_clone.clone(),
                            metrics_clone.clone(),
                            transport_clone.clone(),
                            credentials_clone.clone(),
                        );
                        
                        if let Err(e) = health_check_task.await {
                            eprintln!("Health check error: {}", e);
                        }
                    }
                    _ = shutdown_rx1.recv() => {
                        break;
                    }
                }
            }
        });

        // Metrics update task
        let pool_clone2 = self.pool.clone();
        let metrics_clone2 = self.metrics.clone();
        let mut shutdown_rx2 = shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::update_metrics(pool_clone2.clone(), metrics_clone2.clone()).await;
                    }
                    _ = shutdown_rx2.recv() => {
                        break;
                    }
                }
            }
        });
    }

    /// Background health check task
    async fn background_health_check(
        pool: Arc<Mutex<VecDeque<EnhancedPooledConnection>>>,
        config: EnhancedPoolConfig,
        metrics: Arc<PoolMetrics>,
        transport: SmtpTransport,
        credentials: Option<Credentials>,
    ) -> Result<()> {
        let mut pool_guard = pool.lock().await;
        let mut healthy_connections = VecDeque::new();
        let mut removed_count = 0;

        // Check existing connections
        while let Some(conn) = pool_guard.pop_front() {
            if conn.is_idle_expired(config.max_idle_time) || 
               conn.is_aged(config.max_connection_age) ||
               !conn.is_healthy() {
                removed_count += 1;
                continue;
            }
            healthy_connections.push_back(conn);
        }

        // Ensure minimum connections
        let current_count = healthy_connections.len();
        if current_count < config.min_connections {
            for _ in current_count..config.min_connections {
                match Self::create_connection(transport.clone(), credentials.clone()).await {
                    Ok(connection) => {
                        let pooled_conn = EnhancedPooledConnection::new(connection);
                        healthy_connections.push_back(pooled_conn);
                        metrics.connections_created.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => break, // Stop trying if connection creation fails
                }
            }
        }

        *pool_guard = healthy_connections;
        
        if removed_count > 0 {
            metrics.connections_destroyed.fetch_add(removed_count, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Create a connection helper
    async fn create_connection(
        transport: SmtpTransport,
        credentials: Option<Credentials>,
    ) -> Result<SmtpConnection> {
        let mut client = SmtpClient::new(transport);
        
        if let Some(creds) = credentials {
            client = client.credentials(creds);
        }

        client.connect().await
    }

    /// Update pool metrics
    async fn update_metrics(
        pool: Arc<Mutex<VecDeque<EnhancedPooledConnection>>>,
        metrics: Arc<PoolMetrics>,
    ) {
        let pool_guard = pool.lock().await;
        
        // Update idle connections count
        metrics.idle_connections.store(pool_guard.len(), Ordering::Relaxed);
        
        // Calculate average connection age
        if !pool_guard.is_empty() {
            let total_age: u64 = pool_guard.iter()
                .map(|conn| conn.created_at.elapsed().as_millis() as u64)
                .sum();
            let avg_age = total_age / pool_guard.len() as u64;
            metrics.avg_connection_age_ms.store(avg_age, Ordering::Relaxed);
        }
    }

    /// Shutdown the pool and cleanup resources
    pub async fn shutdown(&self) {
        if let Some(shutdown_tx) = self.shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
        }
        
        // Close all connections
        let mut pool = self.pool.lock().await;
        let count = pool.len();
        pool.clear();
        self.metrics.connections_destroyed.fetch_add(count as u64, Ordering::Relaxed);
        self.metrics.idle_connections.store(0, Ordering::Relaxed);
    }
}

impl Drop for EnhancedSmtpPool {
    fn drop(&mut self) {
        // Attempt graceful shutdown
        if let Ok(mut tx_guard) = self.shutdown_tx.try_lock() {
            if let Some(shutdown_tx) = tx_guard.take() {
                let _ = shutdown_tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_pool_config() {
        let config = EnhancedPoolConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert!(config.validate_connections);
    }

    #[test]
    fn test_pool_metrics() {
        let metrics = PoolMetrics::new();
        metrics.successful_sends.fetch_add(10, Ordering::Relaxed);
        metrics.failed_sends.fetch_add(2, Ordering::Relaxed);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.successful_sends, 10);
        assert_eq!(snapshot.failed_sends, 2);
        assert_eq!(snapshot.success_rate(), 10.0 / 12.0);
    }

    #[test]
    fn test_circuit_breaker() {
        let config = EnhancedPoolConfig::default();
        let mut cb = CircuitBreaker::new(config);
        
        assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
        assert!(cb.can_execute());
        
        // Record failures
        for _ in 0..5 {
            cb.record_failure();
        }
        
        assert_eq!(cb.get_state(), CircuitBreakerState::Open);
        assert!(!cb.can_execute());
    }
}