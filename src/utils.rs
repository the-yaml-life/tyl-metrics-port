//! Validation utilities and helpers for metrics
//!
//! This module provides validation functions and utility helpers that are
//! used across the metrics system for ensuring data quality and consistency.

use crate::{metrics_error, Result};
use regex::Regex;
use std::collections::HashMap;

/// Validates a metric name according to standard conventions
///
/// Metric names should follow these rules:
/// - Must not be empty
/// - Must start with a letter or underscore
/// - Can contain letters, numbers, underscores, and colons
/// - Must not exceed 255 characters
/// - Should not contain spaces or special characters (except _ and :)
///
/// # Arguments
/// * `name` - The metric name to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error with description if invalid
///
/// # Examples
/// ```rust
/// use tyl_metrics_port::validate_metric_name;
///
/// assert!(validate_metric_name("http_requests_total").is_ok());
/// assert!(validate_metric_name("cpu:usage_percent").is_ok());
/// assert!(validate_metric_name("").is_err());
/// assert!(validate_metric_name("invalid name").is_err());
/// ```
pub fn validate_metric_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(metrics_error("name", "Metric name cannot be empty"));
    }

    if name.len() > 255 {
        return Err(metrics_error(
            "name",
            "Metric name cannot exceed 255 characters",
        ));
    }

    // Check if name starts with letter or underscore
    if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
        return Err(metrics_error(
            "name",
            "Metric name must start with a letter or underscore",
        ));
    }

    // Use regex to validate the full name format
    lazy_static::lazy_static! {
        static ref METRIC_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_:]*$").unwrap();
    }

    if !METRIC_NAME_REGEX.is_match(name) {
        return Err(metrics_error(
            "name",
            "Metric name can only contain letters, numbers, underscores, and colons",
        ));
    }

    Ok(())
}

/// Validates a label key according to standard conventions
///
/// Label keys should follow these rules:
/// - Must not be empty
/// - Must start with a letter or underscore
/// - Can contain letters, numbers, and underscores
/// - Must not exceed 128 characters
/// - Cannot start with double underscore (reserved for internal use)
///
/// # Arguments
/// * `key` - The label key to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error with description if invalid
pub fn validate_label_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(metrics_error("label_key", "Label key cannot be empty"));
    }

    if key.len() > 128 {
        return Err(metrics_error(
            "label_key",
            "Label key cannot exceed 128 characters",
        ));
    }

    if key.starts_with("__") {
        return Err(metrics_error(
            "label_key",
            "Label keys starting with '__' are reserved",
        ));
    }

    if !key.chars().next().unwrap().is_alphabetic() && !key.starts_with('_') {
        return Err(metrics_error(
            "label_key",
            "Label key must start with a letter or underscore",
        ));
    }

    lazy_static::lazy_static! {
        static ref LABEL_KEY_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }

    if !LABEL_KEY_REGEX.is_match(key) {
        return Err(metrics_error(
            "label_key",
            "Label key can only contain letters, numbers, and underscores",
        ));
    }

    Ok(())
}

/// Validates a label value
///
/// Label values are more permissive than keys but still have some constraints:
/// - Cannot exceed 1024 characters
/// - Cannot contain null bytes
/// - Should be valid UTF-8 (enforced by Rust strings)
///
/// # Arguments
/// * `value` - The label value to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error with description if invalid
pub fn validate_label_value(value: &str) -> Result<()> {
    if value.len() > 1024 {
        return Err(metrics_error(
            "label_value",
            "Label value cannot exceed 1024 characters",
        ));
    }

    if value.contains('\0') {
        return Err(metrics_error(
            "label_value",
            "Label value cannot contain null bytes",
        ));
    }

    Ok(())
}

/// Validates a set of labels
///
/// This function validates both keys and values in a label set and also
/// checks for reasonable limits on the total number of labels.
///
/// # Arguments
/// * `labels` - The labels to validate
///
/// # Returns
/// * `Result<()>` - Ok if all labels are valid, error describing the first invalid label
pub fn validate_labels(labels: &HashMap<String, String>) -> Result<()> {
    if labels.len() > 32 {
        return Err(metrics_error(
            "labels",
            "Cannot have more than 32 labels per metric",
        ));
    }

    for (key, value) in labels {
        validate_label_key(key)?;
        validate_label_value(value)?;
    }

    Ok(())
}

/// Validates a metric value
///
/// Metric values should be finite numbers (not NaN or infinite).
///
/// # Arguments
/// * `value` - The metric value to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error if invalid
pub fn validate_metric_value(value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(metrics_error(
            "value",
            "Metric value must be a finite number",
        ));
    }

    Ok(())
}

/// Validates a counter value
///
/// Counter values must be non-negative since counters are monotonically increasing.
///
/// # Arguments
/// * `value` - The counter value to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error if invalid
pub fn validate_counter_value(value: f64) -> Result<()> {
    validate_metric_value(value)?;

    if value < 0.0 {
        return Err(metrics_error(
            "value",
            "Counter values must be non-negative",
        ));
    }

    Ok(())
}

/// Format labels for consistent display and logging
///
/// This function creates a consistent string representation of labels
/// that can be used for logging, debugging, or display purposes.
///
/// # Arguments
/// * `labels` - The labels to format
///
/// # Returns
/// * `String` - Formatted label string in key=value,key=value format
///
/// # Examples
/// ```rust
/// use std::collections::HashMap;
/// use tyl_metrics_port::format_labels;
///
/// let mut labels = HashMap::new();
/// labels.insert("method".to_string(), "GET".to_string());
/// labels.insert("status".to_string(), "200".to_string());
///
/// let formatted = format_labels(&labels);
/// // Output: "method=GET,status=200" or "status=200,method=GET" (order may vary)
/// ```
pub fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return "{}".to_string();
    }

    let mut pairs: Vec<_> = labels.iter().collect();
    pairs.sort_by_key(|(k, _)| *k); // Sort by key for consistent output

    pairs
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",")
}

