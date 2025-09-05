// Development server that works without database for frontend testing
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tower_http::cors::{CorsLayer, Any};

mod config;
mod mock_database;

use config::Config;

// Simplified AppState for development
#[derive(Clone)]
pub struct DevAppState {
    pub config: Config,
}

// User registration data structures
#[derive(Debug, Serialize, Deserialize)]
struct SignupRequest {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    email: String,
    #[serde(rename = "nostrPubkey")]
    nostr_pubkey: Option<String>,
    plan: String,
    interests: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with more verbose output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("🚀 Starting Pleb.One Development Server...");
    
    let config = Config::from_env();
    println!("📋 Configuration loaded successfully");
    info!("Starting Pleb.One Development Server with Authentication");
    
    let state = DevAppState { config };

    // Build the application with CORS for frontend development
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/auth/signup", post(signup_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/metrics/events", get(events_handler))
        .route("/api/metrics/performance", get(performance_handler))
        .route("/api/metrics/all", get(all_metrics_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🔌 Attempting to bind to {}", addr);
    
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("✅ Successfully bound to port 8080");
            listener
        },
        Err(e) => {
            println!("❌ Failed to bind to port 8080: {}", e);
            return Err(e.into());
        }
    };
    
    println!("🚀 Pleb.One Development Server running on http://localhost:8080");
    println!("📊 API Endpoints available:");
    println!("   - http://localhost:8080/api/auth/signup");
    println!("   - http://localhost:8080/api/auth/login");
    println!("   - http://localhost:8080/api/metrics/all");
    println!("   - http://localhost:8080/api/metrics/events");
    println!("   - http://localhost:8080/api/metrics/performance");
    
    info!("Server is starting...");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root_handler() -> Html<&'static str> {
    Html(r#"
    <html>
    <head><title>Pleb.One Development Server</title></head>
    <body style="font-family: system-ui; padding: 2rem; max-width: 800px; margin: 0 auto;">
        <h1>🚀 Pleb.One Development Server</h1>
        <p><strong>Status:</strong> Running and ready for frontend development!</p>
        
        <h2>📊 API Endpoints</h2>
        <ul>
            <li><a href="/api/metrics/all" target="_blank">All Metrics</a></li>
            <li><a href="/api/metrics/relay-status" target="_blank">Relay Status</a></li>
            <li><a href="/api/metrics/events" target="_blank">Event Metrics</a></li>
            <li><a href="/api/metrics/performance" target="_blank">Performance Metrics</a></li>
        </ul>
        
        <h2>🎨 Frontend Testing</h2>
        <p>Open <code>services/community-web/api-test.html</code> in your browser to see the dashboard.</p>
        
        <style>
            body { background: #f8fafc; }
            h1 { color: #1e293b; }
            h2 { color: #475569; margin-top: 2rem; }
            a { color: #3b82f6; text-decoration: none; }
            a:hover { text-decoration: underline; }
            code { background: #e2e8f0; padding: 0.25rem 0.5rem; border-radius: 0.25rem; }
        </style>
    </body>
    </html>
    "#)
}

// Mock API handlers with realistic demo data
async fn relay_status_handler(State(state): State<DevAppState>) -> impl IntoResponse {
    let data = serde_json::json!({
        "active_connections": 4,
        "total_connections": 127,
        "uptime_seconds": 3600, // 1 hour
        "status": "healthy"
    });
    (StatusCode::OK, serde_json::to_string(&data).unwrap())
}

async fn events_handler(State(state): State<DevAppState>) -> impl IntoResponse {
    let data = serde_json::json!({
        "events_received": 1542,
        "events_stored": 1538,
        "events_rejected": 4,
        "avg_processing_time_ms": 23.5
    });
    (StatusCode::OK, serde_json::to_string(&data).unwrap())
}

async fn performance_handler(State(state): State<DevAppState>) -> impl IntoResponse {
    let data = serde_json::json!({
        "queries_received": 892,
        "active_subscriptions": 18,
        "rate_limited_events": 12,
        "database_operations": 2341,
        "database_errors": 0,
        "avg_query_time_ms": 15.2
    });
    (StatusCode::OK, serde_json::to_string(&data).unwrap())
}

async fn all_metrics_handler(State(state): State<DevAppState>) -> impl IntoResponse {
    let data = serde_json::json!({
        "relay_status": {
            "active_connections": 4,
            "total_connections": 127,
            "uptime_seconds": 3600,
            "status": "healthy"
        },
        "events": {
            "events_received": 1542,
            "events_stored": 1538,
            "events_rejected": 4,
            "avg_processing_time_ms": 23.5
        },
        "performance": {
            "queries_received": 892,
            "active_subscriptions": 18,
            "rate_limited_events": 12,
            "database_operations": 2341,
            "database_errors": 0,
            "avg_query_time_ms": 15.2
        }
    });
    (StatusCode::OK, serde_json::to_string(&data).unwrap())
}

// User registration handler
async fn signup_handler(Json(signup_data): Json<SignupRequest>) -> impl IntoResponse {
    info!("New user signup: {} <{}>", signup_data.first_name, signup_data.email);
    
    // Validate email format (basic validation)
    if !signup_data.email.contains('@') {
        let response = ApiResponse {
            success: false,
            message: "Invalid email format".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    
    // Validate plan
    let valid_plans = ["community", "pro", "enterprise"];
    if !valid_plans.contains(&signup_data.plan.as_str()) {
        let response = ApiResponse {
            success: false,
            message: "Invalid plan selected".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    
    // In a real implementation, you would:
    // 1. Check if email already exists
    // 2. Hash password (we don't have password in this demo)
    // 3. Store user in database
    // 4. Send verification email
    // 5. Generate auth tokens
    
    info!("User registered: plan={}, nostr_key={:?}", 
          signup_data.plan, 
          signup_data.nostr_pubkey.as_ref().map(|k| &k[..20]));
    
    let user_data = serde_json::json!({
        "id": format!("user_{}", chrono::Utc::now().timestamp()),
        "name": format!("{} {}", signup_data.first_name, signup_data.last_name),
        "email": signup_data.email,
        "plan": signup_data.plan,
        "nostr_pubkey": signup_data.nostr_pubkey,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "verified": false
    });
    
    let response = ApiResponse {
        success: true,
        message: "Account created successfully! Please check your email for verification.".to_string(),
        data: Some(user_data),
    };
    
    (StatusCode::CREATED, Json(response))
}

// User login handler
async fn login_handler(Json(login_data): Json<LoginRequest>) -> impl IntoResponse {
    info!("Login attempt: {}", login_data.email);
    
    // Validate email format
    if !login_data.email.contains('@') {
        let response = ApiResponse {
            success: false,
            message: "Invalid email format".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    
    // In a real implementation, you would:
    // 1. Look up user by email
    // 2. Verify password hash
    // 3. Check if account is verified
    // 4. Generate JWT tokens
    // 5. Update last login time
    
    // For demo purposes, accept any login
    let user_data = serde_json::json!({
        "id": "user_demo",
        "name": login_data.email.split('@').next().unwrap_or("User"),
        "email": login_data.email,
        "plan": "pro",
        "created_at": "2024-01-01T00:00:00Z",
        "verified": true,
        "token": format!("demo_token_{}", chrono::Utc::now().timestamp())
    });
    
    let response = ApiResponse {
        success: true,
        message: "Login successful!".to_string(),
        data: Some(user_data),
    };
    
    (StatusCode::OK, Json(response))
}
