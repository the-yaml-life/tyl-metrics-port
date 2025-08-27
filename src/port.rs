//! Core port interface for metrics collection
//!
//! This module defines the pure interface contract that all metrics adapters must implement.
//! Following hexagonal architecture principles, this port defines WHAT can be done,
//! not HOW it's implemented.

use super::*;
use async_trait::async_trait;
use std::time::Duration;

/// **Primary Port Interface** for metrics collection
///
/// This trait defines the contract that all metrics adapters must implement.
/// It provides a clean, adapter-agnostic interface for recording metrics.
///
/// ## Design Principles
/// - **Async-first**: All operations are async for non-blocking metrics recording
/// - **Generic Configuration**: Each adapter defines its own config type
/// - **TYL Integration**: Uses TYL error handling and patterns
/// - **Dependency Injection**: Adapters are injected via constructor or factory
///
/// ## Example Implementation
/// ```rust
/// use tyl_metrics_port::{MetricsManager, MetricRequest, Result, async_trait};
/// use std::collections::HashMap;
///
/// pub struct MyMetricsAdapter {
///     config: MyConfig,
/// }
///
/// #[async_trait]
/// impl MetricsManager for MyMetricsAdapter {
///     type Config = MyConfig;
///     
///     async fn new(config: Self::Config) -> Result<Self> {
///         Ok(Self { config })
///     }
///     
///     async fn record(&self, request: &MetricRequest) -> Result<()> {
///         // Implementation specific logic
///         Ok(())
///     }
///     
///     async fn health_check(&self) -> Result<HealthStatus> {
///         Ok(HealthStatus::healthy())
///     }
/// }
/// ```
#[async_trait]
pub trait MetricsManager: Send + Sync {
    /// Configuration type specific to this adapter
    type Config: Send + Sync;
    
    /// Create a new metrics adapter instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Adapter-specific configuration
    ///
    /// # Returns
    /// * `Result<Self>` - The configured metrics adapter instance
    async fn new(config: Self::Config) -> Result<Self>
    where
        Self: Sized;
    
    /// Record a metric event
    ///
    /// This is the primary method for recording metrics. The adapter implementation
    /// determines how the metric is actually stored, transmitted, or processed.
    ///
    /// # Arguments
    /// * `request` - The metric request containing all necessary information
    ///
    /// # Returns
    /// * `Result<()>` - Success or error using TYL error handling
    async fn record(&self, request: &MetricRequest) -> Result<()>;
    
    /// Start a timer and return a guard that records duration when dropped
    ///
    /// This provides a convenient RAII pattern for measuring durations.
    /// The timer is automatically recorded when the guard is dropped.
    ///
    /// # Arguments
    /// * `name` - The metric name for the timer
    /// * `labels` - Labels to attach to the timer metric
    ///
    /// # Returns
    /// * `TimerGuard` - RAII guard that records duration on drop
    fn start_timer(&self, name: &str, labels: Labels) -> TimerGuard;
    
    /// Check the health status of the metrics adapter
    ///
    /// This method allows monitoring systems to verify that the metrics
    /// collection system is functioning correctly.
    ///
    /// # Returns
    /// * `Result<HealthStatus>` - Current health status or error
    async fn health_check(&self) -> Result<HealthStatus>;
    
    /// Get current metrics snapshot (optional, primarily for debugging)
    ///
    /// Not all adapters may implement this meaningfully (e.g., push-based systems
    /// like OpenTelemetry may return empty results).
    ///
    /// # Returns
    /// * `Result<Vec<MetricSnapshot>>` - Current metrics or empty if not applicable
    async fn get_snapshot(&self) -> Result<Vec<MetricSnapshot>> {
        // Default implementation returns empty - push-based systems don't store metrics
        Ok(Vec::new())
    }
}

/// Health status information for metrics adapters
#[derive(Debug, Clone, PartialEq)]
pub struct HealthStatus {
    /// Whether the adapter is healthy
    pub is_healthy: bool,
    
    /// Human-readable status message
    pub message: String,
    
    /// Optional additional metadata
    pub metadata: std::collections::HashMap<String, String>,
    
    /// Timestamp of health check (Unix epoch seconds)
    pub timestamp: u64,
}

impl HealthStatus {
    /// Create a healthy status
    pub fn healthy() -> Self {
        Self {
            is_healthy: true,
            message: "Metrics adapter is healthy".to_string(),
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Create an unhealthy status with a message
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            is_healthy: false,
            message: message.into(),
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Add metadata to the health status
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::healthy()
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.is_healthy { "HEALTHY" } else { "UNHEALTHY" };
        write!(f, "[{}] {}", status, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::healthy();
        assert!(status.is_healthy);
        assert!(status.message.contains("healthy"));
        assert!(status.timestamp > 0);
    }
    
    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::unhealthy("Connection failed");
        assert!(!status.is_healthy);
        assert_eq!(status.message, "Connection failed");
        assert!(status.timestamp > 0);
    }
    
    #[test]
    fn test_health_status_with_metadata() {
        let status = HealthStatus::healthy()
            .with_metadata("version", "1.0.0")
            .with_metadata("endpoint", "localhost:9090");
            
        assert_eq!(status.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(status.metadata.get("endpoint"), Some(&"localhost:9090".to_string()));
    }
    
    #[test]
    fn test_health_status_display() {
        let healthy = HealthStatus::healthy();
        assert!(healthy.to_string().contains("[HEALTHY]"));
        
        let unhealthy = HealthStatus::unhealthy("Error occurred");
        assert!(unhealthy.to_string().contains("[UNHEALTHY]"));
        assert!(unhealthy.to_string().contains("Error occurred"));
    }
}