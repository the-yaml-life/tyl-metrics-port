//! Core domain types for metrics collection
//!
//! This module defines the value objects and domain types used throughout
//! the metrics system. Following domain-driven design principles, these
//! types represent the core concepts of the metrics domain.

use crate::{Result, TylError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Type alias for metric labels - a map of string key-value pairs
pub type Labels = HashMap<String, String>;

/// Core metric request that encapsulates all information needed to record a metric
///
/// This is the primary value object that flows through the metrics system.
/// It contains all the necessary information for any adapter to record
/// the metric appropriately.
///
/// ## Design Principles
/// - **Immutable Value Object**: Cannot be modified after creation
/// - **Builder Pattern**: Fluent API for construction
/// - **Validation**: Built-in validation during construction
/// - **Serializable**: Can be serialized for persistence or transmission
///
/// ## Example Usage
/// ```rust
/// use tyl_metrics_port::{MetricRequest, MetricType};
///
/// let request = MetricRequest::counter("http_requests_total", 1.0)
///     .with_label("method", "GET")
///     .with_label("status", "200")
///     .with_label("endpoint", "/api/users");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricRequest {
    /// The metric name (must follow metric naming conventions)
    name: String,

    /// The type of metric being recorded
    metric_type: MetricType,

    /// The numeric value to record
    value: MetricValue,

    /// Labels attached to this metric
    labels: Labels,

    /// Optional help text describing what this metric measures
    help: Option<String>,

    /// Timestamp when the metric was created (Unix epoch nanoseconds)
    timestamp: u64,
}

impl MetricRequest {
    /// Create a new counter metric request
    ///
    /// # Arguments
    /// * `name` - The metric name (will be validated)
    /// * `value` - The counter increment value (must be >= 0)
    ///
    /// # Returns
    /// * `MetricRequest` - A new metric request builder
    pub fn counter(name: impl Into<String>, value: f64) -> Self {
        Self::new(name.into(), MetricType::Counter, MetricValue::Single(value))
    }

    /// Create a new gauge metric request
    ///
    /// # Arguments
    /// * `name` - The metric name (will be validated)
    /// * `value` - The gauge value
    ///
    /// # Returns
    /// * `MetricRequest` - A new metric request builder
    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self::new(name.into(), MetricType::Gauge, MetricValue::Single(value))
    }

    /// Create a new histogram metric request
    ///
    /// # Arguments
    /// * `name` - The metric name (will be validated)
    /// * `value` - The observed value to add to the histogram
    ///
    /// # Returns
    /// * `MetricRequest` - A new metric request builder
    pub fn histogram(name: impl Into<String>, value: f64) -> Self {
        Self::new(
            name.into(),
            MetricType::Histogram,
            MetricValue::Single(value),
        )
    }

    /// Create a new timer metric request
    ///
    /// # Arguments
    /// * `name` - The metric name (will be validated)
    /// * `duration` - The duration to record
    ///
    /// # Returns
    /// * `MetricRequest` - A new metric request builder
    pub fn timer(name: impl Into<String>, duration: Duration) -> Self {
        Self::new(
            name.into(),
            MetricType::Timer,
            MetricValue::Single(duration.as_secs_f64()),
        )
    }

    /// Internal constructor for creating metric requests
    fn new(name: String, metric_type: MetricType, value: MetricValue) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels: Labels::new(),
            help: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }

    /// Add a label to the metric request
    ///
    /// # Arguments
    /// * `key` - The label key
    /// * `value` - The label value
    ///
    /// # Returns
    /// * `Self` - The metric request for chaining
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add multiple labels to the metric request
    ///
    /// # Arguments
    /// * `labels` - Iterator of (key, value) pairs
    ///
    /// # Returns
    /// * `Self` - The metric request for chaining
    pub fn with_labels<I, K, V>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in labels {
            self.labels.insert(key.into(), value.into());
        }
        self
    }

    /// Add help text to the metric request
    ///
    /// # Arguments
    /// * `help` - Descriptive text about what this metric measures
    ///
    /// # Returns
    /// * `Self` - The metric request for chaining
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the metric type
    pub fn metric_type(&self) -> &MetricType {
        &self.metric_type
    }

    /// Get the metric value
    pub fn value(&self) -> f64 {
        match &self.value {
            MetricValue::Single(v) => *v,
            MetricValue::Histogram {
                sum,
                count,
                buckets: _,
            } => sum / (*count as f64),
        }
    }

    /// Get the metric value as the full value object
    pub fn metric_value(&self) -> &MetricValue {
        &self.value
    }

    /// Get the labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Get the help text if available
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// Enumeration of supported metric types
///
/// Each type represents a different way of measuring and aggregating data.
/// The choice of metric type affects how the data is stored and queried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter - Monotonically increasing value (requests, errors, bytes sent)
    Counter,

    /// Gauge - Value that can go up or down (memory usage, CPU, active connections)
    Gauge,

    /// Histogram - Statistical distribution of values (request latencies, payload sizes)
    Histogram,

    /// Timer - Duration measurements (typically converted to histograms by adapters)
    Timer,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Timer => write!(f, "timer"),
        }
    }
}

