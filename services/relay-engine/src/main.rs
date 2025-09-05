use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, ConnectInfo,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use nostr::{Event, Filter, RelayMessage, ClientMessage, SubscriptionId};
use serde_json;
use std::{
    collections::HashMap,
    net::{SocketAddr, IpAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, time::timeout, sync::RwLock};
use tracing::{error, info, warn, debug};
use uuid::Uuid;

mod config;
mod database;
mod metrics;
mod rate_limiter;
mod app_state;

use config::Config;
use database::PostgresDatabase;
use metrics::Metrics;
use rate_limiter::{RateLimiter, RateLimitConfig};
use app_state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env();
    info!("Starting Pleb.One Relay with config: {:?}", config);
    
    // Initialize database
    let database = PostgresDatabase::new(&config.database_url).await?;
    database.create_tables().await?;
    info!("Database connected and tables created successfully");
    
    // Initialize metrics
    let metrics = Metrics::new()?;
    info!("Metrics initialized");
    
    // Initialize rate limiter
    let rate_limit_config = RateLimitConfig::default();
    let rate_limiter = RateLimiter::new(rate_limit_config);
    info!("Rate limiter initialized");
    
    // Create application state
    let state = AppState {
        database,
        subscriptions: Arc::new(RwLock::new(HashMap::new())),
        rate_limiter,
        metrics,
        config: config.clone(),
    };

    // Build the application
    let app = Router::new()
        .route("/", get(websocket_handler))
        .route("/metrics", get(metrics_handler))
        .merge(metrics::create_metrics_api_router())
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("Pleb.One Relay listening on {}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    
    Ok(())
}

// Handler functions
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state, addr.ip()))
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics.render() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error generating metrics".to_string()),
    }
}

