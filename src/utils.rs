//! Validation utilities for metrics port
//!
//! This module provides validation functions for metric names, labels, values,
//! and other aspects of metrics collection. It ensures data quality and
//! consistency across all adapters.

use super::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

// Maximum allowed lengths for various metric components
const MAX_METRIC_NAME_LENGTH: usize = 255;
const MAX_LABEL_KEY_LENGTH: usize = 128;
const MAX_LABEL_VALUE_LENGTH: usize = 1024;
const MAX_LABELS_COUNT: usize = 32;

/// Validate a metric name
///
/// Ensures metric names follow standard conventions:
/// - Must not be empty
/// - Must start with a letter or underscore
/// - Can contain letters, numbers, underscores, and colons
/// - Must be within reasonable length limits
///
/// # Examples
/// ```rust
/// use tyl_metrics_port::validate_metric_name;
///
/// assert!(validate_metric_name("http_requests_total").is_ok());
/// assert!(validate_metric_name("").is_err());
/// ```
pub fn validate_metric_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(metrics_error("metric_name", "Metric name cannot be empty"));
    }

    if name.len() > MAX_METRIC_NAME_LENGTH {
        return Err(metrics_error(
            "metric_name",
            format!("Metric name too long (max {MAX_METRIC_NAME_LENGTH} chars)"),
        ));
    }

    lazy_static! {
        static ref METRIC_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z_:][a-zA-Z0-9_:]*$").unwrap();
    }

    if !METRIC_NAME_REGEX.is_match(name) {
        return Err(metrics_error(
            "metric_name",
            "Invalid metric name format (must match [a-zA-Z_:][a-zA-Z0-9_:]*)",
        ));
    }

    Ok(())
}

/// Validate a label key
///
/// Ensures label keys follow standard conventions:
/// - Must not be empty
/// - Must start with a letter or underscore
/// - Cannot start with double underscore (reserved)
/// - Can contain letters, numbers, and underscores
/// - Must be within reasonable length limits
pub fn validate_label_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(metrics_error("label_key", "Label key cannot be empty"));
    }

    if key.len() > MAX_LABEL_KEY_LENGTH {
        return Err(metrics_error(
            "label_key",
            format!("Label key too long (max {MAX_LABEL_KEY_LENGTH} chars)"),
        ));
    }

    if key.starts_with("__") {
        return Err(metrics_error(
            "label_key",
            "Label keys starting with '__' are reserved",
        ));
    }

    lazy_static! {
        static ref LABEL_KEY_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }

    if !LABEL_KEY_REGEX.is_match(key) {
        return Err(metrics_error(
            "label_key",
            "Invalid label key format (must match [a-zA-Z_][a-zA-Z0-9_]*)",
        ));
    }

    Ok(())
}

/// Validate a label value
///
/// Ensures label values meet basic quality requirements:
/// - Can be empty
/// - Must not contain null bytes
/// - Must be within reasonable length limits
pub fn validate_label_value(value: &str) -> Result<()> {
    if value.len() > MAX_LABEL_VALUE_LENGTH {
        return Err(metrics_error(
            "label_value",
            format!("Label value too long (max {MAX_LABEL_VALUE_LENGTH} chars)"),
        ));
    }

    if value.contains('\0') {
        return Err(metrics_error(
            "label_value",
            "Label values cannot contain null bytes",
        ));
    }

    Ok(())
}

/// Validate a complete set of labels
///
/// Ensures the entire label set meets requirements:
/// - Total number of labels within limits
/// - All keys and values individually valid
pub fn validate_labels(labels: &HashMap<String, String>) -> Result<()> {
    if labels.len() > MAX_LABELS_COUNT {
        return Err(metrics_error(
            "labels",
            format!("Too many labels (max {MAX_LABELS_COUNT})"),
        ));
    }

    for (key, value) in labels {
        validate_label_key(key)?;
        validate_label_value(value)?;
    }

    Ok(())
}

/// Validate a metric value
///
/// Ensures metric values are valid numbers:
/// - Must be finite (no NaN, Infinity)
/// - Can be positive, negative, or zero
pub fn validate_metric_value(value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(metrics_error(
            "metric_value",
            "Metric values must be finite (no NaN or Infinity)",
        ));
    }

    Ok(())
}

/// Validate a counter value
///
/// Counter values have additional restrictions:
/// - Must be non-negative
/// - Must be finite
pub fn validate_counter_value(value: f64) -> Result<()> {
    validate_metric_value(value)?;

    if value < 0.0 {
        return Err(metrics_error(
            "counter_value",
            "Counter values must be non-negative",
        ));
    }

    Ok(())
}

