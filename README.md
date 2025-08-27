# TYL Metrics Port

**Hexagonal Architecture Port** for metrics collection in the TYL framework.

This module defines the **pure interface** for metrics collection without any implementation details. Following the hexagonal architecture pattern, it provides the contract that all metrics adapters must implement.

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
tyl-metrics-port = { git = "https://github.com/the-yaml-life/tyl-metrics-port", branch = "main" }
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

### Basic Usage

```rust
use tyl_metrics_port::{MetricsManager, MockMetricsAdapter, MockMetricsConfig, MetricRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the mock adapter for testing/examples
    let config = MockMetricsConfig::new("my-service");
    let metrics = MockMetricsAdapter::new(config);
    
    // Record different types of metrics
    let request = MetricRequest::counter("http_requests_total", 1.0)
        .with_label("method", "GET")
        .with_label("status", "200");
    metrics.record(&request).await?;
    
    let request = MetricRequest::gauge("memory_usage_mb", 512.0);
    metrics.record(&request).await?;
    
    // RAII timer usage
    {
        let _timer = metrics.start_timer("database_query", std::collections::HashMap::new());
        // ... perform database operation ...
        // Duration automatically recorded when timer drops
    }
    
    // Check health
    let health = metrics.health_check().await?;
    println!("Metrics health: {}", health);
    
    Ok(())
}
```

## 🏗️ Architecture

This is a **PORT** - it defines **WHAT** can be done, not **HOW**. Concrete implementations (adapters) are in separate modules:

- `tyl-prometheus-metrics-adapter` - Prometheus implementation
- `tyl-otel-metrics-adapter` - OpenTelemetry implementation (planned)
- Custom adapters by implementing `MetricsManager`

### Core Trait

```rust
#[async_trait]
pub trait MetricsManager: Send + Sync {
    type Config: Send + Sync;
    
    async fn new(config: Self::Config) -> Result<Self> where Self: Sized;
    async fn record(&self, request: &MetricRequest) -> Result<()>;
    fn start_timer(&self, name: &str, labels: Labels) -> TimerGuard;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn get_snapshot(&self) -> Result<Vec<MetricSnapshot>>;
}
```

## 📊 Supported Metrics

- **Counters** - Monotonically increasing values (requests, errors)
- **Gauges** - Current values that can go up/down (memory, CPU)
- **Histograms** - Statistical distributions (request times, sizes)
- **Timers** - Duration measurements with RAII pattern

## 🔌 Dependency Injection

Use generic programming for adapter-agnostic code:

```rust
use tyl_metrics_port::{MetricsManager, MetricRequest};

async fn record_business_metrics<M: MetricsManager>(metrics: &M) -> tyl_metrics_port::Result<()> {
    let request = MetricRequest::counter("business_events", 1.0)
        .with_label("event_type", "user_signup")
        .with_label("source", "web");
    
    metrics.record(&request).await?;
    Ok(())
}

// Works with ANY adapter that implements MetricsManager
let prometheus_metrics = PrometheusMetricsAdapter::new(prometheus_config).await?;
record_business_metrics(&prometheus_metrics).await?;

let otel_metrics = OtelMetricsAdapter::new(otel_config).await?;
record_business_metrics(&otel_metrics).await?;
```

## 🧪 Mock for Testing

The port includes `MockMetricsAdapter` for testing and examples:

```rust
use tyl_metrics_port::{MockMetricsAdapter, MockMetricsConfig};

#[tokio::test]
async fn test_metrics_collection() {
    let config = MockMetricsConfig::new("test-service")
        .with_storage(true)  // Enable metric storage for inspection
        .with_max_stored(1000);
    
    let metrics = MockMetricsAdapter::new(config);
    
    let request = MetricRequest::counter("test_counter", 1.0);
    metrics.record(&request).await.unwrap();
    
    // Inspect recorded metrics
    let stored = metrics.get_stored_metrics().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].name, "test_counter");
}
```

## 🎯 Features

- **Builder Pattern** - Fluent API for constructing metrics
- **Validation** - Comprehensive input validation
- **Type Safety** - Strong typing throughout
- **Thread Safe** - Concurrent access support
- **TYL Integration** - Uses TylError and framework patterns
- **Health Checking** - Built-in health monitoring
- **Label Management** - Key-value metric labeling

## 📚 Examples

### HTTP Server Metrics
```rust
// Request count
let request = MetricRequest::counter("http_requests_total", 1.0)
    .with_label("method", "POST")
    .with_label("endpoint", "/api/users")
    .with_label("status_code", "201");

// Response time
let request = MetricRequest::histogram("http_request_duration_seconds", 0.123)
    .with_label("endpoint", "/api/users");
```

### Database Metrics
```rust
// Active connections
let request = MetricRequest::gauge("database_connections_active", 42.0);

// Query timing with RAII
{
    let _timer = metrics.start_timer("database_query_duration", labels);
    // Execute database query
    // Duration automatically recorded when _timer drops
}
```

### Business Metrics
```rust
// User actions
let request = MetricRequest::counter("user_actions_total", 1.0)
    .with_label("action_type", "signup")
    .with_label("source", "mobile_app");

// System resources
let request = MetricRequest::gauge("memory_usage_bytes", memory_bytes as f64)
    .with_label("component", "cache");
```

## 🛠️ Development

```bash
# Run tests
cargo test -p tyl-metrics-port

# Check compilation
cargo check -p tyl-metrics-port

# Generate docs
cargo doc --no-deps -p tyl-metrics-port --open
```

## 📄 License

Licensed under AGPL-3.0. See [LICENSE](LICENSE) for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes
4. Ensure all tests pass (`cargo test`)
5. Commit your changes (`git commit -am 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📖 Documentation

- [CLAUDE.md](CLAUDE.md) - Development context and examples
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [Crate Documentation](https://docs.rs/tyl-metrics-port) - API reference

## 🔗 Related Projects

- [tyl-prometheus-metrics-adapter](https://github.com/the-yaml-life/tyl-prometheus-metrics-adapter) - Prometheus adapter
- [TYL Framework](https://github.com/the-yaml-life) - Complete framework ecosystem