/// Normalize a metric name for consistent storage and comparison
///
/// This function applies standard normalization rules to metric names:
/// - Converts to lowercase (for case-insensitive systems)
/// - Replaces multiple consecutive underscores with single underscores
/// - Trims leading and trailing whitespace
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

/// Parse and validate histogram buckets
///
/// This function validates that histogram buckets are properly ordered
/// and contain valid boundary values.
///
/// # Arguments
/// * `buckets` - Vector of bucket upper bounds
///
/// # Returns
/// * `Result<Vec<f64>>` - Validated and sorted bucket bounds
pub fn validate_histogram_buckets(buckets: &[f64]) -> Result<Vec<f64>> {
    if buckets.is_empty() {
        return Err(metrics_error(
            "buckets",
            "Histogram must have at least one bucket",
        ));
    }

    // Validate all bucket values
    for &bucket in buckets {
        validate_metric_value(bucket)?;
    }

    // Sort buckets and ensure they're unique
    let mut sorted_buckets = buckets.to_vec();
    sorted_buckets.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sorted_buckets.dedup();

    if sorted_buckets.len() != buckets.len() {
        return Err(metrics_error("buckets", "Histogram buckets must be unique"));
    }

    // Ensure the last bucket is +Inf or a reasonable large value
    if sorted_buckets.last() != Some(&f64::INFINITY) {
        sorted_buckets.push(f64::INFINITY);
    }

    Ok(sorted_buckets)
}

/// Create standard histogram buckets for common use cases
///
/// This function provides pre-defined bucket sets for common histogram patterns.
pub struct HistogramBuckets;

impl HistogramBuckets {
    /// Linear buckets: start, width, count
    /// Example: linear(0.0, 0.1, 10) creates [0.0, 0.1, 0.2, ..., 0.9, +Inf]
    pub fn linear(start: f64, width: f64, count: usize) -> Vec<f64> {
        let mut buckets = Vec::with_capacity(count + 1);
        for i in 0..count {
            buckets.push(start + (i as f64) * width);
        }
        buckets.push(f64::INFINITY);
        buckets
    }

    /// Exponential buckets: start, factor, count
    /// Example: exponential(1.0, 2.0, 5) creates [1.0, 2.0, 4.0, 8.0, 16.0, +Inf]
    pub fn exponential(start: f64, factor: f64, count: usize) -> Vec<f64> {
        let mut buckets = Vec::with_capacity(count + 1);
        let mut current = start;
        for _ in 0..count {
            buckets.push(current);
            current *= factor;
        }
        buckets.push(f64::INFINITY);
        buckets
    }

    /// Standard latency buckets suitable for measuring HTTP request durations
    pub fn latency() -> Vec<f64> {
        vec![
            0.001,
            0.002,
            0.005,
            0.01,
            0.025,
            0.05,
            0.1,
            0.25,
            0.5,
            1.0,
            2.5,
            5.0,
            10.0,
            f64::INFINITY,
        ]
    }

    /// Standard size buckets suitable for measuring payload sizes
    pub fn size_bytes() -> Vec<f64> {
        vec![
            64.0,
            256.0,
            1024.0,
            4096.0,
            16384.0,
            65536.0,
            262144.0,
            1048576.0,
            4194304.0,
            16777216.0,
            f64::INFINITY,
        ]
    }
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

    #[test]
    fn test_validate_histogram_buckets() {
        let buckets = vec![0.1, 0.5, 1.0, 2.0];
        let validated = validate_histogram_buckets(&buckets).unwrap();
        assert_eq!(validated, vec![0.1, 0.5, 1.0, 2.0, f64::INFINITY]);

        // Empty buckets
        assert!(validate_histogram_buckets(&[]).is_err());

        // Duplicate buckets
        let duplicates = vec![0.1, 0.5, 0.5, 1.0];
        assert!(validate_histogram_buckets(&duplicates).is_err());

        // Invalid values
        let invalid = vec![0.1, f64::NAN, 1.0];
        assert!(validate_histogram_buckets(&invalid).is_err());
    }

    #[test]
    fn test_histogram_buckets_linear() {
        let buckets = HistogramBuckets::linear(0.0, 0.1, 5);
        assert_eq!(buckets.len(), 6);
        assert_eq!(buckets[0], 0.0);
        assert_eq!(buckets[1], 0.1);
        assert_eq!(buckets[2], 0.2);
        assert!((buckets[3] - 0.3).abs() < 1e-10); // Handle floating point precision
        assert_eq!(buckets[4], 0.4);
        assert_eq!(buckets[5], f64::INFINITY);
    }

    #[test]
    fn test_histogram_buckets_exponential() {
        let buckets = HistogramBuckets::exponential(1.0, 2.0, 3);
        assert_eq!(buckets, vec![1.0, 2.0, 4.0, f64::INFINITY]);
    }

    #[test]
    fn test_histogram_buckets_latency() {
        let buckets = HistogramBuckets::latency();
        assert!(buckets.len() > 0);
        assert_eq!(buckets.last(), Some(&f64::INFINITY));
        assert!(buckets.contains(&0.001));
        assert!(buckets.contains(&1.0));
    }

    #[test]
    fn test_histogram_buckets_size_bytes() {
        let buckets = HistogramBuckets::size_bytes();
        assert!(buckets.len() > 0);
        assert_eq!(buckets.last(), Some(&f64::INFINITY));
        assert!(buckets.contains(&1024.0));
        assert!(buckets.contains(&1048576.0));
    }
}
