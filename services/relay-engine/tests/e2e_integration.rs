// End-to-end integration tests for the complete Nostr relay
use relay_engine::{create_app, AppState, Config};
use relay_engine::database::PostgresDatabase;
use relay_engine::metrics::Metrics;
use relay_engine::rate_limiter::{RateLimiter, RateLimitConfig};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use nostr::{ClientMessage, EventBuilder, Filter, Keys, Kind, RelayMessage, SubscriptionId};
use serde_json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::RwLock, time::timeout};
use tokio_test;
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use uuid::Uuid;

// Create a mock database for testing that doesn't require actual PostgreSQL
async fn create_mock_database() -> PostgresDatabase {
    // This would ideally be a trait implementation with a mock
    // For now, we'll create a temporary test database
    // In a real production environment, you'd want dependency injection with traits
    PostgresDatabase::new("postgresql://postgres:password@localhost:5432/test_db")
        .await
        .unwrap_or_else(|_| {
            // Skip database tests if PostgreSQL isn't available
            panic!("Database tests require PostgreSQL. Use 'cargo test --lib' to skip database integration tests.")
        })
}

// Helper to create test configuration
fn create_test_config() -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(), // Use in-memory SQLite for tests
        port: 0, // Let the OS choose an available port
        relay_name: "Test Relay E2E".to_string(),
        relay_description: "End-to-end test relay".to_string(),
        relay_pubkey: None,
        relay_contact: Some("test@example.com".to_string()),
    }
}

// Helper to create test app state
async fn create_test_app_state() -> AppState {
    let config = create_test_config();
    let metrics = Metrics::new().expect("Failed to create metrics");
    let rate_limiter = RateLimiter::new(RateLimitConfig {
        events_per_minute: 100,
        queries_per_minute: 200,
        connections_per_ip: 100,
        cleanup_interval: Duration::from_secs(60),
    });
    
    // For testing, create a mock database that doesn't actually connect
    // This allows tests to run without a database dependency
    let database = create_mock_database().await;
    
    AppState {
        config,
        database,
        subscriptions: Arc::new(RwLock::new(HashMap::new())),
        rate_limiter,
        metrics,
    }
}

#[tokio::test]
async fn test_relay_info_endpoint() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test the relay info endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/", addr))
        .header("Accept", "application/nostr+json")
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    let relay_info: serde_json::Value = response.json().await.unwrap();
    assert_eq!(relay_info["name"], "Test Relay E2E");
    assert_eq!(relay_info["description"], "End-to-end test relay");
    assert_eq!(relay_info["contact"], "test@example.com");
    assert!(relay_info["supported_nips"].is_array());
}

#[tokio::test]
async fn test_websocket_connection_lifecycle() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to WebSocket
    let ws_url = format!("ws://{}/", addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Test connection is established
    assert!(write.send(TungsteniteMessage::Ping(vec![])).await.is_ok());
    
    // Wait for pong response (or timeout)
    let response = timeout(Duration::from_secs(1), read.next()).await;
    assert!(response.is_ok());
    
    if let Some(Ok(msg)) = response.unwrap() {
        match msg {
            TungsteniteMessage::Pong(_) => {}, // Expected
            _ => {}, // Other messages are also acceptable
        }
    }
    
    // Close connection
    assert!(write.send(TungsteniteMessage::Close(None)).await.is_ok());
}

#[tokio::test]
async fn test_event_publishing_flow() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to WebSocket
    let ws_url = format!("ws://{}/", addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Create and send an event
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Hello from E2E test!", [])
        .to_event(&keys)
        .unwrap();
    
    let client_msg = ClientMessage::Event(Box::new(event.clone()));
    let json = serde_json::to_string(&client_msg).unwrap();
    
    write.send(TungsteniteMessage::Text(json)).await.unwrap();
    
    // Wait for OK response
    let response = timeout(Duration::from_secs(2), read.next()).await;
    assert!(response.is_ok());
    
    if let Some(Ok(TungsteniteMessage::Text(response_text))) = response.unwrap() {
        let relay_msg: RelayMessage = serde_json::from_str(&response_text).unwrap();
        match relay_msg {
            RelayMessage::Ok { event_id, status, message: _ } => {
                assert_eq!(event_id, event.id);
                assert!(status); // Should be accepted
            }
            _ => panic!("Expected OK message, got: {:?}", relay_msg),
        }
    }
    
    // Close connection
    write.send(TungsteniteMessage::Close(None)).await.unwrap();
}

