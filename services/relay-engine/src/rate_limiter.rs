use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub events_per_minute: u32,
    pub queries_per_minute: u32,
    pub connections_per_ip: u32,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            events_per_minute: 60,
            queries_per_minute: 120,
            connections_per_ip: 10,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[derive(Debug)]
struct RateLimitEntry {
    events: Vec<Instant>,
    queries: Vec<Instant>,
    connections: u32,
    last_cleanup: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            queries: Vec::new(),
            connections: 0,
            last_cleanup: Instant::now(),
        }
    }

    fn cleanup_old_entries(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.events.retain(|&time| time > cutoff);
        self.queries.retain(|&time| time > cutoff);
        self.last_cleanup = Instant::now();
    }

    fn should_cleanup(&self, cleanup_interval: Duration) -> bool {
        Instant::now() - self.last_cleanup > cleanup_interval
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    entries: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let entries = Arc::new(RwLock::new(HashMap::new()));
        
        // Start cleanup task
        let cleanup_entries = Arc::clone(&entries);
        let cleanup_config = config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_config.cleanup_interval);
            loop {
                interval.tick().await;
                Self::cleanup_task(&cleanup_entries, &cleanup_config).await;
            }
        });

        Self { config, entries }
    }

    async fn cleanup_task(
        entries: &Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
        _config: &RateLimitConfig,
    ) {
        let mut entries_guard = entries.write().await;
        let window = Duration::from_secs(60);
        
        // Clean up old entries and remove empty ones
        entries_guard.retain(|_ip, entry| {
            entry.cleanup_old_entries(window);
            
            // Keep entry if it has recent activity or active connections
            !entry.events.is_empty() 
                || !entry.queries.is_empty() 
                || entry.connections > 0
        });
        
        debug!("Rate limiter cleanup completed. Active IPs: {}", entries_guard.len());
    }

    pub async fn check_event_rate(&self, ip: IpAddr) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(ip).or_insert_with(RateLimitEntry::new);

        // Cleanup if needed
        if entry.should_cleanup(self.config.cleanup_interval) {
            entry.cleanup_old_entries(Duration::from_secs(60));
        }

        // Check rate limit
        if entry.events.len() >= self.config.events_per_minute as usize {
            warn!("Event rate limit exceeded for IP: {}", ip);
            return Ok(false);
        }

        // Add current request
        entry.events.push(Instant::now());
        debug!("Event recorded for IP: {}. Count: {}", ip, entry.events.len());
        Ok(true)
    }

    pub async fn check_query_rate(&self, ip: IpAddr) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(ip).or_insert_with(RateLimitEntry::new);

        // Cleanup if needed
        if entry.should_cleanup(self.config.cleanup_interval) {
            entry.cleanup_old_entries(Duration::from_secs(60));
        }

        // Check rate limit
        if entry.queries.len() >= self.config.queries_per_minute as usize {
            warn!("Query rate limit exceeded for IP: {}", ip);
            return Ok(false);
        }

        // Add current request
        entry.queries.push(Instant::now());
        debug!("Query recorded for IP: {}. Count: {}", ip, entry.queries.len());
        Ok(true)
    }

    pub async fn check_connection_limit(&self, ip: IpAddr) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(ip).or_insert_with(RateLimitEntry::new);

        if entry.connections >= self.config.connections_per_ip {
            warn!("Connection limit exceeded for IP: {}. Current: {}", ip, entry.connections);
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn add_connection(&self, ip: IpAddr) -> Result<()> {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(ip).or_insert_with(RateLimitEntry::new);
        entry.connections += 1;
        debug!("Connection added for IP: {}. Total: {}", ip, entry.connections);
        Ok(())
    }

    pub async fn remove_connection(&self, ip: IpAddr) -> Result<()> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(&ip) {
            if entry.connections > 0 {
                entry.connections -= 1;
                debug!("Connection removed for IP: {}. Remaining: {}", ip, entry.connections);
            }
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<RateLimitStats> {
        let entries = self.entries.read().await;
        let mut total_connections = 0;
        let mut total_active_ips = 0;
        let mut max_connections_per_ip = 0;

        for entry in entries.values() {
            total_connections += entry.connections;
            if entry.connections > 0 || !entry.events.is_empty() || !entry.queries.is_empty() {
                total_active_ips += 1;
            }
            if entry.connections > max_connections_per_ip {
                max_connections_per_ip = entry.connections;
            }
        }

        Ok(RateLimitStats {
            total_connections,
            total_active_ips,
            max_connections_per_ip,
            tracked_ips: entries.len(),
        })
    }
}

#[derive(Debug)]
pub struct RateLimitStats {
    pub total_connections: u32,
    pub total_active_ips: usize,
    pub max_connections_per_ip: u32,
    pub tracked_ips: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    fn test_ip() -> IpAddr {
        IpAddr::from_str("127.0.0.1").unwrap()
    }

    fn test_ip2() -> IpAddr {
        IpAddr::from_str("192.168.1.1").unwrap()
    }

    #[tokio::test]
    async fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        
        assert_eq!(config.events_per_minute, 60);
        assert_eq!(config.queries_per_minute, 120);
        assert_eq!(config.connections_per_ip, 10);
        assert_eq!(config.cleanup_interval, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_rate_limiter_new() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        
        // Test that the limiter is created successfully
        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.total_active_ips, 0);
    }

    #[tokio::test]
    async fn test_event_rate_limiting() {
        let config = RateLimitConfig {
            events_per_minute: 3,
            queries_per_minute: 120,
            connections_per_ip: 10,
            cleanup_interval: Duration::from_secs(300),
        };
        let limiter = RateLimiter::new(config);
        let ip = test_ip();

        // First 3 events should be allowed
        assert!(limiter.check_event_rate(ip).await.unwrap());
        assert!(limiter.check_event_rate(ip).await.unwrap());
        assert!(limiter.check_event_rate(ip).await.unwrap());

        // 4th event should be rate limited
        assert!(!limiter.check_event_rate(ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_query_rate_limiting() {
        let config = RateLimitConfig {
            events_per_minute: 60,
            queries_per_minute: 2,
            connections_per_ip: 10,
            cleanup_interval: Duration::from_secs(300),
        };
        let limiter = RateLimiter::new(config);
        let ip = test_ip();

        // First 2 queries should be allowed
        assert!(limiter.check_query_rate(ip).await.unwrap());
        assert!(limiter.check_query_rate(ip).await.unwrap());

        // 3rd query should be rate limited
        assert!(!limiter.check_query_rate(ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_connection_limiting() {
        let config = RateLimitConfig {
            events_per_minute: 60,
            queries_per_minute: 120,
            connections_per_ip: 2,
            cleanup_interval: Duration::from_secs(300),
        };
        let limiter = RateLimiter::new(config);
        let ip = test_ip();

        // First 2 connections should be allowed
        assert!(limiter.check_connection_limit(ip).await.unwrap());
        limiter.add_connection(ip).await.unwrap();
        
        assert!(limiter.check_connection_limit(ip).await.unwrap());
        limiter.add_connection(ip).await.unwrap();

        // 3rd connection should be rejected
        assert!(!limiter.check_connection_limit(ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_connection_management() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let ip = test_ip();

        // Add connections
        limiter.add_connection(ip).await.unwrap();
        limiter.add_connection(ip).await.unwrap();

        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 2);

        // Remove connection
        limiter.remove_connection(ip).await.unwrap();

        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 1);

        // Remove another connection
        limiter.remove_connection(ip).await.unwrap();

        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 0);

        // Removing from zero should not underflow
        limiter.remove_connection(ip).await.unwrap();

        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 0);
    }

    #[tokio::test]
    async fn test_multiple_ips_independent_limits() {
        let config = RateLimitConfig {
            events_per_minute: 2,
            queries_per_minute: 120,
            connections_per_ip: 10,
            cleanup_interval: Duration::from_secs(300),
        };
        let limiter = RateLimiter::new(config);
        let ip1 = test_ip();
        let ip2 = test_ip2();

        // IP1 uses its limit
        assert!(limiter.check_event_rate(ip1).await.unwrap());
        assert!(limiter.check_event_rate(ip1).await.unwrap());
        assert!(!limiter.check_event_rate(ip1).await.unwrap()); // Rate limited

        // IP2 should still have its full limit
        assert!(limiter.check_event_rate(ip2).await.unwrap());
        assert!(limiter.check_event_rate(ip2).await.unwrap());
        assert!(!limiter.check_event_rate(ip2).await.unwrap()); // Rate limited
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let ip1 = test_ip();
        let ip2 = test_ip2();

        // Add some activity
        limiter.add_connection(ip1).await.unwrap();
        limiter.add_connection(ip2).await.unwrap();
        limiter.check_event_rate(ip1).await.unwrap();
        limiter.check_query_rate(ip2).await.unwrap();

        let stats = limiter.get_stats().await.unwrap();
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.total_active_ips, 2);
        assert_eq!(stats.max_connections_per_ip, 1);
        assert_eq!(stats.tracked_ips, 2);
    }

    #[tokio::test]
    async fn test_rate_limit_entry_cleanup() {
        let mut entry = RateLimitEntry::new();
        let old_time = Instant::now() - Duration::from_secs(120);
        let recent_time = Instant::now();

        // Add some old and recent entries
        entry.events.push(old_time);
        entry.events.push(recent_time);
        entry.queries.push(old_time);
        entry.queries.push(recent_time);

        // Cleanup should remove old entries
        entry.cleanup_old_entries(Duration::from_secs(60));

        assert_eq!(entry.events.len(), 1);
        assert_eq!(entry.queries.len(), 1);
    }

    #[tokio::test]
    async fn test_rate_limit_entry_should_cleanup() {
        let mut entry = RateLimitEntry::new();
        
        // Just created, should not need cleanup
        assert!(!entry.should_cleanup(Duration::from_secs(300)));

        // Simulate time passing
        entry.last_cleanup = Instant::now() - Duration::from_secs(400);
        assert!(entry.should_cleanup(Duration::from_secs(300)));
    }
}
