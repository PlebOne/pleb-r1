pub mod database;
pub mod cache;
pub mod repository;
pub mod error;
pub mod migrations;

// Re-export main types
pub use database::{Database, DatabaseConfig};
pub use cache::{Cache, CacheConfig};
pub use repository::{EventRepository, UserRepository, SubscriptionRepository};
pub use error::{StorageError, StorageResult};

/// Storage layer facade that combines database and cache
pub struct Storage {
    pub database: Database,
    pub cache: Cache,
    pub event_repo: EventRepository,
    pub user_repo: UserRepository,
    pub subscription_repo: SubscriptionRepository,
}

impl Storage {
    pub async fn new(db_config: DatabaseConfig, cache_config: CacheConfig) -> StorageResult<Self> {
        let database = Database::new(db_config).await?;
        let cache = Cache::new(cache_config).await?;
        
        let event_repo = EventRepository::new(database.pool().clone(), cache.client().clone());
        let user_repo = UserRepository::new(database.pool().clone(), cache.client().clone());
        let subscription_repo = SubscriptionRepository::new(cache.client().clone());
        
        Ok(Self {
            database,
            cache,
            event_repo,
            user_repo,
            subscription_repo,
        })
    }
    
    /// Run database migrations
    pub async fn migrate(&self) -> StorageResult<()> {
        self.database.migrate().await
    }
    
    /// Health check for storage layer
    pub async fn health_check(&self) -> StorageResult<StorageHealth> {
        let db_health = self.database.health_check().await?;
        let cache_health = self.cache.health_check().await?;
        
        Ok(StorageHealth {
            database: db_health,
            cache: cache_health,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StorageHealth {
    pub database: DatabaseHealth,
    pub cache: CacheHealth,
}

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub pool_size: u32,
    pub active_connections: u32,
}

#[derive(Debug, Clone)]
pub struct CacheHealth {
    pub connected: bool,
    pub memory_usage: u64,
    pub connected_clients: u32,
}
