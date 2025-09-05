use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{error, info, warn};

use crate::{TrafficEvent, ReportQuery, TrafficReport, RealtimeMetrics, ResponseTimeStats};
use config_manager::Config;
use storage_layer::Database;

pub struct AnalyticsEngine {
    db: Database,
    redis: redis::Client,
}

impl AnalyticsEngine {
    pub async fn new(config: &Config) -> Result<Self> {
        let db = Database::new(config).await?;
        let redis = redis::Client::open(config.redis.url.as_str())?;
        
        // Create analytics tables if they don't exist
        Self::init_analytics_tables(&db.pool).await?;
        
        Ok(Self { db, redis })
    }

    async fn init_analytics_tables(pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traffic_events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                event_id VARCHAR NOT NULL,
                client_id VARCHAR,
                event_type VARCHAR NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                metadata JSONB,
                response_time_ms INTEGER,
                bytes_transferred BIGINT,
                error_code VARCHAR,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_traffic_events_timestamp ON traffic_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_traffic_events_type ON traffic_events(event_type);
            CREATE INDEX IF NOT EXISTS idx_traffic_events_client ON traffic_events(client_id);

            -- Convert to hypertable for time-series optimization
            SELECT create_hypertable('traffic_events', 'timestamp', if_not_exists => TRUE);

            CREATE TABLE IF NOT EXISTS connection_metrics (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                active_connections INTEGER NOT NULL,
                peak_connections INTEGER NOT NULL,
                events_per_second REAL NOT NULL,
                subscriptions_count INTEGER NOT NULL,
                memory_usage_bytes BIGINT NOT NULL,
                cpu_usage_percent REAL NOT NULL,
                disk_usage_bytes BIGINT NOT NULL
            );

            SELECT create_hypertable('connection_metrics', 'timestamp', if_not_exists => TRUE);
            "#,
        )
        .execute(pool)
        .await?;

