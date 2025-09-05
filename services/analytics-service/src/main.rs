use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use uuid::Uuid;

mod analytics;
mod metrics;
mod reports;

use analytics::AnalyticsEngine;
use config_manager::Config;

#[derive(Clone)]
pub struct AppState {
    analytics: Arc<AnalyticsEngine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficEvent {
    pub event_id: String,
    pub client_id: Option<String>,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub report_type: Option<String>,
    pub granularity: Option<String>, // hour, day, week, month
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficReport {
    pub period: String,
    pub total_events: u64,
    pub unique_clients: u64,
    pub events_by_type: HashMap<String, u64>,
    pub peak_concurrent_connections: u64,
    pub bandwidth_usage: u64, // bytes
    pub error_rate: f64,
    pub response_times: ResponseTimeStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTimeStats {
    pub average_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeMetrics {
    pub active_connections: u64,
    pub events_per_second: f64,
    pub subscriptions_count: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_usage: u64,
}

async fn record_traffic_event(
    State(state): State<AppState>,
    Json(event): Json<TrafficEvent>,
) -> Result<StatusCode, StatusCode> {
    match state.analytics.record_event(event).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!("Failed to record traffic event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_traffic_report(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<TrafficReport>, StatusCode> {
    match state.analytics.generate_report(query).await {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            error!("Failed to generate traffic report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_realtime_metrics(
    State(state): State<AppState>,
) -> Result<Json<RealtimeMetrics>, StatusCode> {
    match state.analytics.get_realtime_metrics().await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => {
            error!("Failed to get realtime metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn export_report(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<String, StatusCode> {
    match state.analytics.export_csv_report(query).await {
        Ok(csv_data) => Ok(csv_data),
        Err(e) => {
            error!("Failed to export report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    let config = Config::load("analytics-service")?;
    let analytics = Arc::new(AnalyticsEngine::new(&config).await?);

    let state = AppState { analytics };

    let app = Router::new()
        .route("/events", post(record_traffic_event))
        .route("/reports/traffic", get(get_traffic_report))
        .route("/metrics/realtime", get(get_realtime_metrics))
        .route("/reports/export", get(export_report))
        .with_state(state);

    let listener = TcpListener::bind(&config.server.bind_address).await?;
    info!("Analytics service listening on {}", config.server.bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