async fn handle_websocket(socket: WebSocket, state: AppState, client_ip: IpAddr) {
    let client_id = Uuid::new_v4().to_string();
    let connection_start = Instant::now();
    
    // Check connection limit
    if !state.rate_limiter.check_connection_limit(client_ip).await.unwrap_or(false) {
        warn!("Connection limit exceeded for IP: {}", client_ip);
        state.metrics.record_rate_limit_connection();
        return;
    }

    info!("New client connected: {} from {}", client_id, client_ip);
    
    // Record connection metrics
    state.metrics.record_connection_start();
    let _ = state.rate_limiter.add_connection(client_ip).await;

    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_message(
                    &text,
                    &client_id,
                    client_ip,
                    &state,
                    &mut sender,
                ).await {
                    error!("Error handling message from {}: {}", client_id, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} disconnected", client_id);
                break;
            }
            Err(e) => {
                error!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    cleanup_client_subscriptions(&client_id, &state).await;
    let _ = state.rate_limiter.remove_connection(client_ip).await;
    
    let connection_duration = connection_start.elapsed().as_secs_f64();
    state.metrics.record_connection_end(connection_duration);
    
    info!("Client {} session ended", client_id);
}

async fn handle_client_message(
    message: &str,
    client_id: &str,
    client_ip: IpAddr,
    state: &AppState,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let start_time = Instant::now();

    // Parse the client message
    let client_message: ClientMessage = match serde_json::from_str(message) {
        Ok(msg) => msg,
        Err(e) => {
            warn!("Invalid message format from client {}: {}", client_id, e);
            let error_msg = RelayMessage::Notice {
                message: "Invalid message format".to_string(),
            };
            send_message(sender, &error_msg).await?;
            return Ok(());
        }
    };

    match client_message {
        ClientMessage::Event(event) => {
            // Check event rate limit
            if !state.rate_limiter.check_event_rate(client_ip).await? {
                state.metrics.record_rate_limit_event();
                let error_msg = RelayMessage::Notice {
                    message: "Event rate limit exceeded".to_string(),
                };
                send_message(sender, &error_msg).await?;
                return Ok(());
            }
            
            state.metrics.record_event_received();
            handle_event_message(*event, client_id, state, sender).await?;
        }
        ClientMessage::Req { subscription_id, filters } => {
            // Check query rate limit
            if !state.rate_limiter.check_query_rate(client_ip).await? {
                let error_msg = RelayMessage::Notice {
                    message: "Query rate limit exceeded".to_string(),
                };
                send_message(sender, &error_msg).await?;
                return Ok(());
            }
            
            state.metrics.record_query_received();
            handle_req_message(subscription_id.to_string(), filters, client_id, state, sender).await?;
        }
        ClientMessage::Close(subscription_id) => {
            handle_close_message(subscription_id.to_string(), client_id, state).await?;
        }
        _ => {
            debug!("Unhandled message type from client {}", client_id);
        }
    }

    let processing_time = start_time.elapsed().as_secs_f64();
    debug!("Message processed in {:.3}ms", processing_time * 1000.0);

    Ok(())
}

async fn handle_event_message(
    event: Event,
    client_id: &str,
    state: &AppState,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let start_time = Instant::now();
    debug!("Received event from client {}: {}", client_id, event.id);

    // Validate the event
    if let Err(e) = event.verify() {
        warn!("Invalid event signature from client {}: {}", client_id, e);
        let response = RelayMessage::Ok {
            event_id: event.id,
            status: false,
            message: "Invalid event signature".to_string(),
        };
        send_message(sender, &response).await?;
        
        let processing_time = start_time.elapsed().as_secs_f64();
        state.metrics.record_event_rejected(processing_time);
        return Ok(());
    }

    // Check if event already exists
    if state.database.event_exists(&event.id).await? {
        let response = RelayMessage::Ok {
            event_id: event.id,
            status: true,
            message: "duplicate: event already exists".to_string(),
        };
        send_message(sender, &response).await?;
        
        let processing_time = start_time.elapsed().as_secs_f64();
        state.metrics.record_event_stored(processing_time);
        return Ok(());
    }

    // Store the event in database
    let db_start = Instant::now();
    match state.database.save_event(&event).await {
        Ok(_) => {
            let db_duration = db_start.elapsed().as_secs_f64();
            state.metrics.record_database_operation(db_duration);
            
            debug!("Stored event {} from client {}", event.id, client_id);
            
            // Send success response
            let response = RelayMessage::Ok {
                event_id: event.id,
                status: true,
                message: "".to_string(),
            };
            send_message(sender, &response).await?;
            
            let processing_time = start_time.elapsed().as_secs_f64();
            state.metrics.record_event_stored(processing_time);
        }
        Err(e) => {
            state.metrics.record_database_error();
            error!("Failed to store event: {}", e);
            let response = RelayMessage::Ok {
                event_id: event.id,
                status: false,
                message: "Failed to store event".to_string(),
            };
            send_message(sender, &response).await?;
            
            let processing_time = start_time.elapsed().as_secs_f64();
            state.metrics.record_event_rejected(processing_time);
        }
    }

    Ok(())
}

async fn handle_req_message(
    subscription_id: String,
    filters: Vec<Filter>,
    client_id: &str,
    state: &AppState,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let start_time = Instant::now();
    debug!("REQ from client {}: subscription {}", client_id, subscription_id);

    // Store subscription
    {
        let mut subs = state.subscriptions.write().await;
        let client_subs = subs.entry(client_id.to_string()).or_insert_with(HashMap::new);
        
        for (i, filter) in filters.iter().enumerate() {
            let filter_key = format!("{}:{}", subscription_id, i);
            client_subs.insert(filter_key, filter.clone());
        }
    }
    
    state.metrics.record_subscription_start();

    // Query existing events that match the filters
    for filter in filters {
        let db_start = Instant::now();
        let events = state.database.query_events(&filter).await?;
        let db_duration = db_start.elapsed().as_secs_f64();
        state.metrics.record_database_operation(db_duration);
        
        for event in events {
            let response = RelayMessage::Event {
                subscription_id: SubscriptionId::new(subscription_id.clone()),
                event: Box::new(event),
            };
            send_message(sender, &response).await?;
        }
    }

    // Send EOSE (End of Stored Events)
    let eose = RelayMessage::EndOfStoredEvents(SubscriptionId::new(subscription_id));
    send_message(sender, &eose).await?;

    let processing_time = start_time.elapsed().as_secs_f64();
    state.metrics.record_query_processed(processing_time);

    Ok(())
}

async fn handle_close_message(
    subscription_id: String,
    client_id: &str,
    state: &AppState,
) -> anyhow::Result<()> {
    debug!("CLOSE from client {}: subscription {}", client_id, subscription_id);

    // Remove subscription
    {
        let mut subs = state.subscriptions.write().await;
        if let Some(client_subs) = subs.get_mut(client_id) {
            let before_count = client_subs.len();
            client_subs.retain(|key, _| !key.starts_with(&format!("{}:", subscription_id)));
            let removed_count = before_count - client_subs.len();
            
            // Update metrics for each removed subscription
            for _ in 0..removed_count {
                state.metrics.record_subscription_end();
            }
        }
    }

    Ok(())
}

async fn cleanup_client_subscriptions(client_id: &str, state: &AppState) {
    let mut subs = state.subscriptions.write().await;
    if let Some(client_subs) = subs.remove(client_id) {
        // Update metrics for all removed subscriptions
        for _ in 0..client_subs.len() {
            state.metrics.record_subscription_end();
        }
        debug!("Cleaned up {} subscriptions for client {}", client_subs.len(), client_id);
    }
}

async fn send_message(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    relay_message: &RelayMessage,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(relay_message)?;
    
    // Add timeout to prevent hanging
    match timeout(Duration::from_secs(5), sender.send(Message::Text(json))).await {
        Ok(result) => result.map_err(Into::into),
        Err(_) => {
            error!("Timeout sending message to client");
            Err(anyhow::anyhow!("Send timeout"))
        }
    }
}

