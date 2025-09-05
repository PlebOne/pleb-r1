use anyhow::{Result, anyhow};
use nostr_types::{Event, EventKind};
use pleb_one_storage::Storage;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};

use crate::connection::Connection;
use crate::rate_limiter::RateLimiter;

pub struct EventHandler {
    storage: Arc<Storage>,
    rate_limiter: RateLimiter,
}

impl EventHandler {
    pub async fn new(storage: Arc<Storage>) -> Result<Self> {
        let rate_limiter = RateLimiter::new(100, 60); // 100 events per minute per client
        
        Ok(Self {
            storage,
            rate_limiter,
        })
    }

    pub async fn process_event(
        &self,
        event: Event,
        connection: &Arc<Connection>,
    ) -> Result<bool> {
        let client_id = connection.id().to_string();
        
        // Check rate limits
        if !self.rate_limiter.check_rate_limit(&client_id).await {
            warn!("🚫 Rate limit exceeded for client {}", client_id);
            return Ok(false);
        }

        // Validate event structure and signature
        if !self.validate_event(&event).await? {
            warn!("❌ Event validation failed: {}", event.id);
            return Ok(false);
        }

        // Check for duplicates
        if self.storage.event_exists(&event.id).await? {
            debug!("🔄 Duplicate event rejected: {}", event.id);
            return Ok(false);
        }

        // Additional validation based on event kind
        if !self.validate_event_kind(&event, connection).await? {
            warn!("🚫 Event kind validation failed: {} (kind {})", event.id, event.kind);
            return Ok(false);
        }

        // Store the event
        match self.storage.store_event(&event).await {
            Ok(_) => {
                info!("✅ Event stored successfully: {} (kind: {})", event.id, event.kind);
                
                // Record rate limit usage
                self.rate_limiter.record_event(&client_id).await;
                
                Ok(true)
            }
            Err(e) => {
                error!("💥 Failed to store event {}: {}", event.id, e);
                Err(e)
            }
        }
    }

    pub async fn process_auth(
        &self,
        auth_event: Event,
        connection: &Arc<Connection>,
    ) -> Result<bool> {
        // Validate auth event structure
        if auth_event.kind != EventKind::Auth as u64 {
            return Err(anyhow!("Invalid auth event kind: {}", auth_event.kind));
        }

        // Validate signature
        if !self.validate_event_signature(&auth_event).await? {
            warn!("🔐 Auth event signature validation failed");
            return Ok(false);
        }

        // Extract challenge from tags
        let challenge = self.extract_auth_challenge(&auth_event)?;
        
        // Validate challenge (implement your challenge validation logic)
        if !self.validate_auth_challenge(&challenge).await? {
            warn!("🔐 Auth challenge validation failed");
            return Ok(false);
        }

        // Set connection as authenticated
        connection.set_authenticated(Some(auth_event.pubkey.clone())).await;
        
        info!("🔐 Successfully authenticated connection {} with pubkey {}", 
              connection.id(), auth_event.pubkey);
        
        Ok(true)
    }

    async fn validate_event(&self, event: &Event) -> Result<bool> {
        // Basic structure validation
        if event.id.is_empty() {
            return Ok(false);
        }

        if event.pubkey.is_empty() {
            return Ok(false);
        }

        if event.sig.is_empty() {
            return Ok(false);
        }

        // Timestamp validation (not too far in the future or past)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        
        let max_future = 60 * 10; // 10 minutes
        let max_past = 60 * 60 * 24 * 7; // 1 week
        
        if event.created_at > now + max_future {
            warn!("📅 Event too far in future: {}", event.id);
            return Ok(false);
        }
        
        if event.created_at < now - max_past {
            warn!("📅 Event too far in past: {}", event.id);
            return Ok(false);
        }

        // Signature validation
        self.validate_event_signature(event).await
    }

    async fn validate_event_signature(&self, event: &Event) -> Result<bool> {
        // Validate the event signature using nostr cryptographic verification
        match event.verify_signature() {
            Ok(valid) => {
                if !valid {
                    warn!("🔐 Invalid signature for event: {}", event.id);
                }
                Ok(valid)
            }
            Err(e) => {
                error!("💥 Signature verification error for event {}: {}", event.id, e);
                Ok(false)
            }
        }
    }

