use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub relay: RelayConfig,
    pub limits: LimitsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RelayConfig {
    pub name: String,
    pub description: String,
    pub pubkey: String,
    pub contact: String,
    pub max_connections: usize,
    pub max_subscriptions_per_connection: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitsConfig {
    pub max_event_size: usize,
    pub max_content_length: usize,
    pub rate_limit_per_minute: u32,
    pub max_filters_per_req: usize,
    pub max_limit_per_req: usize,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            port: std::env::var("RELAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            relay: RelayConfig {
                name: std::env::var("RELAY_NAME")
                    .unwrap_or_else(|_| "Pleb.One Relay".to_string()),
                description: std::env::var("RELAY_DESCRIPTION")
                    .unwrap_or_else(|_| "Community-owned Nostr infrastructure".to_string()),
                pubkey: std::env::var("RELAY_PUBKEY")
                    .unwrap_or_else(|_| "".to_string()),
                contact: std::env::var("RELAY_CONTACT")
                    .unwrap_or_else(|_| "admin@pleb.one".to_string()),
                max_connections: std::env::var("MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()?,
                max_subscriptions_per_connection: std::env::var("MAX_SUBS_PER_CONN")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
            },
            limits: LimitsConfig {
                max_event_size: std::env::var("MAX_EVENT_SIZE")
                    .unwrap_or_else(|_| "65536".to_string())
                    .parse()?,
                max_content_length: std::env::var("MAX_CONTENT_LENGTH")
                    .unwrap_or_else(|_| "32768".to_string())
                    .parse()?,
                rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                    .unwrap_or_else(|_| "120".to_string())
                    .parse()?,
                max_filters_per_req: std::env::var("MAX_FILTERS_PER_REQ")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                max_limit_per_req: std::env::var("MAX_LIMIT_PER_REQ")
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()?,
            },
        })
    }
}
