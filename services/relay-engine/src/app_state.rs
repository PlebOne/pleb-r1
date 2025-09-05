use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use nostr::Filter;

use crate::{
    config::Config,
    database::PostgresDatabase,
    metrics::Metrics,
    rate_limiter::RateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub database: PostgresDatabase,
    pub subscriptions: Arc<RwLock<HashMap<String, HashMap<String, Filter>>>>,
    pub rate_limiter: RateLimiter,
    pub metrics: Metrics,
    pub config: Config,
}
