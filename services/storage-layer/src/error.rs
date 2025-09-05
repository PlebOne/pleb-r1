use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Event not found: {id}")]
    EventNotFound { id: String },
    
    #[error("User not found: {pubkey}")]
    UserNotFound { pubkey: String },
    
    #[error("Subscription not found: {id}")]
    SubscriptionNotFound { id: String },
    
    #[error("Duplicate event: {id}")]
    DuplicateEvent { id: String },
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation timeout")]
    Timeout,
    
    #[error("Storage capacity exceeded")]
    CapacityExceeded,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl StorageError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, 
            StorageError::EventNotFound { .. } |
            StorageError::UserNotFound { .. } |
            StorageError::SubscriptionNotFound { .. }
        )
    }
    
    pub fn is_duplicate(&self) -> bool {
        matches!(self, StorageError::DuplicateEvent { .. })
    }
    
    pub fn is_temporary(&self) -> bool {
        matches!(self, 
            StorageError::Timeout |
            StorageError::Connection(_)
        )
    }
}
