use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrError {
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Event too large: {size} bytes (max: {max})")]
    EventTooLarge { size: usize, max: usize },
    
    #[error("Too many tags: {count} (max: {max})")]
    TooManyTags { count: usize, max: usize },
    
    #[error("Tag value too long: {length} chars (max: {max})")]
    TagValueTooLong { length: usize, max: usize },
    
    #[error("Content too long: {length} chars (max: {max})")]
    ContentTooLong { length: usize, max: usize },
    
    #[error("Invalid kind: {kind}")]
    InvalidKind(u64),
    
    #[error("Missing required field: {field}")]
    MissingField(String),
    
    #[error("Invalid field format: {field}")]
    InvalidFieldFormat(String),
}
