use anyhow::Result;
use nostr::Event;
use sqlx::{Pool, Sqlite, Row};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct MockDatabase {
    // In-memory storage for development
    events: Arc<RwLock<Vec<Event>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_event(&self, event: Event) -> Result<()> {
        let mut events = self.events.write().await;
        events.push(event);
        Ok(())
    }

    pub async fn get_events(&self, _filters: &[nostr::Filter]) -> Result<Vec<Event>> {
        let events = self.events.read().await;
        // Return last 10 events for demo
        Ok(events.iter().rev().take(10).cloned().collect())
    }

    pub async fn get_event_count(&self) -> Result<i64> {
        let events = self.events.read().await;
        Ok(events.len() as i64)
    }
}
