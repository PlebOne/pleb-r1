# Pleb.One Technical Architecture

## System Overview

Pleb.One will be built as a distributed, community-owned Nostr infrastructure platform using a hybrid Go/Rust architecture optimized for reliability, performance, and community governance.

## Core Architecture Principles

1. **Microservices Design**: Loosely coupled services for independent scaling
2. **Event-Driven Architecture**: Async communication using message queues
3. **Database Per Service**: Each service owns its data
4. **API-First**: All functionality exposed via well-documented APIs
5. **Cloud-Native**: Container-based deployment with Kubernetes
6. **Observability**: Comprehensive monitoring, logging, and tracing

## Technology Stack Details

### Backend Services

#### Rust Components (Performance Critical)
- **Relay Engine** (`nostr-relay-rs`)
  - WebSocket server implementation
  - Event validation and filtering
  - Real-time event streaming
  - Memory-efficient event storage

- **Cryptographic Service** (`crypto-service-rs`)
  - Event signature verification
  - Schnorr signature operations
  - Key derivation and management
  - NIP-19 encoding/decoding

- **Event Processor** (`event-processor-rs`)
  - High-throughput event ingestion
  - Event deduplication
  - Content filtering pipeline
  - Metrics collection

#### Go Components (Business Logic & APIs)
- **API Gateway** (`api-gateway`)
  - Authentication/authorization
  - Rate limiting
  - Request routing
  - Response caching

- **User Service** (`user-service`)
  - Account management
  - NIP-05 name registration
  - Profile management
  - Community membership

- **Community Service** (`community-service`)
  - Governance voting
  - Moderation tools
  - Reputation system
  - Content curation

- **Analytics Service** (`analytics-service`)
  - Performance metrics
  - Usage statistics
  - Community insights
  - Reporting APIs

### Frontend Stack

#### Web Application (TypeScript/React)
- **Next.js 14** with App Router
- **TailwindCSS** for styling
- **Zustand** for state management
- **React Query** for data fetching
- **Framer Motion** for animations

#### Mobile Applications
- **React Native** for cross-platform mobile
- **Expo** for development workflow
- **Native modules** for platform-specific crypto operations

## Data Architecture

### Primary Databases

#### Event Store (PostgreSQL + TimescaleDB)
```sql
-- Events table optimized for time-series queries
CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    event_id VARCHAR(64) UNIQUE NOT NULL, -- hex encoded
    pubkey VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    kind INTEGER NOT NULL,
    content TEXT,
    tags JSONB,
    sig VARCHAR(128) NOT NULL,
    received_at TIMESTAMP DEFAULT NOW()
);

-- Hypertable for time-based partitioning
SELECT create_hypertable('events', 'created_at');

-- Indexes for common queries
CREATE INDEX idx_events_pubkey ON events (pubkey);
CREATE INDEX idx_events_kind ON events (kind);
CREATE INDEX idx_events_created_at ON events (created_at DESC);
CREATE INDEX idx_events_tags ON events USING GIN (tags);
```

#### User Data (PostgreSQL)
```sql
-- Users and their pleb.one accounts
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pubkey VARCHAR(64) UNIQUE NOT NULL,
    nip05_name VARCHAR(50) UNIQUE,
    display_name VARCHAR(100),
    about TEXT,
    picture_url TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Community memberships and roles
CREATE TABLE community_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    role VARCHAR(20) DEFAULT 'member',
    reputation_score INTEGER DEFAULT 0,
    joined_at TIMESTAMP DEFAULT NOW()
);
```

### Caching Layer (Redis)

#### Data Structures
```redis
# Active WebSocket connections
websocket:connections:{relay_id} -> SET of connection_ids

# Event cache for fast retrieval
event:cache:{event_id} -> JSON event data (TTL: 24h)

# User sessions
session:{session_id} -> JSON user data (TTL: 7d)

# Rate limiting
ratelimit:{pubkey}:{endpoint} -> counter (TTL: 1h)

# Community voting
vote:{proposal_id}:{pubkey} -> vote_value
```

## Service Communication

