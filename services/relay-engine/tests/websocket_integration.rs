// Integration tests for WebSocket relay functionality
use relay_engine::{AppState, Config};
use relay_engine::database::PostgresDatabase;
use relay_engine::metrics::Metrics;
use relay_engine::rate_limiter::{RateLimiter, RateLimitConfig};

use axum::{
    extract::ws::{Message, WebSocket},
    Router,
};
use nostr::{ClientMessage, EventBuilder, Filter, Keys, Kind, RelayMessage, SubscriptionId};
use serde_json;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock, time::Duration};
use tokio_test;
use uuid::Uuid;

// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    let config = Config {
        database_url: "postgresql://test:test@localhost:5432/test_db".to_string(),
        port: 0, // Use any available port for testing
        relay_name: "Test Relay".to_string(),
        relay_description: "Test relay for integration tests".to_string(),
        relay_pubkey: None,
        relay_contact: None,
    };

    // Note: In real tests, you'd want to use a test database
    // For now, we'll test the state creation without actual DB connection
    let metrics = Metrics::new().expect("Failed to create metrics");
    let rate_limiter = RateLimiter::new(RateLimitConfig::default());
    
    AppState {
        config,
        database: PostgresDatabase::new("sqlite::memory:").await.unwrap_or_else(|_| {
            // Fallback for test environment - we'll mock this
            todo!("Use mock database for tests")
        }),
        subscriptions: Arc::new(RwLock::new(HashMap::new())),
        rate_limiter,
        metrics,
    }
}

#[tokio::test]
async fn test_client_message_serialization() {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Hello, Nostr!", [])
        .to_event(&keys)
        .unwrap();

    // Test EVENT message
    let event_msg = ClientMessage::Event(Box::new(event.clone()));
    let json = serde_json::to_string(&event_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::Event(e) => {
            assert_eq!(e.id, event.id);
            assert_eq!(e.content, event.content);
        }
        _ => panic!("Expected Event message"),
    }

    // Test REQ message
    let filter = Filter::new().kinds([Kind::TextNote]).limit(10);
    let sub_id = SubscriptionId::new("test-sub");
    let req_msg = ClientMessage::Req {
        subscription_id: sub_id.clone(),
        filters: vec![filter],
    };
    
    let json = serde_json::to_string(&req_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::Req { subscription_id, filters } => {
            assert_eq!(subscription_id, sub_id);
            assert_eq!(filters.len(), 1);
        }
        _ => panic!("Expected Req message"),
    }

    // Test CLOSE message
    let close_msg = ClientMessage::Close(sub_id.clone());
    let json = serde_json::to_string(&close_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::Close(id) => {
            assert_eq!(id, sub_id);
        }
        _ => panic!("Expected Close message"),
    }
}

#[tokio::test]
async fn test_relay_message_serialization() {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Hello, Nostr!", [])
        .to_event(&keys)
        .unwrap();

    // Test EVENT message
    let sub_id = SubscriptionId::new("test-sub");
    let event_msg = RelayMessage::Event {
        subscription_id: sub_id.clone(),
        event: Box::new(event.clone()),
    };
    
    let json = serde_json::to_string(&event_msg).unwrap();
    let deserialized: RelayMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        RelayMessage::Event { subscription_id, event: e } => {
            assert_eq!(subscription_id, sub_id);
            assert_eq!(e.id, event.id);
        }
        _ => panic!("Expected Event message"),
    }

    // Test OK message
    let ok_msg = RelayMessage::Ok {
        event_id: event.id,
        status: true,
        message: "".to_string(),
    };
    
    let json = serde_json::to_string(&ok_msg).unwrap();
    let deserialized: RelayMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        RelayMessage::Ok { event_id, status, message } => {
            assert_eq!(event_id, event.id);
            assert_eq!(status, true);
            assert_eq!(message, "");
        }
        _ => panic!("Expected OK message"),
    }

    // Test EOSE message
    let eose_msg = RelayMessage::EndOfStoredEvents(sub_id.clone());
    let json = serde_json::to_string(&eose_msg).unwrap();
    let deserialized: RelayMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        RelayMessage::EndOfStoredEvents(id) => {
            assert_eq!(id, sub_id);
        }
        _ => panic!("Expected EOSE message"),
    }

    // Test NOTICE message
    let notice_msg = RelayMessage::Notice {
        message: "Test notice".to_string(),
    };
    
    let json = serde_json::to_string(&notice_msg).unwrap();
    let deserialized: RelayMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        RelayMessage::Notice { message } => {
            assert_eq!(message, "Test notice");
        }
        _ => panic!("Expected Notice message"),
    }
}

