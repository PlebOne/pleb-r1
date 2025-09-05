use prometheus::{Counter, Histogram, HistogramOpts, IntGauge, Registry, Encoder, TextEncoder};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    
    // Connection metrics
    pub active_connections: IntGauge,
    pub total_connections: Counter,
    pub connection_duration: Histogram,
    
    // Event metrics
    pub events_received: Counter,
    pub events_stored: Counter,
    pub events_rejected: Counter,
    pub event_processing_time: Histogram,
    
    // Query metrics
    pub queries_received: Counter,
    pub query_processing_time: Histogram,
    pub subscription_count: IntGauge,
    
    // Rate limiting metrics
    pub rate_limited_connections: Counter,
    pub rate_limited_events: Counter,
    
    // Database metrics
    pub database_operations: Counter,
    pub database_errors: Counter,
    pub database_query_time: Histogram,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Connection metrics
        let active_connections = IntGauge::new(
            "relay_active_connections",
            "Number of active WebSocket connections"
        )?;
        registry.register(Box::new(active_connections.clone()))?;
        
        let total_connections = Counter::new(
            "relay_total_connections",
            "Total number of WebSocket connections"
        )?;
        registry.register(Box::new(total_connections.clone()))?;
        
        let connection_duration = Histogram::with_opts(HistogramOpts::new(
            "relay_connection_duration_seconds",
            "Duration of WebSocket connections"
        ))?;
        registry.register(Box::new(connection_duration.clone()))?;
        
        // Event metrics
        let events_received = Counter::new(
            "relay_events_received_total",
            "Total number of events received"
        )?;
        registry.register(Box::new(events_received.clone()))?;
        
        let events_stored = Counter::new(
            "relay_events_stored_total",
            "Total number of events successfully stored"
        )?;
        registry.register(Box::new(events_stored.clone()))?;
        
        let events_rejected = Counter::new(
            "relay_events_rejected_total",
            "Total number of events rejected"
        )?;
        registry.register(Box::new(events_rejected.clone()))?;
        
        let event_processing_time = Histogram::with_opts(HistogramOpts::new(
            "relay_event_processing_seconds",
            "Time to process an event"
        ))?;
        registry.register(Box::new(event_processing_time.clone()))?;
        
        // Query metrics
        let queries_received = Counter::new(
            "relay_queries_received_total",
            "Total number of queries received"
        )?;
        registry.register(Box::new(queries_received.clone()))?;
        
        let query_processing_time = Histogram::with_opts(HistogramOpts::new(
            "relay_query_processing_seconds",
            "Time to process a query"
        ))?;
        registry.register(Box::new(query_processing_time.clone()))?;
        
        let subscription_count = IntGauge::new(
            "relay_active_subscriptions",
            "Number of active subscriptions"
        )?;
        registry.register(Box::new(subscription_count.clone()))?;
        
        // Rate limiting metrics
        let rate_limited_connections = Counter::new(
            "relay_rate_limited_connections_total",
            "Total number of rate limited connections"
        )?;
        registry.register(Box::new(rate_limited_connections.clone()))?;
        
        let rate_limited_events = Counter::new(
            "relay_rate_limited_events_total",
            "Total number of rate limited events"
        )?;
        registry.register(Box::new(rate_limited_events.clone()))?;
        
        // Database metrics
        let database_operations = Counter::new(
            "relay_database_operations_total",
            "Total number of database operations"
        )?;
        registry.register(Box::new(database_operations.clone()))?;
        
        let database_errors = Counter::new(
            "relay_database_errors_total",
            "Total number of database errors"
        )?;
        registry.register(Box::new(database_errors.clone()))?;
        
        let database_query_time = Histogram::with_opts(HistogramOpts::new(
            "relay_database_query_seconds",
            "Time to execute database queries"
        ))?;
        registry.register(Box::new(database_query_time.clone()))?;
        
        Ok(Self {
            registry,
            active_connections,
            total_connections,
            connection_duration,
            events_received,
            events_stored,
            events_rejected,
            event_processing_time,
            queries_received,
            query_processing_time,
            subscription_count,
            rate_limited_connections,
            rate_limited_events,
            database_operations,
            database_errors,
            database_query_time,
        })
    }
    
    pub fn record_connection_start(&self) {
        self.total_connections.inc();
        self.active_connections.inc();
    }
    
    pub fn record_connection_end(&self, duration: f64) {
        self.active_connections.dec();
        self.connection_duration.observe(duration);
    }
    
    pub fn record_event_received(&self) {
        self.events_received.inc();
    }
    
    pub fn record_event_stored(&self, processing_time: f64) {
        self.events_stored.inc();
        self.event_processing_time.observe(processing_time);
    }
    
    pub fn record_event_rejected(&self, processing_time: f64) {
        self.events_rejected.inc();
        self.event_processing_time.observe(processing_time);
    }
    
    pub fn record_query_received(&self) {
        self.queries_received.inc();
    }
    
    pub fn record_query_processed(&self, processing_time: f64) {
        self.query_processing_time.observe(processing_time);
    }
    
    pub fn record_subscription_start(&self) {
        self.subscription_count.inc();
    }
    
    pub fn record_subscription_end(&self) {
        self.subscription_count.dec();
    }
    
    pub fn record_rate_limit_connection(&self) {
        self.rate_limited_connections.inc();
    }
    
    pub fn record_rate_limit_event(&self) {
        self.rate_limited_events.inc();
    }
    
    pub fn record_database_operation(&self, duration: f64) {
        self.database_operations.inc();
        self.database_query_time.observe(duration);
    }
    
    pub fn record_database_error(&self) {
        self.database_errors.inc();
    }
    
    pub fn render(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    
    // Helper method to get metrics as structured data for API endpoints
    pub fn get_api_metrics(&self) -> ApiMetrics {
        ApiMetrics {
            relay_status: RelayStatus {
                active_connections: self.active_connections.get() as u64,
                total_connections: self.total_connections.get() as u64,
                uptime_seconds: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                status: if self.active_connections.get() > 0 { "healthy" } else { "idle" }.to_string(),
            },
            events: EventMetrics {
                events_received: self.events_received.get() as u64,
                events_stored: self.events_stored.get() as u64,
                events_rejected: self.events_rejected.get() as u64,
                avg_processing_time_ms: self.get_avg_processing_time(),
            },
            performance: PerformanceMetrics {
                queries_received: self.queries_received.get() as u64,
                active_subscriptions: self.subscription_count.get() as u64,
                rate_limited_events: self.rate_limited_events.get() as u64,
                database_operations: self.database_operations.get() as u64,
                database_errors: self.database_errors.get() as u64,
                avg_query_time_ms: self.get_avg_query_time(),
            },
        }
    }
    
    fn get_avg_processing_time(&self) -> f64 {
        // Get sample count and sum from histogram
        let sample_count = self.event_processing_time.get_sample_count();
        if sample_count > 0 {
            let sample_sum = self.event_processing_time.get_sample_sum();
            (sample_sum / sample_count as f64) * 1000.0 // Convert to milliseconds
        } else {
            0.0
        }
    }
    
    fn get_avg_query_time(&self) -> f64 {
        let sample_count = self.query_processing_time.get_sample_count();
        if sample_count > 0 {
            let sample_sum = self.query_processing_time.get_sample_sum();
            (sample_sum / sample_count as f64) * 1000.0 // Convert to milliseconds
        } else {
            0.0
        }
    }
}

