pub mod event;
pub mod filter;
pub mod message;
pub mod error;
pub mod crypto;
pub mod validation;

// Re-export commonly used types
pub use event::{Event, EventId, EventBuilder};
pub use filter::Filter;
pub use message::{ClientMessage, RelayMessage, SubscriptionId};
pub use error::{NostrError, ValidationError};
pub use crypto::{PublicKey, Signature, verify_signature};

/// Nostr protocol constants
pub mod constants {
    /// Maximum size of an event in bytes
    pub const MAX_EVENT_SIZE: usize = 65_536; // 64KB
    
    /// Maximum number of tags per event
    pub const MAX_TAGS_COUNT: usize = 2000;
    
    /// Maximum length of tag values
    pub const MAX_TAG_VALUE_LENGTH: usize = 1024;
    
    /// Maximum content length
    pub const MAX_CONTENT_LENGTH: usize = 65_536;
    
    /// Maximum number of active subscriptions per connection
    pub const MAX_SUBSCRIPTIONS_PER_CONNECTION: usize = 20;
}