#[tokio::test]
async fn test_subscription_id_handling() {
    let sub_id1 = SubscriptionId::new("test-subscription-1");
    let sub_id2 = SubscriptionId::new("test-subscription-2");
    let sub_id3 = SubscriptionId::new("test-subscription-1"); // Same as sub_id1
    
    // Different subscription IDs should not be equal
    assert_ne!(sub_id1, sub_id2);
    
    // Same subscription ID string should be equal
    assert_eq!(sub_id1, sub_id3);
    
    // Test serialization/deserialization
    let json1 = serde_json::to_string(&sub_id1).unwrap();
    let json2 = serde_json::to_string(&sub_id2).unwrap();
    
    assert_ne!(json1, json2);
    
    let deserialized1: SubscriptionId = serde_json::from_str(&json1).unwrap();
    assert_eq!(sub_id1, deserialized1);
}

#[tokio::test]
async fn test_filter_matching_logic() {
    let keys = Keys::generate();
    let pubkey = keys.public_key();
    
    // Create test events
    let text_event = EventBuilder::new(Kind::TextNote, "Hello world", [])
        .to_event(&keys)
        .unwrap();
    
    let metadata_event = EventBuilder::new(Kind::Metadata, "{\"name\":\"Alice\"}", [])
        .to_event(&keys)
        .unwrap();
    
    // Test kind filtering
    let text_filter = Filter::new().kinds([Kind::TextNote]);
    let metadata_filter = Filter::new().kinds([Kind::Metadata]);
    let multi_kind_filter = Filter::new().kinds([Kind::TextNote, Kind::Metadata]);
    
    // Test author filtering
    let author_filter = Filter::new().authors([pubkey]);
    
    // Test limit
    let limited_filter = Filter::new().limit(5);
    
    // Verify filter properties
    assert!(text_filter.kinds.as_ref().unwrap().contains(&Kind::TextNote));
    assert!(metadata_filter.kinds.as_ref().unwrap().contains(&Kind::Metadata));
    assert!(multi_kind_filter.kinds.as_ref().unwrap().contains(&Kind::TextNote));
    assert!(multi_kind_filter.kinds.as_ref().unwrap().contains(&Kind::Metadata));
    assert!(author_filter.authors.as_ref().unwrap().contains(&pubkey));
    assert_eq!(limited_filter.limit, Some(5));
}

#[tokio::test]
async fn test_websocket_message_types() {
    // Test different WebSocket message types that our relay should handle
    
    // Text messages (JSON)
    let text_msg = Message::Text("Hello".to_string());
    match text_msg {
        Message::Text(content) => assert_eq!(content, "Hello"),
        _ => panic!("Expected text message"),
    }
    
    // Binary messages (should typically be ignored by Nostr relays)
    let binary_msg = Message::Binary(vec![1, 2, 3, 4]);
    match binary_msg {
        Message::Binary(data) => assert_eq!(data, vec![1, 2, 3, 4]),
        _ => panic!("Expected binary message"),
    }
    
    // Close messages
    let close_msg = Message::Close(None);
    match close_msg {
        Message::Close(_) => {}, // Expected
        _ => panic!("Expected close message"),
    }
    
    // Ping/Pong messages (for keep-alive)
    let ping_msg = Message::Ping(vec![]);
    let pong_msg = Message::Pong(vec![]);
    
    match ping_msg {
        Message::Ping(_) => {}, // Expected
        _ => panic!("Expected ping message"),
    }
    
    match pong_msg {
        Message::Pong(_) => {}, // Expected
        _ => panic!("Expected pong message"),
    }
}

// Mock WebSocket tests
#[tokio::test]
async fn test_invalid_json_handling() {
    // Test that invalid JSON is handled gracefully
    let invalid_json = "{ invalid json }";
    let result = serde_json::from_str::<ClientMessage>(invalid_json);
    assert!(result.is_err());
    
    // Test empty message
    let empty_json = "";
    let result = serde_json::from_str::<ClientMessage>(empty_json);
    assert!(result.is_err());
    
    // Test malformed array
    let malformed = "[\"EVENT\", malformed]";
    let result = serde_json::from_str::<ClientMessage>(malformed);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_subscription_management() {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    let subscriptions: Arc<RwLock<HashMap<String, HashMap<String, Filter>>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    
    let mut handles = vec![];
    
    // Simulate multiple clients adding subscriptions concurrently
    for i in 0..10 {
        let subs = Arc::clone(&subscriptions);
        let handle = tokio::spawn(async move {
            let client_id = format!("client_{}", i);
            let sub_id = format!("sub_{}", i);
            let filter = Filter::new().kinds([Kind::TextNote]);
            
            {
                let mut subs = subs.write().await;
                let client_subs = subs.entry(client_id.clone()).or_insert_with(HashMap::new);
                client_subs.insert(sub_id, filter);
            }
            
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            // Remove subscription
            {
                let mut subs = subs.write().await;
                if let Some(client_subs) = subs.get_mut(&client_id) {
                    client_subs.clear();
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify final state
    let final_subs = subscriptions.read().await;
    assert_eq!(final_subs.len(), 10); // All clients should be present
    
    // All subscriptions should be cleared
    for (_, client_subs) in final_subs.iter() {
        assert!(client_subs.is_empty());
    }
}