### Message Queue (NATS)

#### Event Flows
```
Relay Engine → event.received → Event Processor
Event Processor → event.validated → Analytics Service
Community Service → moderation.action → All Relays
User Service → user.registered → Community Service
```

### API Design

#### REST APIs (Go Services)
```yaml
# OpenAPI 3.0 specification excerpt
paths:
  /api/v1/users/{pubkey}:
    get:
      summary: Get user profile
      parameters:
        - name: pubkey
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-f0-9]{64}$'
      responses:
        200:
          description: User profile
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'

  /api/v1/community/proposals:
    post:
      summary: Create governance proposal
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ProposalRequest'
```

#### WebSocket API (Rust Relay)
```json
// Subscription request
{
  "id": "sub1",
  "type": "REQ",
  "filters": [
    {
      "kinds": [1],
      "since": 1672531200,
      "limit": 100
    }
  ]
}

// Event response
{
  "id": "sub1",
  "type": "EVENT",
  "event": {
    "id": "a1b2c3...",
    "pubkey": "abc123...",
    "created_at": 1672531200,
    "kind": 1,
    "tags": [],
    "content": "Hello Nostr!",
    "sig": "def456..."
  }
}
```

## Security Architecture

### Authentication & Authorization

#### JWT-Based Authentication
```go
type Claims struct {
    Pubkey string `json:"pubkey"`
    Role   string `json:"role"`
    jwt.RegisteredClaims
}

// NIP-98 HTTP Auth implementation
func VerifyNostrAuth(req *http.Request) (*Claims, error) {
    authHeader := req.Header.Get("Authorization")
    if !strings.HasPrefix(authHeader, "Nostr ") {
        return nil, ErrInvalidAuth
    }
    
    eventB64 := strings.TrimPrefix(authHeader, "Nostr ")
    eventBytes, err := base64.StdEncoding.DecodeString(eventB64)
    if err != nil {
        return nil, err
    }
    
    var event NostrEvent
    if err := json.Unmarshal(eventBytes, &event); err != nil {
        return nil, err
    }
    
    // Verify signature and validate auth event
    return validateAuthEvent(&event)
}
```

### Rate Limiting Strategy

#### Multi-Layer Rate Limiting
```rust
// Rust implementation for WebSocket connections
pub struct RateLimiter {
    // Per-connection limits
    conn_limits: HashMap<String, TokenBucket>,
    // Per-pubkey limits
    pubkey_limits: HashMap<String, TokenBucket>,
    // Global limits
    global_limit: TokenBucket,
}

impl RateLimiter {
    pub fn check_limits(&mut self, conn_id: &str, pubkey: &str) -> Result<(), RateLimitError> {
        // Check global rate limit first
        self.global_limit.consume(1)?;
        
        // Check per-pubkey limit
        let pubkey_bucket = self.pubkey_limits
            .entry(pubkey.to_string())
            .or_insert_with(|| TokenBucket::new(100, Duration::minutes(1)));
        pubkey_bucket.consume(1)?;
        
        // Check per-connection limit
        let conn_bucket = self.conn_limits
            .entry(conn_id.to_string())
            .or_insert_with(|| TokenBucket::new(50, Duration::minutes(1)));
        conn_bucket.consume(1)?;
        
        Ok(())
    }
}
```

## Performance Optimizations

### Event Processing Pipeline

#### Rust Event Processor
```rust
use tokio::sync::mpsc;
use sqlx::PgPool;

pub struct EventProcessor {
    db: PgPool,
    event_rx: mpsc::Receiver<NostrEvent>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl EventProcessor {
    pub async fn process_events(&mut self) -> Result<(), ProcessorError> {
        let mut batch = Vec::with_capacity(self.batch_size);
        let mut timeout = tokio::time::interval(self.batch_timeout);
        
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    if let Some(event) = event {
                        batch.push(event);
                        
                        if batch.len() >= self.batch_size {
                            self.flush_batch(&mut batch).await?;
                        }
                    }
                }
                _ = timeout.tick() => {
                    if !batch.is_empty() {
                        self.flush_batch(&mut batch).await?;
                    }
                }
            }
        }
    }
    
    async fn flush_batch(&self, batch: &mut Vec<NostrEvent>) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;
        
        for event in batch.drain(..) {
            sqlx::query!(
                "INSERT INTO events (event_id, pubkey, created_at, kind, content, tags, sig) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (event_id) DO NOTHING",
                event.id,
                event.pubkey,
                event.created_at,
                event.kind,
                event.content,
                event.tags,
                event.sig
            )
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }
}
```

