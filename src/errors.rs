//! Error handling integration for TYL metrics
//!
//! This module provides helper functions for creating domain-specific errors
//! using the TYL framework error system. It follows the established TYL
//! pattern of providing semantic error constructors rather than custom error types.

use super::*;

/// Error category for metrics-related errors
pub const METRICS_ERROR_CATEGORY: &str = "metrics";

/// Create a metrics validation error
///
/// Used when metric names, values, or other inputs fail validation.
///
/// # Arguments
/// * `field` - The field that failed validation
/// * `message` - Description of the validation failure
///
/// # Returns
/// * `TylError` - Structured error with metrics context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_error;
///
/// let error = metrics_error("metric_name", "Names cannot contain spaces");
/// ```
pub fn metrics_error(field: impl Into<String>, message: impl Into<String>) -> TylError {
    TylError::validation(field.into(), message.into())
}

/// Create a metrics configuration error
///
/// Used when there are issues with metrics adapter configuration.
///
/// # Arguments
/// * `config_key` - The configuration key that caused the error
/// * `message` - Description of the configuration issue
///
/// # Returns
/// * `TylError` - Structured error with configuration context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_config_error;
///
/// let error = metrics_config_error("prometheus.port", "Port must be between 1024 and 65535");
/// ```
pub fn metrics_config_error(config_key: impl Into<String>, message: impl Into<String>) -> TylError {
    TylError::configuration(format!(
        "Metrics config error for {}: {}",
        config_key.into(),
        message.into()
    ))
}

/// Create a metrics connection error
///
/// Used when adapters fail to connect to external systems like Prometheus or OTEL endpoints.
///
/// # Arguments
/// * `endpoint` - The endpoint that failed to connect
/// * `message` - Description of the connection failure
///
/// # Returns
/// * `TylError` - Structured error with connection context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_connection_error;
///
/// let error = metrics_connection_error("http://localhost:9090", "Connection refused");
/// ```
pub fn metrics_connection_error(
    endpoint: impl Into<String>,
    message: impl Into<String>,
) -> TylError {
    TylError::network(format!(
        "Metrics connection error to {}: {}",
        endpoint.into(),
        message.into()
    ))
}

/// Create a metrics recording error
///
/// Used when the actual recording of a metric fails within an adapter.
///
/// # Arguments
/// * `metric_name` - The name of the metric that failed to record
/// * `message` - Description of why recording failed
///
/// # Returns
/// * `TylError` - Structured error with recording context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_recording_error;
///
/// let error = metrics_recording_error("http_requests_total", "Metric registry full");
/// ```
pub fn metrics_recording_error(
    metric_name: impl Into<String>,
    message: impl Into<String>,
) -> TylError {
    TylError::internal(format!(
        "Metrics recording error for {}: {}",
        metric_name.into(),
        message.into()
    ))
}

/// Create a metrics adapter initialization error
///
/// Used when a metrics adapter fails to initialize properly.
///
/// # Arguments
/// * `adapter_type` - The type of adapter (e.g., "prometheus", "otel")
/// * `message` - Description of the initialization failure
///
/// # Returns
/// * `TylError` - Structured error with initialization context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_adapter_error;
///
/// let error = metrics_adapter_error("prometheus", "Failed to create metric registry");
/// ```
pub fn metrics_adapter_error(
    adapter_type: impl Into<String>,
    message: impl Into<String>,
) -> TylError {
    TylError::internal(format!(
        "Metrics adapter error for {}: {}",
        adapter_type.into(),
        message.into()
    ))
}

/// Create a metrics health check error
///
/// Used when health checks fail on metrics adapters.
///
/// # Arguments
/// * `adapter_type` - The type of adapter being health checked
/// * `message` - Description of the health check failure
///
/// # Returns
/// * `TylError` - Structured error with health check context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_health_error;
///
/// let error = metrics_health_error("prometheus", "Metrics endpoint unreachable");
/// ```
pub fn metrics_health_error(
    adapter_type: impl Into<String>,
    message: impl Into<String>,
) -> TylError {
    TylError::internal(format!(
        "Metrics health check error for {}: {}",
        adapter_type.into(),
        message.into()
    ))
}

/// Create a metrics serialization error
///
/// Used when metric data fails to serialize for transmission or storage.
///
/// # Arguments
/// * `format` - The serialization format (e.g., "json", "protobuf")
/// * `message` - Description of the serialization failure
///
/// # Returns
/// * `TylError` - Structured error with serialization context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_serialization_error;
///
/// let error = metrics_serialization_error("json", "Invalid UTF-8 sequence in metric name");
/// ```
pub fn metrics_serialization_error(
    format: impl Into<String>,
    message: impl Into<String>,
) -> TylError {
    TylError::internal(format!(
        "Metrics serialization error for {}: {}",
        format.into(),
        message.into()
    ))
}

