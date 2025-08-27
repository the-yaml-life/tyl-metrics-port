//! # TYL Metrics Port
//!
//! **Hexagonal Architecture Port** for metrics collection in the TYL framework.
//!
//! This module defines the **pure interface** for metrics collection without any implementation details.
//! Following the hexagonal architecture pattern, it provides:
//!
//! - **Port Interface**: `MetricsManager` trait
//! - **Domain Types**: Core metrics types and value objects
//! - **Mock Adapter**: Simple implementation for testing and examples
//!
//! ## Architecture Philosophy
//!
//! This is a **PORT** - it defines **WHAT** can be done, not **HOW**.
//! Concrete implementations (adapters) are in separate modules:
//!
//! - `tyl-prometheus-metrics-adapter` - Prometheus implementation
//! - `tyl-otel-metrics-adapter` - OpenTelemetry implementation (future)
//! - Additional adapters can be created by implementing `MetricsManager`
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_metrics_port::{MetricsManager, MetricRequest, MetricType};
//!
//! // In your application, inject any adapter that implements MetricsManager
//! async fn record_metrics<M: MetricsManager>(metrics: &M) -> tyl_metrics_port::Result<()> {
//!     let request = MetricRequest::counter("http_requests", 1.0)
//!         .with_label("method", "GET")
//!         .with_label("status", "200");
//!     
//!     metrics.record(&request).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Mock for Testing
//!
//! ```rust
//! use tyl_metrics_port::MockMetricsAdapter;
//!
//! #[tokio::test]
//! async fn test_metrics_collection() {
//!     let metrics = MockMetricsAdapter::new();
//!     // Use in tests...
//! }
//! ```

// Re-export TYL framework functionality (CRITICAL pattern)
pub use tyl_config::{ConfigManager, ConfigPlugin};
pub use tyl_errors::{TylError, TylResult};
pub use tyl_logging::Environment;

// Core port interface
mod port;
pub use port::{HealthStatus, MetricsManager};

// Domain types (port concern)
mod types;
pub use types::{Labels, MetricRequest, MetricSnapshot, MetricType, MetricValue, TimerGuard};

// Error helpers for metrics domain
mod errors;
pub use errors::{
    from_io_error, from_serde_json_error, metrics_adapter_error, metrics_config_error,
    metrics_connection_error, metrics_error, metrics_health_error, metrics_recording_error,
    metrics_serialization_error, metrics_timeout_error, MetricsErrorExt,
};

// Utilities and validation (port concern)
mod utils;
pub use utils::{format_labels, validate_metric_name, normalize_metric_name};

// Mock adapter for testing and examples
#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use mock::{MockMetricsAdapter, MockMetricsConfig};

// Always expose mock for examples and testing
#[cfg(not(feature = "mock"))]
mod mock;
pub use mock::{MockMetricsAdapter, MockMetricsConfig};

/// Result type for metrics operations using TYL error handling
pub type Result<T> = TylResult<T>;

/// Re-export async_trait for adapter implementations
pub use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockMetricsConfig;

    #[tokio::test]
    async fn test_port_basic_functionality() {
        let metrics = MockMetricsAdapter::new(MockMetricsConfig::default());

        let request = MetricRequest::counter("test_metric", 1.0).with_label("test", "true");

        let result = metrics.record(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_port_health_check() {
        let metrics = MockMetricsAdapter::new(MockMetricsConfig::default());
        let health = metrics.health_check().await;
        assert!(health.is_ok());
    }

    #[test]
    fn test_metric_request_builder() {
        let request = MetricRequest::gauge("memory_usage", 512.0)
            .with_label("unit", "MB")
            .with_label("server", "web-01");

        assert_eq!(request.name(), "memory_usage");
        assert_eq!(request.metric_type(), &MetricType::Gauge);
        assert_eq!(request.value(), 512.0);
        assert_eq!(request.labels().len(), 2);
    }

    #[test]
    fn test_tyl_error_integration() {
        let error = metrics_error("test_metric", "Invalid metric name");
        assert!(error.to_string().contains("Invalid metric name"));
    }

    #[test]
    fn test_validation() {
        assert!(validate_metric_name("valid_metric").is_ok());
        assert!(validate_metric_name("").is_err());
        assert!(validate_metric_name("invalid name").is_err());
    }
}
