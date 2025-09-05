use anyhow::Result;
use nostr_types::{Event, Filter, RelayMessage};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub filters: Vec<Filter>,
    pub created_at: Instant,
}

impl Subscription {
    pub fn new(id: String, filters: Vec<Filter>) -> Self {
        Self {
            id,
            filters,
            created_at: Instant::now(),
        }
    }

    pub fn matches_event(&self, event: &Event) -> bool {
        self.filters.iter().any(|filter| filter.matches(event))
    }
}

#[derive(Debug)]
pub struct Connection {
    id: Uuid,
    subscriptions: RwLock<HashMap<String, Subscription>>,
    authenticated: RwLock<bool>,
    pubkey: RwLock<Option<String>>,
    last_activity: RwLock<Instant>,
    message_sender: broadcast::Sender<RelayMessage>,
    _message_receiver: broadcast::Receiver<RelayMessage>,
}

impl Connection {
    pub fn new(id: Uuid) -> Self {
        let (tx, rx) = broadcast::channel(1000);
        
        Self {
            id,
            subscriptions: RwLock::new(HashMap::new()),
            authenticated: RwLock::new(false),
            pubkey: RwLock::new(None),
            last_activity: RwLock::new(Instant::now()),
            message_sender: tx,
            _message_receiver: rx,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn update_last_activity(&self) {
        let mut last_activity = self.last_activity.write().await;
        *last_activity = Instant::now();
    }

    pub async fn last_activity(&self) -> Instant {
        *self.last_activity.read().await
    }

    pub async fn is_authenticated(&self) -> bool {
        *self.authenticated.read().await
    }

    pub async fn set_authenticated(&self, pubkey: Option<String>) {
        let mut authenticated = self.authenticated.write().await;
        let mut stored_pubkey = self.pubkey.write().await;
        
        *authenticated = pubkey.is_some();
        *stored_pubkey = pubkey;
    }

    pub async fn pubkey(&self) -> Option<String> {
        self.pubkey.read().await.clone()
    }

    pub async fn add_subscription(&self, subscription_id: String, filters: Vec<Filter>) {
        let subscription = Subscription::new(subscription_id.clone(), filters);
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);
        
        debug!("📝 Added subscription for connection {}", self.id);
    }

    pub async fn remove_subscription(&self, subscription_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        
        debug!("🗑️ Removed subscription {} for connection {}", subscription_id, self.id);
    }

    pub async fn get_matching_subscriptions(&self, event: &Event) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.matches_event(event))
            .map(|sub| sub.id.clone())
            .collect()
    }

    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    pub async fn send_message(&self, message: RelayMessage) -> Result<()> {
        match self.message_sender.send(message) {
            Ok(_) => Ok(()),
            Err(broadcast::error::SendError(_)) => {
                // No receivers, connection might be closed
                Ok(())
            }
        }
    }

    pub async fn subscribe_to_messages(&self) -> broadcast::Receiver<RelayMessage> {
        self.message_sender.subscribe()
    }

    pub async fn send_event_to_subscriptions(&self, event: &Event) -> Result<()> {
        let matching_subs = self.get_matching_subscriptions(event).await;
        
        for sub_id in matching_subs {
            let message = RelayMessage::Event {
                subscription_id: sub_id,
                event: event.clone(),
            };
            self.send_message(message).await?;
        }
        
        Ok(())
    }
}

pub struct ConnectionManager {
    connections: RwLock<HashMap<Uuid, Arc<Connection>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_connection(&self, connection: Arc<Connection>) {
        let mut connections = self.connections.write().await;
        let id = connection.id();
        connections.insert(id, connection);
        
        info!("➕ Connection {} added (total: {})", id, connections.len());
    }

    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
        
        info!("➖ Connection {} removed (total: {})", id, connections.len());
    }

    pub async fn get_connection(&self, id: Uuid) -> Option<Arc<Connection>> {
        let connections = self.connections.read().await;
        connections.get(&id).cloned()
    }

    pub async fn get_all_connections(&self) -> Vec<Arc<Connection>> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn broadcast_event(&self, event: &Event) {
        let connections = self.get_all_connections().await;
        let mut successful_broadcasts = 0;
        let mut failed_broadcasts = 0;

        for connection in connections {
            match connection.send_event_to_subscriptions(event).await {
                Ok(_) => {
                    let matching_subs = connection.get_matching_subscriptions(event).await;
                    if !matching_subs.is_empty() {
                        successful_broadcasts += 1;
                        debug!("📡 Broadcasted event {} to {} subscriptions on connection {}", 
                               event.id, matching_subs.len(), connection.id());
                    }
                }
                Err(e) => {
                    failed_broadcasts += 1;
                    warn!("❌ Failed to broadcast event {} to connection {}: {}", 
                          event.id, connection.id(), e);
                }
            }
        }

        if successful_broadcasts > 0 || failed_broadcasts > 0 {
            info!("📡 Event {} broadcast complete: {} successful, {} failed", 
                  event.id, successful_broadcasts, failed_broadcasts);
        }
    }

    pub async fn cleanup_inactive_connections(&self, timeout_secs: u64) {
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        let now = Instant::now();
        let mut to_remove = Vec::new();

        {
            let connections = self.connections.read().await;
            for (id, connection) in connections.iter() {
                let last_activity = connection.last_activity().await;
                if now.duration_since(last_activity) > timeout_duration {
                    to_remove.push(*id);
                }
            }
        }

        if !to_remove.is_empty() {
            let mut connections = self.connections.write().await;
            for id in to_remove {
                connections.remove(&id);
                warn!("🧹 Removed inactive connection: {}", id);
            }
        }
    }

    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        let mut total_subscriptions = 0;
        let mut authenticated_connections = 0;

        for connection in connections.values() {
            total_subscriptions += connection.subscription_count().await;
            if connection.is_authenticated().await {
                authenticated_connections += 1;
            }
        }

        ConnectionStats {
            total_connections,
            authenticated_connections,
            total_subscriptions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub authenticated_connections: usize,
    pub total_subscriptions: usize,
}

// Cleanup task for inactive connections
pub async fn start_connection_cleanup_task(
    connection_manager: Arc<ConnectionManager>,
    cleanup_interval_secs: u64,
    timeout_secs: u64,
) {
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(cleanup_interval_secs)
    );

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            connection_manager.cleanup_inactive_connections(timeout_secs).await;
        }
    });

    info!("🧹 Connection cleanup task started (interval: {}s, timeout: {}s)", 
          cleanup_interval_secs, timeout_secs);
}