/// Format labels as a string for logging/debugging
///
/// Creates a consistent string representation of labels for debugging output.
/// The format is: "key1=value1,key2=value2"
///
/// # Examples
/// ```rust
/// use std::collections::HashMap;
/// use tyl_metrics_port::format_labels;
///
/// let mut labels = HashMap::new();
/// labels.insert("method".to_string(), "GET".to_string());
/// let formatted = format_labels(&labels);
/// // Result: "method=GET" or similar
/// ```
pub fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return "{}".to_string();
    }

    let mut pairs: Vec<(&String, &String)> = labels.iter().collect();
    pairs.sort_by_key(|(k, _)| *k); // Sort by key for consistent output

    pairs
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Normalize a metric name for consistent storage and comparison
///
/// Performs basic normalization:
/// - Converts to lowercase
/// - Trims whitespace
/// - Collapses multiple underscores to single underscore
///
/// # Arguments
/// * `name` - The metric name to normalize
///
/// # Returns
/// * `String` - The normalized metric name
///
/// # Examples
/// ```rust
/// use tyl_metrics_port::normalize_metric_name;
///
/// assert_eq!(normalize_metric_name("  HTTP__Requests__Total  "), "http_requests_total");
/// ```
pub fn normalize_metric_name(name: &str) -> String {
    lazy_static::lazy_static! {
        static ref UNDERSCORE_REGEX: Regex = Regex::new(r"_+").unwrap();
    }

    let normalized = name.trim().to_lowercase();
    UNDERSCORE_REGEX.replace_all(&normalized, "_").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_metric_name_valid() {
        assert!(validate_metric_name("http_requests_total").is_ok());
        assert!(validate_metric_name("cpu_usage").is_ok());
        assert!(validate_metric_name("db:connection_pool_size").is_ok());
        assert!(validate_metric_name("_private_metric").is_ok());
    }

    #[test]
    fn test_validate_metric_name_invalid() {
        assert!(validate_metric_name("").is_err());
        assert!(validate_metric_name("123_invalid").is_err());
        assert!(validate_metric_name("invalid name").is_err());
        assert!(validate_metric_name("invalid-name").is_err());
        assert!(validate_metric_name(&"x".repeat(256)).is_err());
    }

    #[test]
    fn test_validate_label_key_valid() {
        assert!(validate_label_key("method").is_ok());
        assert!(validate_label_key("status_code").is_ok());
        assert!(validate_label_key("_internal").is_ok());
    }

    #[test]
    fn test_validate_label_key_invalid() {
        assert!(validate_label_key("").is_err());
        assert!(validate_label_key("__reserved").is_err());
        assert!(validate_label_key("123invalid").is_err());
        assert!(validate_label_key("invalid-key").is_err());
        assert!(validate_label_key(&"x".repeat(129)).is_err());
    }

    #[test]
    fn test_validate_label_value_valid() {
        assert!(validate_label_value("GET").is_ok());
        assert!(validate_label_value("200").is_ok());
        assert!(validate_label_value("").is_ok());
        assert!(validate_label_value("some-value").is_ok());
    }

    #[test]
    fn test_validate_label_value_invalid() {
        assert!(validate_label_value("value\0with\0nulls").is_err());
        assert!(validate_label_value(&"x".repeat(1025)).is_err());
    }

    #[test]
    fn test_validate_labels() {
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());
        assert!(validate_labels(&labels).is_ok());

        // Too many labels
        let mut too_many_labels = HashMap::new();
        for i in 0..33 {
            too_many_labels.insert(format!("label_{}", i), "value".to_string());
        }
        assert!(validate_labels(&too_many_labels).is_err());
    }

    #[test]
    fn test_validate_metric_value() {
        assert!(validate_metric_value(123.45).is_ok());
        assert!(validate_metric_value(0.0).is_ok());
        assert!(validate_metric_value(-123.45).is_ok());

        assert!(validate_metric_value(f64::NAN).is_err());
        assert!(validate_metric_value(f64::INFINITY).is_err());
        assert!(validate_metric_value(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_validate_counter_value() {
        assert!(validate_counter_value(123.45).is_ok());
        assert!(validate_counter_value(0.0).is_ok());

        assert!(validate_counter_value(-123.45).is_err());
        assert!(validate_counter_value(f64::NAN).is_err());
    }

    #[test]
    fn test_format_labels() {
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());

        let formatted = format_labels(&labels);
        // Order may vary due to HashMap, but should contain both labels
        assert!(formatted.contains("method=GET"));
        assert!(formatted.contains("status=200"));
        assert!(formatted.contains(","));

        // Empty labels
        let empty_labels = HashMap::new();
        assert_eq!(format_labels(&empty_labels), "{}");
    }

    #[test]
    fn test_normalize_metric_name() {
        assert_eq!(
            normalize_metric_name("HTTP_Requests_Total"),
            "http_requests_total"
        );
        assert_eq!(normalize_metric_name("  CPU__Usage  "), "cpu_usage");
        assert_eq!(
            normalize_metric_name("already_normalized"),
            "already_normalized"
        );
    }
}
