//! Event Producer
//!
//! Kafka producer for publishing events to the event-driven pipeline.
//! Replaces direct HTTP calls with event publishing for better resilience.

use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Kafka error: {0}")]
    Kafka(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

/// Configuration for the event producer
#[derive(Clone, Debug)]
pub struct ProducerConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub retries: u32,
    pub retry_backoff_ms: u64,
    pub request_timeout_ms: u64,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            client_id: "api-backend".to_string(),
            retries: 5,
            retry_backoff_ms: 100,
            request_timeout_ms: 30000,
        }
    }
}

impl ProducerConfig {
    pub fn from_env() -> Self {
        Self {
            bootstrap_servers: std::env::var("KAFKA_BOOTSTRAP_SERVERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            client_id: std::env::var("KAFKA_CLIENT_ID")
                .unwrap_or_else(|_| "api-backend".to_string()),
            retries: std::env::var("KAFKA_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            retry_backoff_ms: std::env::var("KAFKA_RETRY_BACKOFF_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            request_timeout_ms: std::env::var("KAFKA_REQUEST_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30000),
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failures: u32,
    threshold: u32,
    recovery_timeout: Duration,
    last_failure: Option<std::time::Instant>,
}

impl CircuitBreaker {
    fn new(threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failures: 0,
            threshold,
            recovery_timeout,
            last_failure: None,
        }
    }

    fn record_success(&mut self) {
        self.failures = 0;
        self.state = CircuitState::Closed;
    }

    fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(std::time::Instant::now());
        
        if self.failures >= self.threshold {
            self.state = CircuitState::Open;
            warn!("Circuit breaker opened after {} failures", self.failures);
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        info!("Circuit breaker half-open, allowing test request");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// Event producer for Kafka
/// 
/// Publishes events to Kafka topics with:
/// - Automatic retry with exponential backoff
/// - Circuit breaker for fault tolerance
/// - Zstandard compression
/// - Correlation ID tracking
pub struct EventProducer {
    producer: FutureProducer,
    config: ProducerConfig,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
}

impl EventProducer {
    /// Create a new event producer
    pub fn new(config: ProducerConfig) -> Result<Self, ProducerError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("client.id", &config.client_id)
            .set("compression.type", "zstd")
            .set("acks", "all")
            .set("enable.idempotence", "true")
            .set("request.timeout.ms", config.request_timeout_ms.to_string())
            .create()
            .map_err(|e| ProducerError::Kafka(e.to_string()))?;

        info!(
            "Kafka event producer created: bootstrap_servers={}",
            config.bootstrap_servers
        );

        Ok(Self {
            producer,
            config,
            circuit_breaker: Arc::new(RwLock::new(CircuitBreaker::new(5, Duration::from_secs(30)))),
        })
    }

    /// Publish an event to Kafka
    pub async fn publish<E: Serialize>(
        &self,
        topic: &str,
        event: &E,
        correlation_id: Option<&str>,
    ) -> Result<String, ProducerError> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            if !cb.can_execute() {
                return Err(ProducerError::CircuitBreakerOpen);
            }
        }

        let key = correlation_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let value = serde_json::to_vec(event)?;

        match self.publish_with_retry(topic, &key, &value).await {
            Ok(_) => {
                let mut cb = self.circuit_breaker.write().await;
                cb.record_success();
                
                debug!(
                    "Published event: topic={}, correlation_id={}",
                    topic, key
                );
                Ok(key)
            }
            Err(e) => {
                let mut cb = self.circuit_breaker.write().await;
                cb.record_failure();
                Err(e)
            }
        }
    }

    async fn publish_with_retry(
        &self,
        topic: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), ProducerError> {
        let mut last_error = None;

        for attempt in 0..=self.config.retries {
            let record = FutureRecord::to(topic)
                .key(key)
                .payload(value);

            match self.producer.send(
                record,
                Timeout::After(Duration::from_millis(self.config.request_timeout_ms)),
            ).await {
                Ok(_) => return Ok(()),
                Err((e, _)) => {
                    last_error = Some(e.to_string());
                    
                    if attempt < self.config.retries {
                        let backoff = self.config.retry_backoff_ms * (2_u64.pow(attempt));
                        warn!(
                            "Kafka send failed, retrying in {}ms (attempt {}): {}",
                            backoff, attempt + 1, e
                        );
                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                    }
                }
            }
        }

        error!("Max retries exceeded for topic {}", topic);
        Err(ProducerError::Kafka(
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }

    /// Check if the producer is healthy
    pub async fn is_healthy(&self) -> bool {
        let cb = self.circuit_breaker.read().await;
        cb.state != CircuitState::Open
    }
}
