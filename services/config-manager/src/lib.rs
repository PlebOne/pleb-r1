pub mod app_config;
pub mod environment;
pub mod loader;
pub mod validation;
pub mod error;

// Re-export main types
pub use app_config::{AppConfig, RelayConfig, DatabaseConfig, CacheConfig, AuthConfig, MetricsConfig};
pub use environment::Environment;
pub use loader::ConfigLoader;
pub use error::{ConfigError, ConfigResult};

/// Load configuration from multiple sources with environment override
pub fn load_config() -> ConfigResult<AppConfig> {
    ConfigLoader::new()
        .add_file("config/default.toml")?
        .add_env_file()?
        .add_environment_variables()?
        .load()
}

/// Load configuration for specific environment
pub fn load_config_for_env(env: Environment) -> ConfigResult<AppConfig> {
    ConfigLoader::new()
        .add_file("config/default.toml")?
        .add_file(&format!("config/{}.toml", env.as_str()))?
        .add_env_file()?
        .add_environment_variables()?
        .load()
}