/// Metric value that can represent either simple values or histogram data
///
/// This enum allows the metrics system to handle both simple numeric values
/// and more complex histogram distributions within the same type system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Single numeric value (used for counters, gauges, and simple observations)
    Single(f64),

    /// Histogram distribution with buckets
    Histogram {
        /// Total sum of all observed values
        sum: f64,
        /// Total count of observations
        count: u64,
        /// Bucket counts for histogram distribution
        buckets: Vec<HistogramBucket>,
    },
}

/// Histogram bucket for statistical distribution
///
/// Represents a bucket in a histogram with an upper bound and count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound for this bucket (inclusive)
    pub upper_bound: f64,
    /// Number of observations that fell into this bucket
    pub count: u64,
}

/// RAII timer guard for automatic duration recording
///
/// This guard automatically records the elapsed duration when it's dropped,
/// providing a convenient way to measure execution time without manual
/// start/stop calls.
///
/// ## Example Usage
/// ```rust
/// use tyl_metrics_port::TimerGuard;
///
/// {
///     let _timer = metrics.start_timer("database_query", labels);
///     // ... perform database operation ...
///     // Duration automatically recorded when timer drops
/// }
/// ```
pub struct TimerGuard {
    /// The metric name to record to
    name: String,

    /// Labels to attach to the recorded metric
    labels: Labels,

    /// Start time for calculating duration
    start_time: Instant,

    /// Callback function to record the metric when dropped
    /// Uses trait object to abstract over different adapter types
    recorder: Box<dyn Fn(MetricRequest) + Send + Sync>,
}

impl TimerGuard {
    /// Create a new timer guard
    ///
    /// # Arguments
    /// * `name` - The metric name to record to
    /// * `labels` - Labels to attach to the metric
    /// * `recorder` - Callback function to record the metric
    pub fn new<F>(name: String, labels: Labels, recorder: F) -> Self
    where
        F: Fn(MetricRequest) + Send + Sync + 'static,
    {
        Self {
            name,
            labels,
            start_time: Instant::now(),
            recorder: Box::new(recorder),
        }
    }

    /// Get the elapsed duration so far (without stopping the timer)
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Manually record the timer and consume the guard
    pub fn record(self) {
        // Dropping will trigger the recording
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let request = MetricRequest::timer(self.name.clone(), duration)
            .with_labels(self.labels.iter().map(|(k, v)| (k.as_str(), v.as_str())));

        (self.recorder)(request);
    }
}

/// Snapshot of a metric at a point in time
///
/// Used primarily for debugging and testing. Some adapters (push-based systems
/// like OpenTelemetry) may not be able to provide meaningful snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// The metric name
    pub name: String,

    /// The metric type
    pub metric_type: MetricType,

    /// The current value
    pub value: MetricValue,

    /// Labels attached to this metric
    pub labels: Labels,

    /// Optional help text
    pub help: Option<String>,

    /// Timestamp of this snapshot (Unix epoch nanoseconds)
    pub timestamp: u64,
}

impl MetricSnapshot {
    /// Create a new metric snapshot
    pub fn new(name: String, metric_type: MetricType, value: MetricValue, labels: Labels) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels,
            help: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }

    /// Add help text to the snapshot
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