/// Create a metrics timeout error
///
/// Used when operations timeout during metric recording or transmission.
///
/// # Arguments
/// * `operation` - The operation that timed out
/// * `timeout_secs` - The timeout duration in seconds
///
/// # Returns
/// * `TylError` - Structured error with timeout context
///
/// # Example
/// ```rust
/// use tyl_metrics_port::metrics_timeout_error;
///
/// let error = metrics_timeout_error("record_batch", 5);
/// ```
pub fn metrics_timeout_error(operation: impl Into<String>, timeout_secs: u64) -> TylError {
    TylError::internal(format!(
        "Metrics timeout error for {} after {}s",
        operation.into(),
        timeout_secs
    ))
}

/// Helper trait for adding metrics context to existing errors
///
/// This trait allows adding metrics-specific context to any existing TylError,
/// which is useful when adapters need to add context to errors from dependencies.
pub trait MetricsErrorExt {
    /// Add metrics context to an existing error by wrapping it
    fn with_metrics_context(self, context: impl Into<String>) -> TylError;

    /// Add metric name context to an existing error by wrapping it
    fn with_metric_name(self, metric_name: impl Into<String>) -> TylError;

    /// Add adapter type context to an existing error by wrapping it
    fn with_adapter_type(self, adapter_type: impl Into<String>) -> TylError;
}

impl MetricsErrorExt for TylError {
    fn with_metrics_context(self, context: impl Into<String>) -> TylError {
        TylError::internal(format!("Metrics context [{}]: {}", context.into(), self))
    }

    fn with_metric_name(self, metric_name: impl Into<String>) -> TylError {
        TylError::internal(format!("Metric [{}]: {}", metric_name.into(), self))
    }

    fn with_adapter_type(self, adapter_type: impl Into<String>) -> TylError {
        TylError::internal(format!("Adapter [{}]: {}", adapter_type.into(), self))
    }
}

/// Convert common error types to metrics errors with context
///
/// Note: These are helper functions rather than From impls to avoid orphan rule issues
pub fn from_serde_json_error(error: serde_json::Error) -> TylError {
    metrics_serialization_error("json", error.to_string())
}

pub fn from_io_error(error: std::io::Error) -> TylError {
    match error.kind() {
        std::io::ErrorKind::ConnectionRefused => {
            metrics_connection_error("unknown", error.to_string())
        }
        std::io::ErrorKind::TimedOut => metrics_timeout_error("io_operation", 0),
        _ => TylError::internal(format!("Metrics IO error: {}", error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_error() {
        let error = metrics_error("metric_name", "Invalid characters");
        assert!(error.to_string().contains("Invalid characters"));
    }

    #[test]
    fn test_metrics_config_error() {
        let error = metrics_config_error("port", "Port out of range");
        assert!(error.to_string().contains("Port out of range"));
    }

    #[test]
    fn test_metrics_connection_error() {
        let error = metrics_connection_error("localhost:9090", "Connection refused");
        assert!(error.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_metrics_recording_error() {
        let error = metrics_recording_error("cpu_usage", "Registry full");
        assert!(error.to_string().contains("Registry full"));
    }

    #[test]
    fn test_metrics_adapter_error() {
        let error = metrics_adapter_error("prometheus", "Init failed");
        assert!(error.to_string().contains("Init failed"));
    }

    #[test]
    fn test_metrics_health_error() {
        let error = metrics_health_error("otel", "Endpoint unreachable");
        assert!(error.to_string().contains("Endpoint unreachable"));
    }

    #[test]
    fn test_metrics_serialization_error() {
        let error = metrics_serialization_error("protobuf", "Invalid schema");
        assert!(error.to_string().contains("Invalid schema"));
    }

    #[test]
    fn test_metrics_timeout_error() {
        let error = metrics_timeout_error("batch_send", 30);
        assert!(error.to_string().contains("batch_send"));
    }

    #[test]
    fn test_error_extension_trait() {
        let base_error = TylError::validation("test", "test message");
        let extended = base_error
            .with_metrics_context("counter recording")
            .with_metric_name("http_requests")
            .with_adapter_type("prometheus");

        let error_string = extended.to_string();
        assert!(error_string.contains("http_requests"));
        assert!(error_string.contains("prometheus"));
    }

    #[test]
    fn test_serde_json_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let tyl_error = from_serde_json_error(json_error);
        assert!(tyl_error.to_string().contains("json"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let tyl_error = from_io_error(io_error);
        assert!(tyl_error.to_string().contains("connection"));
    }

    #[test]
    fn test_timeout_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Operation timed out");
        let tyl_error = from_io_error(io_error);
        assert!(tyl_error.to_string().contains("timeout"));
    }
}
