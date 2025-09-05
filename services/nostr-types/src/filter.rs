use crate::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subscription ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        SubscriptionId(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SubscriptionId {
    fn from(id: String) -> Self {
        SubscriptionId(id)
    }
}

impl From<&str> for SubscriptionId {
    fn from(id: &str) -> Self {
        SubscriptionId(id.to_string())
    }
}

/// Filter for event subscriptions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<u64>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    
    #[serde(flatten)]
    pub tags: HashMap<String, Vec<String>>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            ids: None,
            authors: None,
            kinds: None,
            since: None,
            until: None,
            limit: None,
            tags: HashMap::new(),
        }
    }
    
    /// Check if an event matches this filter
    pub fn matches(&self, event: &Event) -> bool {
        // Check IDs
        if let Some(ref ids) = self.ids {
            if !ids.contains(&event.id.as_hex().to_string()) {
                return false;
            }
        }
        
        // Check authors
        if let Some(ref authors) = self.authors {
            if !authors.contains(&event.pubkey.as_hex().to_string()) {
                return false;
            }
        }
        
        // Check kinds
        if let Some(ref kinds) = self.kinds {
            if !kinds.contains(&event.kind) {
                return false;
            }
        }
        
        // Check time range
        if let Some(since) = self.since {
            if event.created_at < since {
                return false;
            }
        }
        
        if let Some(until) = self.until {
            if event.created_at > until {
                return false;
            }
        }
        
        // Check tag filters
        for (tag_name, values) in &self.tags {
            let matches_tag = event.tags.iter().any(|tag| {
                tag.tag_name() == Some(tag_name) && 
                tag.values().iter().skip(1).any(|v| values.contains(v))
            });
            
            if !matches_tag {
                return false;
            }
        }
        
        true
    }
    
    /// Add an ID filter
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.ids.get_or_insert_with(Vec::new).push(id.into());
        self
    }
    
    /// Add multiple ID filters
    pub fn ids<I, S>(mut self, ids: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let id_vec = ids.into_iter().map(|s| s.into()).collect();
        self.ids = Some(id_vec);
        self
    }
    
    /// Add an author filter
    pub fn author<S: Into<String>>(mut self, pubkey: S) -> Self {
        self.authors.get_or_insert_with(Vec::new).push(pubkey.into());
        self
    }
    
    /// Add multiple author filters
    pub fn authors<I, S>(mut self, pubkeys: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let author_vec = pubkeys.into_iter().map(|s| s.into()).collect();
        self.authors = Some(author_vec);
        self
    }
    
    /// Add a kind filter
    pub fn kind(mut self, kind: u64) -> Self {
        self.kinds.get_or_insert_with(Vec::new).push(kind);
        self
    }
    
    /// Add multiple kind filters
    pub fn kinds<I>(mut self, kinds: I) -> Self 
    where
        I: IntoIterator<Item = u64>,
    {
        let kind_vec = kinds.into_iter().collect();
        self.kinds = Some(kind_vec);
        self
    }
    
    /// Set since timestamp
    pub fn since(mut self, timestamp: i64) -> Self {
        self.since = Some(timestamp);
        self
    }
    
    /// Set until timestamp
    pub fn until(mut self, timestamp: i64) -> Self {
        self.until = Some(timestamp);
        self
    }
    
    /// Set limit
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Add a tag filter
    pub fn tag<S1, S2>(mut self, tag_name: S1, value: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.tags
            .entry(format!("#{}", tag_name.into()))
            .or_insert_with(Vec::new)
            .push(value.into());
        self
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

/// Messages sent from client to relay
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "0", rename_all = "UPPERCASE")]
pub enum ClientMessage {
    #[serde(rename = "EVENT")]
    Event {
        #[serde(rename = "1")]
        event: Event,
    },
    #[serde(rename = "REQ")]
    Req {
        #[serde(rename = "1")]
        subscription_id: SubscriptionId,
        #[serde(rename = "2")]
        filters: Vec<Filter>,
    },
    #[serde(rename = "CLOSE")]
    Close {
        #[serde(rename = "1")]
        subscription_id: SubscriptionId,
    },
    #[serde(rename = "AUTH")]
    Auth {
        #[serde(rename = "1")]
        event: Event,
    },
}

