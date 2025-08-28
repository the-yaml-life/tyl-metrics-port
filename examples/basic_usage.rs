//! Basic usage example for TYL Metrics Port
//!
//! This example demonstrates how to use the MockMetricsAdapter for testing,
//! development, and integration testing within the TYL framework.
//!
//! The MockMetricsAdapter is the internal mock implementation that:
//! - Stores metrics in memory for inspection
//! - Provides full MetricsManager functionality
//! - Supports all metric types (counters, gauges, histograms, timers)
//! - Enables testing of metrics collection without external dependencies

use std::time::Duration;
use tokio::time::sleep;
use tyl_metrics_port::{
    Labels, MetricRequest, MetricType, MetricsManager, MockMetricsAdapter, MockMetricsConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TYL Metrics Port - Basic Usage Example");
    println!("==========================================");

    // Create mock configuration for testing/development
    let config = MockMetricsConfig::new("basic-usage-example")
        .with_storage(true) // Enable metric storage for inspection
        .with_max_stored(1000); // Store up to 1000 metrics

    // Create the MockMetricsAdapter (our internal mock for testing)
    let metrics = MockMetricsAdapter::new(config);

    println!("✅ Created MockMetricsAdapter for testing and development");

    // Example 1: Counter metrics
    println!("\n📊 Recording counter metrics...");
    let request = MetricRequest::counter("http_requests_total", 1.0)
        .with_label("method", "GET")
        .with_label("status", "200")
        .with_label("endpoint", "/api/users");

    metrics.record(&request).await?;

    // Multiple counter increments
    for i in 1..=5 {
        let request = MetricRequest::counter("requests_processed", 1.0)
            .with_label("batch", &format!("batch_{}", i));
        metrics.record(&request).await?;
    }

    // Example 2: Gauge metrics (current values)
    println!("📏 Recording gauge metrics...");
    let request = MetricRequest::gauge("memory_usage_mb", 512.0).with_label("component", "cache");
    metrics.record(&request).await?;

    let request =
        MetricRequest::gauge("active_connections", 42.0).with_label("service", "database");
    metrics.record(&request).await?;

    // Example 3: Histogram metrics (statistical distributions)
    println!("📈 Recording histogram metrics...");
    for duration_ms in [12, 45, 23, 67, 34, 89, 15] {
        let request =
            MetricRequest::histogram("request_duration_seconds", duration_ms as f64 / 1000.0)
                .with_label("endpoint", "/api/data");
        metrics.record(&request).await?;
    }

    // Example 4: Timer metrics with RAII pattern
    println!("⏱️  Recording timer metrics...");
    {
        let mut labels = Labels::new();
        labels.insert("operation".to_string(), "database_query".to_string());
        labels.insert("table".to_string(), "users".to_string());

        let _timer = metrics.start_timer("query_duration", labels);

        // Simulate database operation
        sleep(Duration::from_millis(50)).await;

        // Timer automatically records duration when dropped
    }

    // Example 5: Timer with explicit duration
    let request = MetricRequest::timer("background_job", Duration::from_millis(150))
        .with_label("job_type", "email_processing");
    metrics.record(&request).await?;

    // Example 6: Health check
    println!("🔍 Checking metrics adapter health...");
    let health = metrics.health_check().await?;
    println!(
        "   Health Status: {} - {}",
        if health.is_healthy {
            "HEALTHY"
        } else {
            "UNHEALTHY"
        },
        health.message
    );

    // Example 7: Inspect stored metrics (MockAdapter feature)
    println!("\n📋 Inspecting stored metrics...");
    let stored_metrics = metrics.get_stored_metrics().await;
    println!("   Total metrics stored: {}", stored_metrics.len());

    // Group metrics by type
    let mut counter_count = 0;
    let mut gauge_count = 0;
    let mut histogram_count = 0;
    let mut timer_count = 0;

    for metric in &stored_metrics {
        match metric.metric_type {
            MetricType::Counter => counter_count += 1,
            MetricType::Gauge => gauge_count += 1,
            MetricType::Histogram => histogram_count += 1,
            MetricType::Timer => timer_count += 1,
        }
    }

    println!("   📊 Counters: {}", counter_count);
    println!("   📏 Gauges: {}", gauge_count);
    println!("   📈 Histograms: {}", histogram_count);
    println!("   ⏱️  Timers: {}", timer_count);

    // Example 8: Search metrics by name
    println!("\n🔍 Searching metrics by name...");
    let http_metrics = metrics.find_metrics_by_name("http_requests_total").await;
    for metric in http_metrics {
        let value = match &metric.value {
            tyl_metrics_port::MetricValue::Single(val) => *val,
            tyl_metrics_port::MetricValue::Histogram { sum, .. } => *sum,
        };
        println!(
            "   Found: {} = {} (labels: {})",
            metric.name,
            value,
            format_labels_simple(&metric.labels)
        );
    }

    // Example 9: Search metrics by type
    println!("🔍 Searching gauge metrics...");
    let gauge_metrics = metrics.find_metrics_by_type(MetricType::Gauge).await;
    for metric in gauge_metrics {
        let value = match &metric.value {
            tyl_metrics_port::MetricValue::Single(val) => *val,
            tyl_metrics_port::MetricValue::Histogram { sum, .. } => *sum,
        };
        println!("   Gauge: {} = {}", metric.name, value);
    }

    // Example 10: Get metrics snapshot
    println!("\n📸 Getting metrics snapshot...");
    let snapshot = metrics.get_snapshot().await?;
    println!("   Snapshot contains {} metrics", snapshot.len());

    // Display some sample metrics
    if !snapshot.is_empty() {
        println!("   Sample metrics:");
        for metric in snapshot.iter().take(3) {
            let value_str = match &metric.value {
                tyl_metrics_port::MetricValue::Single(val) => format!("{:.3}", val),
                tyl_metrics_port::MetricValue::Histogram { sum, count, .. } => {
                    format!("histogram(sum={:.3}, count={})", sum, count)
                }
            };
            println!(
                "     {} = {} {:?}",
                metric.name, value_str, metric.metric_type
            );
        }
    }

    println!("\n✅ MockMetricsAdapter demonstration completed!");
    println!("💡 This mock adapter is perfect for:");
    println!("   • Unit testing metrics collection");
    println!("   • Integration testing without external dependencies");
    println!("   • Development and debugging");
    println!("   • CI/CD pipeline testing");
    println!("   • Validating metrics before production deployment");

    Ok(())
}

// Helper function to format labels simply
fn format_labels_simple(labels: &std::collections::HashMap<String, String>) -> String {
    if labels.is_empty() {
        return "{}".to_string();
    }

    let pairs: Vec<String> = labels.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    format!("{{{}}}", pairs.join(", "))
}