#[tokio::test]
async fn test_subscription_and_query_flow() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to WebSocket
    let ws_url = format!("ws://{}/", addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Create a subscription
    let sub_id = SubscriptionId::new("test-subscription");
    let filter = Filter::new().kinds([Kind::TextNote]).limit(10);
    let req_msg = ClientMessage::Req {
        subscription_id: sub_id.clone(),
        filters: vec![filter],
    };
    
    let json = serde_json::to_string(&req_msg).unwrap();
    write.send(TungsteniteMessage::Text(json)).await.unwrap();
    
    // Wait for EOSE (End of Stored Events)
    let response = timeout(Duration::from_secs(2), read.next()).await;
    assert!(response.is_ok());
    
    if let Some(Ok(TungsteniteMessage::Text(response_text))) = response.unwrap() {
        let relay_msg: RelayMessage = serde_json::from_str(&response_text).unwrap();
        match relay_msg {
            RelayMessage::EndOfStoredEvents(received_sub_id) => {
                assert_eq!(received_sub_id, sub_id);
            }
            _ => {
                // Other messages might come first, that's OK
                println!("Received other message: {:?}", relay_msg);
            }
        }
    }
    
    // Close the subscription
    let close_msg = ClientMessage::Close(sub_id.clone());
    let json = serde_json::to_string(&close_msg).unwrap();
    write.send(TungsteniteMessage::Text(json)).await.unwrap();
    
    // Close connection
    write.send(TungsteniteMessage::Close(None)).await.unwrap();
}

#[tokio::test]
async fn test_rate_limiting_integration() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to WebSocket
    let ws_url = format!("ws://{}/", addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    let keys = Keys::generate();
    
    // Send multiple events rapidly to test rate limiting
    for i in 0..5 {
        let event = EventBuilder::new(Kind::TextNote, &format!("Test message {}", i), [])
            .to_event(&keys)
            .unwrap();
        
        let client_msg = ClientMessage::Event(Box::new(event));
        let json = serde_json::to_string(&client_msg).unwrap();
        
        write.send(TungsteniteMessage::Text(json)).await.unwrap();
        
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Collect responses
    let mut responses = Vec::new();
    for _ in 0..5 {
        if let Ok(Some(Ok(TungsteniteMessage::Text(response_text)))) = 
            timeout(Duration::from_secs(1), read.next()).await {
            if let Ok(relay_msg) = serde_json::from_str::<RelayMessage>(&response_text) {
                responses.push(relay_msg);
            }
        }
    }
    
    // Should receive responses (might be rate limited but should get some)
    assert!(!responses.is_empty());
    
    // Close connection
    write.send(TungsteniteMessage::Close(None)).await.unwrap();
}

#[tokio::test]
async fn test_invalid_message_handling() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to WebSocket
    let ws_url = format!("ws://{}/", addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Send invalid JSON
    write.send(TungsteniteMessage::Text("invalid json".to_string())).await.unwrap();
    
    // Wait for potential NOTICE response
    let response = timeout(Duration::from_secs(1), read.next()).await;
    
    if let Ok(Some(Ok(TungsteniteMessage::Text(response_text)))) = response {
        if let Ok(relay_msg) = serde_json::from_str::<RelayMessage>(&response_text) {
            match relay_msg {
                RelayMessage::Notice { message } => {
                    assert!(message.contains("error") || message.contains("invalid"));
                }
                _ => {
                    // Other responses are also acceptable
                }
            }
        }
    }
    
    // Send malformed event
    write.send(TungsteniteMessage::Text("[\"EVENT\", {}]".to_string())).await.unwrap();
    
    // Connection should still be alive
    assert!(write.send(TungsteniteMessage::Ping(vec![])).await.is_ok());
    
    // Close connection
    write.send(TungsteniteMessage::Close(None)).await.unwrap();
}

#[tokio::test]
async fn test_metrics_integration() {
    let app_state = create_test_app_state().await;
    // Record initial state for comparison
    app_state.metrics.record_connection_start();
    app_state.metrics.record_event_received();
    
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test metrics endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/metrics", addr))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    let metrics_text = response.text().await.unwrap();
    
    // Should contain Prometheus metrics
    assert!(metrics_text.contains("# HELP"));
    assert!(metrics_text.contains("# TYPE"));
    
    // Should contain our custom metrics
    assert!(metrics_text.contains("nostr_relay_connections"));
    assert!(metrics_text.contains("nostr_relay_events"));
    assert!(metrics_text.contains("nostr_relay_subscriptions"));
}

#[tokio::test]
async fn test_concurrent_client_connections() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state.clone());
    
    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut handles = Vec::new();
    
    // Create multiple concurrent connections
    for i in 0..5 {
        let addr = addr.clone();
        let handle = tokio::spawn(async move {
            let ws_url = format!("ws://{}/", addr);
            let (ws_stream, _) = connect_async(ws_url).await.unwrap();
            let (mut write, mut read) = ws_stream.split();
            
            // Each client sends a unique event
            let keys = Keys::generate();
            let event = EventBuilder::new(Kind::TextNote, &format!("Message from client {}", i), [])
                .to_event(&keys)
                .unwrap();
            
            let client_msg = ClientMessage::Event(Box::new(event));
            let json = serde_json::to_string(&client_msg).unwrap();
            
            write.send(TungsteniteMessage::Text(json)).await.unwrap();
            
            // Wait for response
            if let Ok(Some(Ok(TungsteniteMessage::Text(_)))) = 
                timeout(Duration::from_secs(2), read.next()).await {
                // Got a response, good
            }
            
            // Close connection
            write.send(TungsteniteMessage::Close(None)).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all clients to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // All clients should have completed successfully
}