### Caching Strategy

#### Multi-Level Caching
```go
type CacheService struct {
    redis    *redis.Client
    memory   *bigcache.BigCache
    postgres *sql.DB
}

func (c *CacheService) GetEvent(eventID string) (*Event, error) {
    // L1: Memory cache (fastest)
    if data, err := c.memory.Get(eventID); err == nil {
        var event Event
        if err := json.Unmarshal(data, &event); err == nil {
            return &event, nil
        }
    }
    
    // L2: Redis cache (fast)
    if data, err := c.redis.Get(ctx, "event:"+eventID).Result(); err == nil {
        var event Event
        if err := json.Unmarshal([]byte(data), &event); err == nil {
            // Populate L1 cache
            c.memory.Set(eventID, []byte(data))
            return &event, nil
        }
    }
    
    // L3: Database (authoritative)
    event, err := c.getEventFromDB(eventID)
    if err != nil {
        return nil, err
    }
    
    // Populate caches
    data, _ := json.Marshal(event)
    c.redis.Set(ctx, "event:"+eventID, data, 24*time.Hour)
    c.memory.Set(eventID, data)
    
    return event, nil
}
```

## Monitoring & Observability

### Metrics Collection

#### Prometheus Metrics
```rust
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    static ref EVENTS_PROCESSED: Counter = Counter::new(
        "nostr_events_processed_total",
        "Total number of events processed"
    ).unwrap();
    
    static ref EVENT_PROCESSING_DURATION: Histogram = Histogram::new(
        prometheus::HistogramOpts::new(
            "nostr_event_processing_duration_seconds",
            "Time spent processing events"
        )
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: Gauge = Gauge::new(
        "nostr_active_connections",
        "Number of active WebSocket connections"
    ).unwrap();
}

pub fn record_event_processed() {
    EVENTS_PROCESSED.inc();
}

pub fn record_processing_time(duration: Duration) {
    EVENT_PROCESSING_DURATION.observe(duration.as_secs_f64());
}
```

### Distributed Tracing

#### OpenTelemetry Integration
```go
import (
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/trace"
)

func (s *UserService) CreateUser(ctx context.Context, req *CreateUserRequest) (*User, error) {
    tracer := otel.Tracer("user-service")
    ctx, span := tracer.Start(ctx, "CreateUser")
    defer span.End()
    
    span.SetAttributes(
        attribute.String("user.pubkey", req.Pubkey),
        attribute.String("user.nip05", req.NIP05Name),
    )
    
    // Validate request
    if err := s.validateCreateUserRequest(ctx, req); err != nil {
        span.RecordError(err)
        span.SetStatus(codes.Error, err.Error())
        return nil, err
    }
    
    // Create user in database
    user, err := s.db.CreateUser(ctx, req)
    if err != nil {
        span.RecordError(err)
        span.SetStatus(codes.Error, err.Error())
        return nil, err
    }
    
    span.SetStatus(codes.Ok, "User created successfully")
    return user, nil
}
```

## Deployment Architecture

### Kubernetes Configuration

#### Relay Engine Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: relay-engine
  labels:
    app: relay-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: relay-engine
  template:
    metadata:
      labels:
        app: relay-engine
    spec:
      containers:
      - name: relay-engine
        image: pleb.one/relay-engine:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-credentials
              key: url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: relay-engine-service
spec:
  selector:
    app: relay-engine
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

This technical architecture provides a solid foundation for building pleb.one as a reliable, scalable, and community-focused alternative to nostr.land, emphasizing performance, transparency, and decentralization.