/// Messages sent from relay to client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "0", rename_all = "UPPERCASE")]
pub enum RelayMessage {
    #[serde(rename = "EVENT")]
    Event {
        #[serde(rename = "1")]
        subscription_id: SubscriptionId,
        #[serde(rename = "2")]
        event: Event,
    },
    #[serde(rename = "OK")]
    Ok {
        #[serde(rename = "1")]
        event_id: String,
        #[serde(rename = "2")]
        accepted: bool,
        #[serde(rename = "3")]
        message: String,
    },
    #[serde(rename = "EOSE")]
    EndOfStoredEvents {
        #[serde(rename = "1")]
        subscription_id: SubscriptionId,
    },
    #[serde(rename = "CLOSED")]
    Closed {
        #[serde(rename = "1")]
        subscription_id: SubscriptionId,
        #[serde(rename = "2")]
        message: String,
    },
    #[serde(rename = "NOTICE")]
    Notice {
        #[serde(rename = "1")]
        message: String,
    },
    #[serde(rename = "AUTH")]
    Auth {
        #[serde(rename = "1")]
        challenge: String,
    },
}

impl RelayMessage {
    /// Create an OK message for accepted event
    pub fn ok_accepted<S: Into<String>>(event_id: S) -> Self {
        RelayMessage::Ok {
            event_id: event_id.into(),
            accepted: true,
            message: "".to_string(),
        }
    }
    
    /// Create an OK message for rejected event
    pub fn ok_rejected<S1, S2>(event_id: S1, reason: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        RelayMessage::Ok {
            event_id: event_id.into(),
            accepted: false,
            message: reason.into(),
        }
    }
    
    /// Create a notice message
    pub fn notice<S: Into<String>>(message: S) -> Self {
        RelayMessage::Notice {
            message: message.into(),
        }
    }
    
    /// Create an auth challenge
    pub fn auth<S: Into<String>>(challenge: S) -> Self {
        RelayMessage::Auth {
            challenge: challenge.into(),
        }
    }
    
    /// Create an event message
    pub fn event(subscription_id: SubscriptionId, event: Event) -> Self {
        RelayMessage::Event {
            subscription_id,
            event,
        }
    }
    
    /// Create an end of stored events message
    pub fn eose(subscription_id: SubscriptionId) -> Self {
        RelayMessage::EndOfStoredEvents { subscription_id }
    }
    
    /// Create a closed subscription message
    pub fn closed<S: Into<String>>(subscription_id: SubscriptionId, message: S) -> Self {
        RelayMessage::Closed {
            subscription_id,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::PublicKey;
    use crate::event::EventBuilder;
    
    #[test]
    fn test_filter_matching() {
        let pubkey = PublicKey::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        
        let unsigned = EventBuilder::new()
            .pubkey(pubkey.clone())
            .kind(1)
            .content("Hello Nostr!")
            .created_at(1672531200)
            .build_unsigned()
            .unwrap();
        
        // Mock signature for test
        let sig = crate::crypto::Signature::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        let event = unsigned.sign(sig);
        
        // Test kind filter
        let filter = Filter::new().kind(1);
        assert!(filter.matches(&event));
        
        let filter = Filter::new().kind(2);
        assert!(!filter.matches(&event));
        
        // Test author filter
        let filter = Filter::new().author(pubkey.as_hex());
        assert!(filter.matches(&event));
        
        // Test time filter
        let filter = Filter::new().since(1672531100);
        assert!(filter.matches(&event));
        
        let filter = Filter::new().since(1672531300);
        assert!(!filter.matches(&event));
    }
    
    #[test]
    fn test_message_serialization() {
        let subscription_id = SubscriptionId::new("test-sub");
        let filter = Filter::new().kind(1).limit(10);
        
        let req = ClientMessage::Req {
            subscription_id: subscription_id.clone(),
            filters: vec![filter],
        };
        
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("REQ"));
        
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, req);
    }
}
