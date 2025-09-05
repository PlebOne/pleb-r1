use anyhow::Result;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use pleb_one_config::Config;
use pleb_one_storage::Storage;
use serde_json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error, warn, debug};
use uuid::Uuid;

use crate::connection::{Connection, ConnectionManager};
use crate::event_handler::EventHandler;
use crate::metrics::MetricsCollector;

pub struct RelayServer {
    config: Config,
    storage: Arc<Storage>,
    connection_manager: Arc<ConnectionManager>,
    event_handler: Arc<EventHandler>,
    metrics: Arc<MetricsCollector>,
}

impl RelayServer {
    pub async fn new(config: Config, storage: Arc<Storage>) -> Result<Self> {
        let connection_manager = Arc::new(ConnectionManager::new());
        let event_handler = Arc::new(EventHandler::new(storage.clone()).await?);
        let metrics = Arc::new(MetricsCollector::new(&config).await?);

        Ok(Self {
            config,
            storage,
            connection_manager,
            event_handler,
            metrics,
        })
    }

    pub async fn run(self) -> Result<()> {
        let state = AppState {
            storage: self.storage,
            connection_manager: self.connection_manager,
            event_handler: self.event_handler,
            metrics: self.metrics,
            config: Arc::new(self.config.clone()),
        };

        let app = Router::new()
            .route("/", get(websocket_handler))
            .with_state(state);

        let bind_address = format!("0.0.0.0:{}", self.config.relay.port);
        let listener = TcpListener::bind(&bind_address).await?;
        
        info!("🚀 Pleb.One Relay WebSocket server listening on {}", bind_address);
        info!("📡 Connect with: ws://{}", bind_address);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    storage: Arc<Storage>,
    connection_manager: Arc<ConnectionManager>,
    event_handler: Arc<EventHandler>,
    metrics: Arc<MetricsCollector>,
    config: Arc<Config>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let connection_id = Uuid::new_v4();
    let (mut sender, mut receiver) = socket.split();

    info!("🔌 New client connected: {}", connection_id);
    
    // Create connection and register with manager
    let connection = Arc::new(Connection::new(connection_id));
    state.connection_manager.add_connection(connection.clone()).await;
    state.metrics.increment_connections().await;

    // Handle incoming messages
    let state_clone = state.clone();
    let connection_clone = connection.clone();
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("📨 Received message from {}: {}", connection_id, text);
                    
                    if let Err(e) = handle_client_message(
                        &text,
                        &connection_clone,
                        &state_clone,
                    ).await {
                        error!("❌ Error handling message from {}: {}", connection_id, e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("👋 Client {} disconnected", connection_id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("💥 WebSocket error for {}: {}", connection_id, e);
                    break;
                }
                _ => {} // Ignore other message types
            }
        }
    });

    // Handle outgoing messages
    let mut rx = connection.subscribe_to_messages().await;
    let outgoing_task = tokio::spawn(async move {
        while let Ok(relay_msg) = rx.recv().await {
            let json = match serde_json::to_string(&relay_msg) {
                Ok(json) => json,
                Err(e) => {
                    error!("❌ Failed to serialize message: {}", e);
                    continue;
                }
            };

            debug!("📤 Sending to {}: {}", connection_id, json);
            
            if sender.send(Message::Text(json)).await.is_err() {
                debug!("🔌 Connection {} closed while sending", connection_id);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = incoming_task => {},
        _ = outgoing_task => {},
    }

    // Cleanup
    state.connection_manager.remove_connection(connection_id).await;
    state.metrics.decrement_connections().await;
    info!("🧹 Client {} cleanup completed", connection_id);
}

async fn handle_client_message(
    text: &str,
    connection: &Arc<Connection>,
    state: &AppState,
) -> Result<()> {
    use nostr_types::{ClientMessage, RelayMessage};
    
    // Parse the client message
    let client_msg: ClientMessage = serde_json::from_str(text)?;
    
    // Update connection activity
    connection.update_last_activity().await;
    
    match client_msg {
        ClientMessage::Event(event) => {
            info!("📝 Received EVENT from {}: {}", connection.id(), event.id);
            
            // Validate and process event
            match state.event_handler.process_event(event, connection).await {
                Ok(accepted) => {
                    let response = if accepted {
                        RelayMessage::Ok {
                            event_id: event.id.clone(),
                            accepted: true,
                            message: "".to_string(),
                        }
                    } else {
                        RelayMessage::Ok {
                            event_id: event.id.clone(),
                            accepted: false,
                            message: "Event rejected".to_string(),
                        }
                    };
                    
                    connection.send_message(response).await?;
                    
                    if accepted {
                        state.metrics.record_event_processed().await;
                        // Broadcast to relevant subscribers
                        state.connection_manager.broadcast_event(&event).await;
                    }
                }
                Err(e) => {
                    error!("❌ Failed to process event {}: {}", event.id, e);
                    let response = RelayMessage::Ok {
                        event_id: event.id,
                        accepted: false,
                        message: format!("Error: {}", e),
                    };
                    connection.send_message(response).await?;
                }
            }
        }
        
        ClientMessage::Req { subscription_id, filters } => {
            info!("🔍 Received REQ from {}: {} with {} filters", 
                  connection.id(), subscription_id, filters.len());
            
            // Query historical events
            match state.storage.query_events(&filters).await {
                Ok(events) => {
                    info!("📦 Found {} historical events for subscription {}", 
                          events.len(), subscription_id);
                    
                    // Send historical events
                    for event in events {
                        let msg = RelayMessage::Event {
                            subscription_id: subscription_id.clone(),
                            event,
                        };
                        connection.send_message(msg).await?;
                    }
                    
                    // Send EOSE
                    let eose = RelayMessage::Eose(subscription_id.clone());
                    connection.send_message(eose).await?;
                    
                    // Add subscription for future events
                    connection.add_subscription(subscription_id, filters).await;
                    state.metrics.record_subscription_created().await;
                }
                Err(e) => {
                    error!("❌ Failed to query events: {}", e);
                    let notice = RelayMessage::Notice(format!("Query failed: {}", e));
                    connection.send_message(notice).await?;
                }
            }
        }
        
        ClientMessage::Close(subscription_id) => {
            info!("❌ Received CLOSE from {}: {}", connection.id(), subscription_id);
            connection.remove_subscription(&subscription_id).await;
            state.metrics.record_subscription_closed().await;
        }
        
        ClientMessage::Auth(auth_event) => {
            info!("🔐 Received AUTH from {}", connection.id());
            
            match state.event_handler.process_auth(auth_event, connection).await {
                Ok(success) => {
                    if success {
                        info!("✅ Authentication successful for {}", connection.id());
                        state.metrics.record_auth_success().await;
                    } else {
                        warn!("🚫 Authentication failed for {}", connection.id());
                        state.metrics.record_auth_failure().await;
                    }
                }
                Err(e) => {
                    error!("❌ Auth processing error for {}: {}", connection.id(), e);
                    state.metrics.record_auth_failure().await;
                }
            }
        }
        
        ClientMessage::Count { subscription_id, filters } => {
            info!("🔢 Received COUNT from {}: {}", connection.id(), subscription_id);
            
            match state.storage.count_events(&filters).await {
                Ok(count) => {
                    let response = RelayMessage::Count {
                        subscription_id,
                        count,
                    };
                    connection.send_message(response).await?;
                }
                Err(e) => {
                    error!("❌ Failed to count events: {}", e);
                    let notice = RelayMessage::Notice(format!("Count failed: {}", e));
                    connection.send_message(notice).await?;
                }
            }
        }
    }
    
    Ok(())
}
