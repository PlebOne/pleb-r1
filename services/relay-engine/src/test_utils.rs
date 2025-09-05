use crate::{config::Config, database::PostgresDatabase, metrics::Metrics, rate_limiter::{RateLimiter, RateLimitConfig}, app_state::AppState};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Create a test AppState for development and testing
pub async fn create_mock_app_state() -> anyhow::Result<AppState> {
    let config = Config::from_env();
    let metrics = Metrics::new()?;
    let rate_limit_config = RateLimitConfig::default();
    let rate_limiter = RateLimiter::new(rate_limit_config);
    
    // Create in-memory database for testing
    let database = PostgresDatabase::new("sqlite::memory:").await?;
    database.create_tables().await?;
    
    Ok(AppState {
        database,
        subscriptions: Arc::new(RwLock::new(HashMap::new())),
        rate_limiter,
        metrics,
        config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use crate::metrics::create_metrics_api_router;

    #[tokio::test]
    async fn test_metrics_api_endpoints() {
        // Create mock app state
        let state = create_mock_app_state().await.expect("Failed to create app state");
        
        // Create router with metrics endpoints
        let app = create_metrics_api_router().with_state(state.clone());
        
        // Test relay status endpoint
        let request = Request::builder()
            .uri("/api/metrics/relay-status")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test events endpoint
        let request = Request::builder()
            .uri("/api/metrics/events")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test performance endpoint
        let request = Request::builder()
            .uri("/api/metrics/performance")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test all metrics endpoint
        let request = Request::builder()
            .uri("/api/metrics/all")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