    async fn validate_event_kind(
        &self,
        event: &Event,
        connection: &Arc<Connection>,
    ) -> Result<bool> {
        match event.kind {
            // Metadata events (kind 0)
            0 => {
                // Basic metadata validation
                if event.content.len() > 10000 {
                    warn!("📝 Metadata content too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Text note events (kind 1)
            1 => {
                // Basic text note validation
                if event.content.len() > 50000 {
                    warn!("📝 Text note content too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Recommend server (kind 2)
            2 => Ok(true),
            
            // Contact list (kind 3)
            3 => {
                // Validate contact list structure
                if event.content.len() > 100000 {
                    warn!("📝 Contact list too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Encrypted direct message (kind 4)
            4 => {
                // Only authenticated users can send DMs
                if !connection.is_authenticated().await {
                    warn!("🔐 Unauthenticated user attempted to send DM: {}", event.id);
                    return Ok(false);
                }
                
                if event.content.len() > 10000 {
                    warn!("📝 DM content too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Event deletion (kind 5)
            5 => {
                // Only authenticated users can delete events
                if !connection.is_authenticated().await {
                    warn!("🔐 Unauthenticated user attempted event deletion: {}", event.id);
                    return Ok(false);
                }
                
                // Verify user owns the events they're trying to delete
                if let Some(user_pubkey) = connection.pubkey().await {
                    if event.pubkey != user_pubkey {
                        warn!("🚫 User attempted to delete event they don't own: {}", event.id);
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            // Reaction (kind 7)
            7 => {
                if event.content.len() > 100 {
                    warn!("📝 Reaction content too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Channel creation/update (kinds 40-42)
            40..=42 => {
                if event.content.len() > 50000 {
                    warn!("📝 Channel content too large for event: {}", event.id);
                    return Ok(false);
                }
                Ok(true)
            }
            
            // Regular replaceable events (kinds 10000-19999)
            10000..=19999 => Ok(true),
            
            // Ephemeral events (kinds 20000-29999)
            20000..=29999 => {
                debug!("⚡ Processing ephemeral event: {}", event.id);
                Ok(true)
            }
            
            // Parameterized replaceable events (kinds 30000-39999)
            30000..=39999 => Ok(true),
            
            // Other kinds - allow for now but log
            _ => {
                debug!("❓ Unknown event kind {}: {}", event.kind, event.id);
                Ok(true)
            }
        }
    }

    fn extract_auth_challenge(&self, auth_event: &Event) -> Result<String> {
        for tag in &auth_event.tags {
            if tag.len() >= 2 && tag[0] == "challenge" {
                return Ok(tag[1].clone());
            }
        }
        Err(anyhow!("No challenge found in auth event"))
    }

    async fn validate_auth_challenge(&self, challenge: &str) -> Result<bool> {
        // Implement your challenge validation logic here
        // For now, we'll accept any non-empty challenge
        Ok(!challenge.is_empty())
    }

    pub async fn get_event_stats(&self) -> Result<EventStats> {
        // Get stats from storage
        let total_events = self.storage.count_all_events().await?;
        let events_today = self.storage.count_events_since(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64 - 86400
        ).await?;
        
        Ok(EventStats {
            total_events,
            events_today,
            rate_limited_clients: self.rate_limiter.get_rate_limited_count().await,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EventStats {
    pub total_events: u64,
    pub events_today: u64,
    pub rate_limited_clients: usize,
}

// Content filtering utilities
pub struct ContentFilter;

impl ContentFilter {
    pub fn contains_spam_indicators(content: &str) -> bool {
        let spam_keywords = [
            "buy now", "click here", "limited time", "act fast",
            "free money", "guaranteed", "no risk", "instant",
        ];
        
        let content_lower = content.to_lowercase();
        spam_keywords.iter().any(|&keyword| content_lower.contains(keyword))
    }
    
    pub fn contains_inappropriate_content(content: &str) -> bool {
        // Implement content moderation logic here
        // This is a basic example - you might want to use external services
        let inappropriate_words = [
            // Add inappropriate words here
        ];
        
        let content_lower = content.to_lowercase();
        inappropriate_words.iter().any(|&word| content_lower.contains(word))
    }
    
    pub fn extract_mentions(content: &str) -> Vec<String> {
        // Extract @mentions from content
        let mention_regex = regex::Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
        mention_regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
    
    pub fn extract_hashtags(content: &str) -> Vec<String> {
        // Extract #hashtags from content
        let hashtag_regex = regex::Regex::new(r"#([a-zA-Z0-9_]+)").unwrap();
        hashtag_regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
}
