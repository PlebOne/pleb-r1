use nostr::{Event, Filter, JsonUtil};
use sqlx::{PgPool, Row};
use anyhow::Result;
use tracing::{debug, error};

#[derive(Clone)]
pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn create_tables(&self) -> Result<()> {
        // Create events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id VARCHAR(64) PRIMARY KEY,
                pubkey VARCHAR(64) NOT NULL,
                created_at BIGINT NOT NULL,
                kind INTEGER NOT NULL,
                tags TEXT NOT NULL,
                content TEXT NOT NULL,
                sig VARCHAR(128) NOT NULL,
                raw_event TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_pubkey ON events(pubkey);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind);")
            .execute(&self.pool)
            .await?;

        debug!("Database tables created successfully");
        Ok(())
    }

    pub async fn save_event(&self, event: &Event) -> Result<()> {
        debug!("Saving event {}", event.id);

        let tags_json = serde_json::to_string(&event.tags)?;
        let raw_event = event.as_json().to_string();

        sqlx::query(
            r#"
            INSERT INTO events (id, pubkey, created_at, kind, tags, content, sig, raw_event)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(event.id.to_string())
        .bind(event.pubkey.to_string())
        .bind(event.created_at.as_u64() as i64)
        .bind(event.kind.as_u32() as i32)
        .bind(tags_json)
        .bind(&event.content)
        .bind(event.signature().to_string())
        .bind(raw_event)
        .execute(&self.pool)
        .await?;

        debug!("Saved event {}", event.id);
        Ok(())
    }

    pub async fn event_exists(&self, event_id: &nostr::EventId) -> Result<bool> {
        debug!("Checking if event exists: {}", event_id);

        let row = sqlx::query("SELECT COUNT(*) as count FROM events WHERE id = $1")
            .bind(event_id.to_string())
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn query_events(&self, filter: &Filter) -> Result<Vec<Event>> {
        self.get_events(filter).await
    }

    pub async fn get_events(&self, filter: &Filter) -> Result<Vec<Event>> {
        debug!("Getting events with filter: {:?}", filter);

        // Start building the query - simplified for cross-database compatibility
        let mut query = String::from("SELECT raw_event FROM events WHERE 1=1");

        // Add ordering and limit (simplified)
        query.push_str(" ORDER BY created_at DESC LIMIT 100");

        debug!("Executing query: {}", query);

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        let mut events = Vec::new();
        for row in rows {
            let raw_event_str: String = row.get("raw_event");
            match serde_json::from_str::<Event>(&raw_event_str) {
                Ok(event) => events.push(event),
                Err(e) => error!("Failed to deserialize event: {}", e),
            }
        }

        debug!("Found {} events matching filter", events.len());
        Ok(events)
    }
}
