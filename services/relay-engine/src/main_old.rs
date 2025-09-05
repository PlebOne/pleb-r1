mod websocket;
mod connection;
mod subscription;
mod event_handler;
mod rate_limiter;
mod metrics;
mod auth;
mod health;

use anyhow::Result;
use clap::Parser;
use pleb_one_config::{load_config_for_env, Environment};
use pleb_one_storage::Storage;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};

#[derive(Parser, Debug)]
#[command(name = "pleb-one-relay")]
#[command(about = "Pleb.One Nostr Relay Engine")]
struct Args {
    /// Environment to run in
    #[arg(short, long, default_value = "development")]
    env: String,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
    
    /// Port to listen on (overrides config)
    #[arg(short, long)]
    port: Option<u16>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    init_logging(args.verbose);
    
    info!("Starting Pleb.One Relay Engine");
    
    // Load configuration
    let env = Environment::from_str(&args.env)?;
    let mut config = load_config_for_env(env)?;
    
    // Override port if provided
    if let Some(port) = args.port {
        config.relay.port = port;
    }
    
    info!("Loaded configuration for environment: {}", env.as_str());
    
    // Initialize storage
    let storage = Arc::new(
        Storage::new(config.database.clone(), config.cache.clone()).await?
    );
    
    // Run migrations
    info!("Running database migrations...");
    storage.migrate().await?;
    
    // Health check
    let health = storage.health_check().await?;
    info!("Storage health check: database={}, cache={}", 
          health.database.connected, health.cache.connected);
    
    // Initialize relay server
    let relay = websocket::RelayServer::new(config.clone(), storage.clone()).await?;
    
    // Start metrics server
    let metrics_server = metrics::start_metrics_server(config.metrics.clone()).await?;
    
    // Start health check server  
    let health_server = health::start_health_server(storage.clone(), config.relay.health_port).await?;
    
    info!("Relay listening on port {}", config.relay.port);
    info!("Metrics server on port {}", config.metrics.port);
    info!("Health server on port {}", config.relay.health_port);
    
    // Start the relay server
    let relay_handle = tokio::spawn(async move {
        if let Err(e) = relay.run().await {
            error!("Relay server error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal, gracefully shutting down...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    
    // Graceful shutdown
    relay_handle.abort();
    
    if let Err(e) = metrics_server.shutdown().await {
        warn!("Error shutting down metrics server: {}", e);
    }
    
    if let Err(e) = health_server.shutdown().await {
        warn!("Error shutting down health server: {}", e);
    }
    
    info!("Pleb.One Relay Engine shutdown complete");
    
    Ok(())
}
    
    info!("🚀 Starting Pleb.One relay on port {}", server.port());
    info!("Pleb.One Relay Engine shutdown complete");
    
    Ok(())
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    
    let log_level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(log_level))
        )
        .init();
}