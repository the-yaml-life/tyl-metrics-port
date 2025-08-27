# CLAUDE.md - tyl-metrics-port

## 📋 **Module Context**

**tyl-metrics-port** is the metrics collection port for the TYL framework following hexagonal architecture patterns. It provides a pure interface contract that all metrics adapters must implement, enabling pluggable metrics collection backends.

## 🏗️ **Architecture**

### **Port (Interface)**
```rust
#[async_trait]
trait MetricsManager: Send + Sync {
    type Config: Send + Sync;
    
    async fn new(config: Self::Config) -> Result<Self> where Self: Sized;
    async fn record(&self, request: &MetricRequest) -> Result<()>;
    fn start_timer(&self, name: &str, labels: Labels) -> TimerGuard;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn get_snapshot(&self) -> Result<Vec<MetricSnapshot>>;
}
```

### **Adapters (Implementations)**
- `MockMetricsAdapter` - In-memory mock adapter for testing and examples
- `tyl-prometheus-metrics-adapter` - Prometheus integration (separate repository)
- `tyl-otel-metrics-adapter` - OpenTelemetry integration (planned)

### **Core Types**
- `MetricRequest` - Core metric recording request with builder pattern
- `MetricType` - Counter, Gauge, Histogram, Timer
- `MetricValue` - Single values or histogram distributions
- `Labels` - Key-value pairs for metric labeling
- `TimerGuard` - RAII timer for automatic duration recording
- `HealthStatus` - Health check information
- `MetricSnapshot` - Point-in-time metric data for inspection

## 🧪 **Testing**

```bash
# Run all tests (70 comprehensive tests)
cargo test -p tyl-metrics-port

# Run tests with all features
cargo test -p tyl-metrics-port --all-features

# Run doc tests
cargo test --doc -p tyl-metrics-port
```

## 📂 **File Structure**

```
tyl-metrics-port/
├── src/
│   ├── lib.rs           # Main module with re-exports
│   ├── port.rs          # MetricsManager trait definition
│   ├── types.rs         # Core domain types
│   ├── errors.rs        # TYL error integration helpers
│   ├── utils.rs         # Validation utilities
│   └── mock.rs          # MockMetricsAdapter implementation
├── README.md            # Public documentation
├── CLAUDE.md            # This file
└── Cargo.toml           # Dependencies and metadata
```

## 🔧 **How to Use**

### **Basic Usage with Mock Adapter**
```rust
use tyl_metrics_port::{MetricsManager, MockMetricsAdapter, MockMetricsConfig, MetricRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MockMetricsConfig::new("my-service");
    let metrics = MockMetricsAdapter::new(config);
    
    // Record different types of metrics
    let request = MetricRequest::counter("http_requests_total", 1.0)
        .with_label("method", "GET")
        .with_label("status", "200");
    metrics.record(&request).await?;
    
    let request = MetricRequest::gauge("memory_usage_mb", 512.0);
    metrics.record(&request).await?;
    
    let request = MetricRequest::histogram("request_duration_seconds", 0.123);
    metrics.record(&request).await?;
    
    // RAII timer usage
    {
        let _timer = metrics.start_timer("database_query", Labels::new());
        // ... perform database operation ...
        // Duration automatically recorded when timer drops
    }
    
    Ok(())
}
```

### **Generic Usage with Dependency Injection**
```rust
use tyl_metrics_port::{MetricsManager, MetricRequest};

async fn record_business_metrics<M: MetricsManager>(metrics: &M) -> tyl_metrics_port::Result<()> {
    // This works with ANY adapter that implements MetricsManager
    let request = MetricRequest::counter("business_events", 1.0)
        .with_label("event_type", "user_signup")
        .with_label("source", "web");
    
    metrics.record(&request).await?;
    Ok(())
}
```

### **Custom Adapter Implementation**
```rust
use tyl_metrics_port::{MetricsManager, MetricRequest, HealthStatus, Result, async_trait};

pub struct MyCustomAdapter {
    config: MyConfig,
}

#[async_trait]
impl MetricsManager for MyCustomAdapter {
    type Config = MyConfig;
    
    async fn new(config: Self::Config) -> Result<Self> {
        Ok(Self { config })
    }
    
    async fn record(&self, request: &MetricRequest) -> Result<()> {
        // Custom implementation - could send to external service,
        // write to database, etc.
        println!("Recording metric: {} = {}", request.name(), request.value());
        Ok(())
    }
    
    fn start_timer(&self, name: &str, labels: Labels) -> TimerGuard {
        // Implementation using the callback pattern
        TimerGuard::new(name.to_string(), labels, |request| {
            // Handle timer recording
        })
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::healthy())
    }
}
```

## 🛠️ **Useful Commands**

```bash
# Development
cargo clippy -p tyl-metrics-port -- -D warnings
cargo fmt -p tyl-metrics-port --check
cargo doc --no-deps -p tyl-metrics-port --open
cargo test -p tyl-metrics-port --verbose

# Check compilation
cargo check -p tyl-metrics-port
```