impl From<&MetricRequest> for MetricSnapshot {
    fn from(request: &MetricRequest) -> Self {
        Self {
            name: request.name.clone(),
            metric_type: request.metric_type,
            value: request.value.clone(),
            labels: request.labels.clone(),
            help: request.help.clone(),
            timestamp: request.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metric_request_counter() {
        let request = MetricRequest::counter("http_requests", 1.0);
        assert_eq!(request.name(), "http_requests");
        assert_eq!(request.metric_type(), &MetricType::Counter);
        assert_eq!(request.value(), 1.0);
        assert!(request.labels().is_empty());
    }

    #[test]
    fn test_metric_request_with_labels() {
        let request = MetricRequest::gauge("memory_usage", 512.0)
            .with_label("unit", "MB")
            .with_label("server", "web-01");

        assert_eq!(request.labels().len(), 2);
        assert_eq!(request.labels().get("unit"), Some(&"MB".to_string()));
        assert_eq!(request.labels().get("server"), Some(&"web-01".to_string()));
    }

    #[test]
    fn test_metric_request_with_multiple_labels() {
        let labels = vec![("method", "GET"), ("status", "200")];
        let request = MetricRequest::counter("requests", 1.0).with_labels(labels);

        assert_eq!(request.labels().len(), 2);
        assert_eq!(request.labels().get("method"), Some(&"GET".to_string()));
        assert_eq!(request.labels().get("status"), Some(&"200".to_string()));
    }

    #[test]
    fn test_metric_request_with_help() {
        let request = MetricRequest::histogram("request_duration", 0.25)
            .with_help("Time spent processing HTTP requests");

        assert_eq!(request.help(), Some("Time spent processing HTTP requests"));
    }

    #[test]
    fn test_metric_request_timer() {
        let duration = Duration::from_millis(150);
        let request = MetricRequest::timer("db_query", duration);

        assert_eq!(request.metric_type(), &MetricType::Timer);
        assert_eq!(request.value(), 0.15); // 150ms as seconds
    }

    #[test]
    fn test_metric_types_display() {
        assert_eq!(MetricType::Counter.to_string(), "counter");
        assert_eq!(MetricType::Gauge.to_string(), "gauge");
        assert_eq!(MetricType::Histogram.to_string(), "histogram");
        assert_eq!(MetricType::Timer.to_string(), "timer");
    }

    #[test]
    fn test_histogram_bucket() {
        let bucket = HistogramBucket {
            upper_bound: 1.0,
            count: 42,
        };

        assert_eq!(bucket.upper_bound, 1.0);
        assert_eq!(bucket.count, 42);
    }

    #[test]
    fn test_metric_value_single() {
        let value = MetricValue::Single(123.45);
        match value {
            MetricValue::Single(v) => assert_eq!(v, 123.45),
            _ => panic!("Expected single value"),
        }
    }

    #[test]
    fn test_metric_value_histogram() {
        let buckets = vec![
            HistogramBucket {
                upper_bound: 0.1,
                count: 10,
            },
            HistogramBucket {
                upper_bound: 1.0,
                count: 25,
            },
            HistogramBucket {
                upper_bound: 10.0,
                count: 35,
            },
        ];

        let value = MetricValue::Histogram {
            sum: 45.0,
            count: 35,
            buckets,
        };

        match value {
            MetricValue::Histogram {
                sum,
                count,
                buckets,
            } => {
                assert_eq!(sum, 45.0);
                assert_eq!(count, 35);
                assert_eq!(buckets.len(), 3);
            }
            _ => panic!("Expected histogram value"),
        }
    }

    #[test]
    fn test_metric_snapshot_creation() {
        let labels = vec![("env", "test")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let snapshot = MetricSnapshot::new(
            "test_metric".to_string(),
            MetricType::Counter,
            MetricValue::Single(42.0),
            labels,
        )
        .with_help("Test metric for unit tests");

        assert_eq!(snapshot.name, "test_metric");
        assert_eq!(snapshot.metric_type, MetricType::Counter);
        assert_eq!(
            snapshot.help,
            Some("Test metric for unit tests".to_string())
        );
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_metric_snapshot_from_request() {
        let request = MetricRequest::counter("test", 1.0)
            .with_label("env", "test")
            .with_help("Test metric");

        let snapshot = MetricSnapshot::from(&request);
        assert_eq!(snapshot.name, request.name());
        assert_eq!(snapshot.metric_type, *request.metric_type());
        assert_eq!(snapshot.labels, *request.labels());
        assert_eq!(snapshot.help, request.help().map(|s| s.to_string()));
    }

    #[test]
    fn test_timer_guard_creation() {
        let labels = HashMap::new();
        let recorded_metrics = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded_metrics_clone = recorded_metrics.clone();

        let recorder = move |request: MetricRequest| {
            recorded_metrics_clone.lock().unwrap().push(request);
        };

        {
            let _timer = TimerGuard::new("test_timer".to_string(), labels, recorder);
            std::thread::sleep(Duration::from_millis(1));
            // Timer drops here and should record
        }

        let metrics = recorded_metrics.lock().unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name(), "test_timer");
        assert_eq!(metrics[0].metric_type(), &MetricType::Timer);
        assert!(metrics[0].value() > 0.0);
    }
}
