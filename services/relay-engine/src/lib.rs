// Nostr Relay Engine Library
// High-performance relay implementation using rust-nostr

pub mod config;
pub mod database;
pub mod metrics;
pub mod rate_limiter;
pub mod app_state;
pub mod test_utils;
pub mod mock_database;

// Re-export main types
pub use config::Config;
pub use database::PostgresDatabase;
pub use metrics::Metrics;
pub use rate_limiter::{RateLimiter, RateLimitConfig};
pub use app_state::AppState;

use axum::{
    routing::get,
    Router,
    extract::State,
    response::Json,
};
use serde_json::{json, Value};

// Create the main application router
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(relay_info))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_check))
        .with_state(state)
}

// Relay info endpoint (NIP-11)
async fn relay_info(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "name": state.config.relay_name,
        "description": state.config.relay_description,
        "pubkey": state.config.relay_pubkey,
        "contact": state.config.relay_contact,
        "supported_nips": [1, 2, 9, 11, 12, 15, 16, 20, 22, 28, 33],
        "software": "NrelayOne",
        "version": env!("CARGO_PKG_VERSION"),
        "limitation": {
            "max_message_length": 65536,
            "max_subscriptions": 20, // Default value since config is private
            "max_filters": 100,
            "max_limit": 5000,
            "max_subid_length": 100,
            "min_prefix": 4,
            "max_event_tags": 100,
            "max_content_length": 8196,
            "min_pow_difficulty": 0,
            "auth_required": false,
            "payment_required": false
        },
        "payments_url": null,
        "fees": {}
    }))
}

// Metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics.render().unwrap_or_else(|_| "# Metrics unavailable\n".to_string())
}

// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}