        info!("Analytics tables initialized successfully");
        Ok(())
    }

    pub async fn record_event(&self, event: TrafficEvent) -> Result<()> {
        // Store in PostgreSQL for historical analysis
        sqlx::query(
            r#"
            INSERT INTO traffic_events (event_id, client_id, event_type, timestamp, metadata)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.client_id)
        .bind(&event.event_type)
        .bind(event.timestamp)
        .bind(serde_json::to_value(&event.metadata)?)
        .execute(&self.db.pool)
        .await?;

        // Update real-time counters in Redis
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("events:{}:{}", event.event_type, Utc::now().format("%Y%m%d%H"));
        
        redis::cmd("INCR")
            .arg(&key)
            .query_async::<_, i32>(&mut conn)
            .await?;

        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(86400) // 24 hours
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn generate_report(&self, query: ReportQuery) -> Result<TrafficReport> {
        let start_date = query.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
        let end_date = query.end_date.unwrap_or_else(|| Utc::now());
        let granularity = query.granularity.unwrap_or_else(|| "day".to_string());

        // Get total events
        let total_events = self.get_total_events(start_date, end_date).await?;
        
        // Get unique clients
        let unique_clients = self.get_unique_clients(start_date, end_date).await?;
        
        // Get events by type
        let events_by_type = self.get_events_by_type(start_date, end_date).await?;
        
        // Get peak connections
        let peak_concurrent_connections = self.get_peak_connections(start_date, end_date).await?;
        
        // Get bandwidth usage
        let bandwidth_usage = self.get_bandwidth_usage(start_date, end_date).await?;
        
        // Calculate error rate
        let error_rate = self.calculate_error_rate(start_date, end_date).await?;
        
        // Get response time statistics
        let response_times = self.get_response_time_stats(start_date, end_date).await?;

        Ok(TrafficReport {
            period: format!("{} to {}", start_date.format("%Y-%m-%d"), end_date.format("%Y-%m-%d")),
            total_events,
            unique_clients,
            events_by_type,
            peak_concurrent_connections,
            bandwidth_usage,
            error_rate,
            response_times,
        })
    }

    async fn get_total_events(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM traffic_events WHERE timestamp BETWEEN $1 AND $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;
        
        Ok(row.get::<i64, _>("count") as u64)
    }

    async fn get_unique_clients(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(DISTINCT client_id) as count FROM traffic_events WHERE timestamp BETWEEN $1 AND $2 AND client_id IS NOT NULL"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;
        
        Ok(row.get::<i64, _>("count") as u64)
    }

    async fn get_events_by_type(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<HashMap<String, u64>> {
        let rows = sqlx::query(
            "SELECT event_type, COUNT(*) as count FROM traffic_events WHERE timestamp BETWEEN $1 AND $2 GROUP BY event_type"
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db.pool)
        .await?;

        let mut events_by_type = HashMap::new();
        for row in rows {
            let event_type: String = row.get("event_type");
            let count: i64 = row.get("count");
            events_by_type.insert(event_type, count as u64);
        }

        Ok(events_by_type)
    }

    async fn get_peak_connections(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT MAX(peak_connections) as max_connections FROM connection_metrics WHERE timestamp BETWEEN $1 AND $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;
        
        Ok(row.get::<Option<i32>, _>("max_connections").unwrap_or(0) as u64)
    }

    async fn get_bandwidth_usage(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(bytes_transferred), 0) as total_bytes FROM traffic_events WHERE timestamp BETWEEN $1 AND $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;
        
        Ok(row.get::<Option<i64>, _>("total_bytes").unwrap_or(0) as u64)
    }

    async fn calculate_error_rate(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<f64> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN error_code IS NOT NULL THEN 1 END) as errors
            FROM traffic_events 
            WHERE timestamp BETWEEN $1 AND $2
            "#
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;

        let total: i64 = row.get("total");
        let errors: i64 = row.get("errors");

        if total == 0 {
            Ok(0.0)
        } else {
            Ok((errors as f64 / total as f64) * 100.0)
        }
    }

    async fn get_response_time_stats(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<ResponseTimeStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                AVG(response_time_ms) as avg_ms,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY response_time_ms) as p50_ms,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) as p95_ms,
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms) as p99_ms
            FROM traffic_events 
            WHERE timestamp BETWEEN $1 AND $2 AND response_time_ms IS NOT NULL
            "#
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(ResponseTimeStats {
            average_ms: row.get::<Option<f64>, _>("avg_ms").unwrap_or(0.0),
            p50_ms: row.get::<Option<f64>, _>("p50_ms").unwrap_or(0.0),
            p95_ms: row.get::<Option<f64>, _>("p95_ms").unwrap_or(0.0),
            p99_ms: row.get::<Option<f64>, _>("p99_ms").unwrap_or(0.0),
        })
    }

    pub async fn get_realtime_metrics(&self) -> Result<RealtimeMetrics> {
        // Get latest metrics from connection_metrics table
        let row = sqlx::query(
            r#"
            SELECT * FROM connection_metrics 
            ORDER BY timestamp DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(row) = row {
            Ok(RealtimeMetrics {
                active_connections: row.get::<i32, _>("active_connections") as u64,
                events_per_second: row.get::<f32, _>("events_per_second") as f64,
                subscriptions_count: row.get::<i32, _>("subscriptions_count") as u64,
                memory_usage: row.get::<i64, _>("memory_usage_bytes") as u64,
                cpu_usage: row.get::<f32, _>("cpu_usage_percent") as f64,
                disk_usage: row.get::<i64, _>("disk_usage_bytes") as u64,
            })
        } else {
            // Return default metrics if no data available
            Ok(RealtimeMetrics {
                active_connections: 0,
                events_per_second: 0.0,
                subscriptions_count: 0,
                memory_usage: 0,
                cpu_usage: 0.0,
                disk_usage: 0,
            })
        }
    }

    pub async fn export_csv_report(&self, query: ReportQuery) -> Result<String> {
        let start_date = query.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
        let end_date = query.end_date.unwrap_or_else(|| Utc::now());

        let rows = sqlx::query(
            r#"
            SELECT 
                event_id,
                client_id,
                event_type,
                timestamp,
                response_time_ms,
                bytes_transferred,
                error_code
            FROM traffic_events 
            WHERE timestamp BETWEEN $1 AND $2
            ORDER BY timestamp DESC
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.db.pool)
        .await?;

        let mut csv = String::from("event_id,client_id,event_type,timestamp,response_time_ms,bytes_transferred,error_code\n");
        
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                row.get::<String, _>("event_id"),
                row.get::<Option<String>, _>("client_id").unwrap_or_else(|| "".to_string()),
                row.get::<String, _>("event_type"),
                row.get::<DateTime<Utc>, _>("timestamp"),
                row.get::<Option<i32>, _>("response_time_ms").unwrap_or(0),
                row.get::<Option<i64>, _>("bytes_transferred").unwrap_or(0),
                row.get::<Option<String>, _>("error_code").unwrap_or_else(|| "".to_string()),
            ));
        }

        Ok(csv)
    }

    pub async fn record_metrics(&self, metrics: RealtimeMetrics) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO connection_metrics 
            (active_connections, peak_connections, events_per_second, subscriptions_count, memory_usage_bytes, cpu_usage_percent, disk_usage_bytes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(metrics.active_connections as i32)
        .bind(metrics.active_connections as i32) // Using current as peak for now
        .bind(metrics.events_per_second as f32)
        .bind(metrics.subscriptions_count as i32)
        .bind(metrics.memory_usage as i64)
        .bind(metrics.cpu_usage as f32)
        .bind(metrics.disk_usage as i64)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }
}