// API Data Structures
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub relay_status: RelayStatus,
    pub events: EventMetrics,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelayStatus {
    pub active_connections: u64,
    pub total_connections: u64,
    pub uptime_seconds: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMetrics {
    pub events_received: u64,
    pub events_stored: u64,
    pub events_rejected: u64,
    pub avg_processing_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub queries_received: u64,
    pub active_subscriptions: u64,
    pub rate_limited_events: u64,
    pub database_operations: u64,
    pub database_errors: u64,
    pub avg_query_time_ms: f64,
}

// API Handlers
pub async fn get_relay_status(State(state): State<crate::app_state::AppState>) -> Result<Json<RelayStatus>, StatusCode> {
    let metrics = state.metrics.get_api_metrics();
    Ok(Json(metrics.relay_status))
}

pub async fn get_event_metrics(State(state): State<crate::app_state::AppState>) -> Result<Json<EventMetrics>, StatusCode> {
    let metrics = state.metrics.get_api_metrics();
    Ok(Json(metrics.events))
}

pub async fn get_performance_metrics(State(state): State<crate::app_state::AppState>) -> Result<Json<PerformanceMetrics>, StatusCode> {
    let metrics = state.metrics.get_api_metrics();
    Ok(Json(metrics.performance))
}

pub async fn get_all_metrics(State(state): State<crate::app_state::AppState>) -> Result<Json<ApiMetrics>, StatusCode> {
    let metrics = state.metrics.get_api_metrics();
    Ok(Json(metrics))
}

// Router setup for API endpoints
pub fn create_metrics_api_router() -> Router<crate::app_state::AppState> {
    Router::new()
        .route("/api/metrics/relay-status", get(get_relay_status))
        .route("/api/metrics/events", get(get_event_metrics))
        .route("/api/metrics/performance", get(get_performance_metrics))
        .route("/api/metrics/all", get(get_all_metrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Verify all metrics are initialized
        assert_eq!(metrics.active_connections.get(), 0); // IntGauge returns i64
        assert_eq!(metrics.total_connections.get(), 0.0); // Counter returns f64
        assert_eq!(metrics.events_received.get(), 0.0);
        assert_eq!(metrics.events_stored.get(), 0.0);
        assert_eq!(metrics.events_rejected.get(), 0.0);
        assert_eq!(metrics.queries_received.get(), 0.0);
        assert_eq!(metrics.subscription_count.get(), 0); // IntGauge returns i64
        assert_eq!(metrics.rate_limited_connections.get(), 0.0);
        assert_eq!(metrics.rate_limited_events.get(), 0.0);
        assert_eq!(metrics.database_operations.get(), 0.0);
        assert_eq!(metrics.database_errors.get(), 0.0);
    }

    #[test]
    fn test_connection_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Test connection start
        metrics.record_connection_start();
        assert_eq!(metrics.total_connections.get(), 1.0);
        assert_eq!(metrics.active_connections.get(), 1);

        metrics.record_connection_start();
        assert_eq!(metrics.total_connections.get(), 2.0);
        assert_eq!(metrics.active_connections.get(), 2);

        // Test connection end
        metrics.record_connection_end(1.5);
        assert_eq!(metrics.active_connections.get(), 1);
        assert_eq!(metrics.total_connections.get(), 2.0); // Total should not decrease

        metrics.record_connection_end(0.5);
        assert_eq!(metrics.active_connections.get(), 0);
    }

    #[test]
    fn test_event_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Test event received
        metrics.record_event_received();
        metrics.record_event_received();
        assert_eq!(metrics.events_received.get(), 2.0);

        // Test event stored
        metrics.record_event_stored(0.1);
        assert_eq!(metrics.events_stored.get(), 1.0);

        // Test event rejected
        metrics.record_event_rejected(0.05);
        assert_eq!(metrics.events_rejected.get(), 1.0);
    }

    #[test]
    fn test_query_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        metrics.record_query_received();
        metrics.record_query_received();
        metrics.record_query_received();
        assert_eq!(metrics.queries_received.get(), 3.0);

        metrics.record_query_processed(0.2);
        // We can't easily test histogram values, but we can verify the method doesn't panic
    }

    #[test]
    fn test_subscription_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Test subscription start
        metrics.record_subscription_start();
        metrics.record_subscription_start();
        assert_eq!(metrics.subscription_count.get(), 2);

        // Test subscription end
        metrics.record_subscription_end();
        assert_eq!(metrics.subscription_count.get(), 1);

        metrics.record_subscription_end();
        assert_eq!(metrics.subscription_count.get(), 0);
    }

    #[test]
    fn test_rate_limit_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        metrics.record_rate_limit_connection();
        metrics.record_rate_limit_connection();
        assert_eq!(metrics.rate_limited_connections.get(), 2.0);

        metrics.record_rate_limit_event();
        assert_eq!(metrics.rate_limited_events.get(), 1.0);
    }

    #[test]
    fn test_database_metrics() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        metrics.record_database_operation(0.05);
        metrics.record_database_operation(0.1);
        assert_eq!(metrics.database_operations.get(), 2.0);

        metrics.record_database_error();
        metrics.record_database_error();
        assert_eq!(metrics.database_errors.get(), 2.0);
    }

    #[test]
    fn test_metrics_render() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Add some data
        metrics.record_connection_start();
        metrics.record_event_received();
        metrics.record_query_received();
        
        let rendered = metrics.render().expect("Failed to render metrics");
        
        // Verify the output contains expected metric names
        assert!(rendered.contains("relay_active_connections"));
        assert!(rendered.contains("relay_total_connections"));
        assert!(rendered.contains("relay_events_received_total"));
        assert!(rendered.contains("relay_queries_received_total"));
        
        // Verify it's in Prometheus format
        assert!(rendered.contains("# HELP"));
        assert!(rendered.contains("# TYPE"));
    }

    #[test]
    fn test_metrics_histogram_observations() {
        let metrics = Metrics::new().expect("Failed to create metrics");
        
        // Test that histogram observations don't panic with various values
        metrics.record_connection_end(0.0);
        metrics.record_connection_end(1.0);
        metrics.record_connection_end(60.0);
        
        metrics.record_event_stored(0.001);
        metrics.record_event_stored(0.1);
        metrics.record_event_stored(1.0);
        
        metrics.record_event_rejected(0.002);
        metrics.record_query_processed(0.5);
        metrics.record_database_operation(0.01);
        
        // Should not panic and render should still work
        let rendered = metrics.render().expect("Failed to render metrics after observations");
        assert!(!rendered.is_empty());
    }

    #[test]
    fn test_metrics_thread_safety() {
        use std::sync::Arc;
        use std::thread;
        
        let metrics = Arc::new(Metrics::new().expect("Failed to create metrics"));
        let mut handles = vec![];
        
        // Spawn multiple threads to test thread safety
        for i in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.record_connection_start();
                    metrics_clone.record_event_received();
                    metrics_clone.record_query_received();
                    if i % 2 == 0 {
                        metrics_clone.record_connection_end(0.1);
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        // Verify metrics were updated (exact values depend on scheduling)
        assert!(metrics.total_connections.get() > 0.0);
        assert!(metrics.events_received.get() > 0.0);
        assert!(metrics.queries_received.get() > 0.0);
    }
}
