use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub relay_name: String,
    pub relay_description: String,
    pub relay_pubkey: Option<String>,
    pub relay_contact: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/pleb_r1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            relay_name: env::var("RELAY_NAME")
                .unwrap_or_else(|_| "Pleb-R1 Relay".to_string()),
            relay_description: env::var("RELAY_DESCRIPTION")
                .unwrap_or_else(|_| "A community-owned Nostr relay".to_string()),
            relay_pubkey: env::var("RELAY_PUBKEY").ok(),
            relay_contact: env::var("RELAY_CONTACT").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_with_defaults() {
        // Clear environment variables to test defaults
        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
        env::remove_var("RELAY_NAME");
        env::remove_var("RELAY_DESCRIPTION");
        env::remove_var("RELAY_PUBKEY");
        env::remove_var("RELAY_CONTACT");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgresql://postgres:password@localhost:5432/pleb_r1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.relay_name, "Pleb.One Relay");
        assert_eq!(config.relay_description, "A community-owned Nostr relay");
        assert_eq!(config.relay_pubkey, None);
        assert_eq!(config.relay_contact, None);
    }

    #[test]
    fn test_config_from_env_with_custom_values() {
        env::set_var("DATABASE_URL", "postgresql://custom:pass@db:5432/test_db");
        env::set_var("PORT", "9090");
        env::set_var("RELAY_NAME", "Test Relay");
        env::set_var("RELAY_DESCRIPTION", "Test relay description");
        env::set_var("RELAY_PUBKEY", "test_pubkey_123");
        env::set_var("RELAY_CONTACT", "test@example.com");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgresql://custom:pass@db:5432/test_db");
        assert_eq!(config.port, 9090);
        assert_eq!(config.relay_name, "Test Relay");
        assert_eq!(config.relay_description, "Test relay description");
        assert_eq!(config.relay_pubkey, Some("test_pubkey_123".to_string()));
        assert_eq!(config.relay_contact, Some("test@example.com".to_string()));

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
        env::remove_var("RELAY_NAME");
        env::remove_var("RELAY_DESCRIPTION");
        env::remove_var("RELAY_PUBKEY");
        env::remove_var("RELAY_CONTACT");
    }

    #[test]
    fn test_config_invalid_port_uses_default() {
        env::set_var("PORT", "invalid_port");

        let config = Config::from_env();

        assert_eq!(config.port, 8080); // Should fall back to default

        env::remove_var("PORT");
    }

    #[test]
    fn test_config_debug_format() {
        let config = Config::from_env();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("port"));
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::from_env();
        let config2 = config1.clone();

        assert_eq!(config1.database_url, config2.database_url);
        assert_eq!(config1.port, config2.port);
        assert_eq!(config1.relay_name, config2.relay_name);
        assert_eq!(config1.relay_description, config2.relay_description);
        assert_eq!(config1.relay_pubkey, config2.relay_pubkey);
        assert_eq!(config1.relay_contact, config2.relay_contact);
    }
}