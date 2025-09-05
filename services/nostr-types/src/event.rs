use crate::crypto::{PublicKey, Signature, sha256_hash};
use crate::error::NostrError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(String);

impl EventId {
    pub fn new(hex: String) -> Result<Self, NostrError> {
        if hex.len() != 64 {
            return Err(NostrError::InvalidEvent(
                "Event ID must be 64 hex characters".to_string()
            ));
        }
        
        hex::decode(&hex).map_err(|_| {
            NostrError::InvalidEvent("Invalid hex encoding for event ID".to_string())
        })?;
        
        Ok(EventId(hex))
    }
    
    pub fn as_hex(&self) -> &str {
        &self.0
    }
    
    pub fn from_event(event: &Event) -> Self {
        let serialized = event.to_canonical_json();
        let hash = sha256_hash(serialized.as_bytes());
        EventId(hex::encode(hash))
    }
}

/// Tag in a Nostr event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag(Vec<String>);

impl Tag {
    pub fn new(values: Vec<String>) -> Self {
        Tag(values)
    }
    
    pub fn tag_name(&self) -> Option<&str> {
        self.0.first().map(|s| s.as_str())
    }
    
    pub fn values(&self) -> &[String] {
        &self.0
    }
    
    pub fn get(&self, index: usize) -> Option<&str> {
        self.0.get(index).map(|s| s.as_str())
    }
}

/// Core Nostr event structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub pubkey: PublicKey,
    pub created_at: i64,
    pub kind: u64,
    pub tags: Vec<Tag>,
    pub content: String,
    pub sig: Signature,
}

impl Event {
    /// Create canonical JSON representation for ID calculation
    pub fn to_canonical_json(&self) -> String {
        let array = serde_json::json!([
            0,
            self.pubkey.as_hex(),
            self.created_at,
            self.kind,
            self.tags.iter().map(|tag| tag.values()).collect::<Vec<_>>(),
            self.content
        ]);
        array.to_string()
    }
    
    /// Verify the event's signature
    pub fn verify_signature(&self) -> Result<bool, NostrError> {
        let canonical = self.to_canonical_json();
        let hash = sha256_hash(canonical.as_bytes());
        crate::crypto::verify_signature(&hash, &self.pubkey, &self.sig)
    }
    
    /// Verify the event's ID matches its content
    pub fn verify_id(&self) -> bool {
        let calculated_id = EventId::from_event(self);
        calculated_id == self.id
    }
    
    /// Get timestamp as DateTime<Utc>
    pub fn created_at_utc(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.created_at, 0)
    }
    
    /// Check if event is ephemeral (kind 20000-29999)
    pub fn is_ephemeral(&self) -> bool {
        (20000..=29999).contains(&self.kind)
    }
    
    /// Check if event is replaceable (kind 10000-19999)
    pub fn is_replaceable(&self) -> bool {
        (10000..=19999).contains(&self.kind)
    }
    
    /// Check if event is parameterized replaceable (kind 30000-39999)
    pub fn is_parameterized_replaceable(&self) -> bool {
        (30000..=39999).contains(&self.kind)
    }
    
    /// Get the 'd' tag value for parameterized replaceable events
    pub fn d_tag(&self) -> Option<&str> {
        self.tags
            .iter()
            .find(|tag| tag.tag_name() == Some("d"))
            .and_then(|tag| tag.get(1))
    }
    
    /// Get all referenced event IDs from 'e' tags
    pub fn referenced_events(&self) -> Vec<&str> {
        self.tags
            .iter()
            .filter_map(|tag| {
                if tag.tag_name() == Some("e") {
                    tag.get(1)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all referenced pubkeys from 'p' tags
    pub fn referenced_pubkeys(&self) -> Vec<&str> {
        self.tags
            .iter()
            .filter_map(|tag| {
                if tag.tag_name() == Some("p") {
                    tag.get(1)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Builder for creating events
pub struct EventBuilder {
    pubkey: Option<PublicKey>,
    created_at: Option<i64>,
    kind: Option<u64>,
    tags: Vec<Tag>,
    content: String,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self {
            pubkey: None,
            created_at: None,
            kind: None,
            tags: Vec::new(),
            content: String::new(),
        }
    }
    
    pub fn pubkey(mut self, pubkey: PublicKey) -> Self {
        self.pubkey = Some(pubkey);
        self
    }
    
    pub fn created_at(mut self, timestamp: i64) -> Self {
        self.created_at = Some(timestamp);
        self
    }
    
    pub fn kind(mut self, kind: u64) -> Self {
        self.kind = Some(kind);
        self
    }
    
    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = content.into();
        self
    }
    
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }
    
    pub fn add_tag<T: Into<String>>(mut self, tag_name: T, values: Vec<String>) -> Self {
        let mut tag_values = vec![tag_name.into()];
        tag_values.extend(values);
        self.tags.push(Tag::new(tag_values));
        self
    }
    
    /// Build the event (requires external signing)
    pub fn build_unsigned(self) -> Result<UnsignedEvent, NostrError> {
        let pubkey = self.pubkey.ok_or_else(|| {
            NostrError::InvalidEvent("Missing pubkey".to_string())
        })?;
        
        let kind = self.kind.ok_or_else(|| {
            NostrError::InvalidEvent("Missing kind".to_string())
        })?;
        
        let created_at = self.created_at.unwrap_or_else(|| {
            Utc::now().timestamp()
        });
        
        Ok(UnsignedEvent {
            pubkey,
            created_at,
            kind,
            tags: self.tags,
            content: self.content,
        })
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Unsigned event that needs to be signed
#[derive(Debug, Clone)]
pub struct UnsignedEvent {
    pub pubkey: PublicKey,
    pub created_at: i64,
    pub kind: u64,
    pub tags: Vec<Tag>,
    pub content: String,
}

impl UnsignedEvent {
    /// Convert to canonical JSON for signing
    pub fn to_canonical_json(&self) -> String {
        let array = serde_json::json!([
            0,
            self.pubkey.as_hex(),
            self.created_at,
            self.kind,
            self.tags.iter().map(|tag| tag.values()).collect::<Vec<_>>(),
            self.content
        ]);
        array.to_string()
    }
    
    /// Calculate the event ID
    pub fn id(&self) -> EventId {
        let canonical = self.to_canonical_json();
        let hash = sha256_hash(canonical.as_bytes());
        EventId(hex::encode(hash))
    }
    
    /// Sign the event (signature implementation would be external)
    pub fn sign(self, signature: Signature) -> Event {
        let id = self.id();
        Event {
            id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_builder() {
        let pubkey = PublicKey::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        
        let unsigned = EventBuilder::new()
            .pubkey(pubkey.clone())
            .kind(1)
            .content("Hello Nostr!")
            .created_at(1672531200)
            .build_unsigned()
            .unwrap();
        
        assert_eq!(unsigned.pubkey, pubkey);
        assert_eq!(unsigned.kind, 1);
        assert_eq!(unsigned.content, "Hello Nostr!");
        assert_eq!(unsigned.created_at, 1672531200);
    }
    
    #[test]
    fn test_event_id_calculation() {
        let pubkey = PublicKey::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        
        let unsigned = EventBuilder::new()
            .pubkey(pubkey)
            .kind(1)
            .content("Hello Nostr!")
            .created_at(1672531200)
            .build_unsigned()
            .unwrap();
        
        let id = unsigned.id();
        assert_eq!(id.as_hex().len(), 64);
    }
}