## 📦 **Dependencies**

### **TYL Framework Integration**
- `tyl-errors` - Unified error handling (`TylError`, `TylResult`)
- `tyl-config` - Configuration management patterns
- `tyl-logging` - Structured logging integration

### **Core Runtime**
- `serde` + `serde_json` - Serialization support
- `async-trait` - Async trait support for `MetricsManager`
- `tokio` - Async runtime with sync primitives
- `uuid` - Unique identifier generation

### **Validation and Utilities**
- `regex` - Metric name validation
- `lazy_static` - Static regex compilation
- `fastrand` - Random number generation for mock failures

### **Development**
- `tokio-test` - Async testing utilities

## 🎯 **Design Principles**

1. **Pure Hexagonal Port** - Contains only interfaces and domain types, no implementation details
2. **TYL Framework Integration** - Uses `TylError`, follows established patterns
3. **Async-First** - All operations are async for non-blocking metrics collection
4. **Generic Configuration** - Each adapter defines its own config type
5. **Dependency Injection** - Adapters are injected via constructor patterns
6. **RAII Patterns** - `TimerGuard` for automatic duration measurement
7. **Comprehensive Validation** - Input validation using TYL error patterns
8. **Mock Included** - `MockMetricsAdapter` for testing and examples

## 🚀 **Features**

### **Metric Types Supported**
- **Counters** - Monotonically increasing values (requests, events, errors)
- **Gauges** - Current values that can go up/down (memory, CPU, connections)
- **Histograms** - Statistical distributions (request times, payload sizes)
- **Timers** - Duration measurements (query times, processing duration)

### **Integration Features**
- **Builder Pattern** - Fluent API for constructing metric requests
- **Label Management** - Key-value labeling with validation
- **Error Handling** - TYL framework error integration with context
- **Health Checking** - Adapter health monitoring
- **Thread Safety** - All operations are thread-safe
- **Concurrent Access** - Supports high-throughput concurrent recording

### **Validation Features**
- **Metric Name Validation** - Ensures names follow standard conventions
- **Label Validation** - Validates keys and values with reasonable limits
- **Value Validation** - Ensures metric values are finite numbers
- **Histogram Validation** - Validates bucket configurations

## ⚠️ **Known Limitations**

- **Mock Storage** - `MockMetricsAdapter` stores metrics in memory (not persistent)
- **Histogram Buckets** - Some adapters may have global bucket configuration
- **Timer Precision** - Timer precision depends on system clock resolution
- **Snapshot Support** - Push-based adapters may not support meaningful snapshots

## 📝 **Notes for Contributors**

### **Development Workflow**
1. **Pure Port Only** - This module contains no adapter implementations
2. **TDD Approach** - All new features should be test-driven
3. **TYL Integration** - Always use TYL framework patterns
4. **Validation First** - Input validation is critical for metrics quality
5. **Async Patterns** - All public APIs should be async-compatible

### **Adding New Features**
1. Add domain types to `types.rs`
2. Add validation to `utils.rs`
3. Update `MockMetricsAdapter` to support new features
4. Add comprehensive tests
5. Update documentation

### **Testing Requirements**
- **Unit Tests** - For all core functionality
- **Integration Tests** - Between different modules
- **Mock Tests** - Comprehensive `MockMetricsAdapter` testing
- **Validation Tests** - For all validation functions
- **Performance Tests** - For concurrent access patterns

## 🔗 **Related TYL Modules**

- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - Error handling framework
- [`tyl-config`](https://github.com/the-yaml-life/tyl-config) - Configuration management
- [`tyl-logging`](https://github.com/the-yaml-life/tyl-logging) - Structured logging
- [`tyl-tracing`](https://github.com/the-yaml-life/tyl-tracing) - Distributed tracing

## 📈 **Metrics Collection Patterns**

### **Application Metrics**
```rust
// HTTP request metrics
let request = MetricRequest::counter("http_requests_total", 1.0)
    .with_label("method", "POST")
    .with_label("endpoint", "/api/users")
    .with_label("status_code", "201");

// Response time histogram
let request = MetricRequest::histogram("http_request_duration_seconds", duration_secs)
    .with_label("endpoint", "/api/users");
```

### **Business Metrics**
```rust
// User activity tracking
let request = MetricRequest::counter("user_actions_total", 1.0)
    .with_label("action_type", "signup")
    .with_label("source", "mobile_app");

// Resource utilization
let request = MetricRequest::gauge("database_connections_active", active_count as f64);
```

### **System Metrics**
```rust
// Memory usage
let request = MetricRequest::gauge("memory_usage_bytes", memory_bytes as f64)
    .with_label("component", "cache");

// Processing times
{
    let _timer = metrics.start_timer("batch_processing_duration", labels);
    // ... process batch ...
    // Duration automatically recorded
}
```

The TYL Metrics Port provides a solid foundation for building pluggable, type-safe metrics collection systems that integrate seamlessly with the TYL framework ecosystem.