# Pleb.One Relay Engine - Phase 1

A high-performance Nostr relay implementation built with Rust using the battle-tested `rust-nostr` ecosystem.

## Features

### Core Relay Functionality
- **WebSocket-based Nostr protocol support** using `nostr` crate v0.32
- **Event storage and retrieval** with PostgreSQL backend
- **Real-time subscriptions** with efficient filtering
- **Event validation** including signature verification
- **Rate limiting** per IP address for events, queries, and connections

### Database Integration
- **PostgreSQL storage** with SQLx for async operations
- **Automatic schema creation** with proper indexing
- **Event deduplication** to prevent duplicate storage
- **Efficient querying** with support for all standard Nostr filters

### Monitoring & Metrics
- **Prometheus metrics** for performance monitoring
- **Structured logging** with tracing
- **Connection tracking** and performance measurement
- **Rate limiting statistics** and database operation metrics

### Configuration
- **Environment-based configuration** for easy deployment
- **Configurable rate limits** and connection limits
- **Relay metadata** support (NIP-11)

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   WebSocket     │    │   Rate Limiter  │    │   PostgreSQL    │
│   Handler       │◄──►│   Per IP        │    │   Database      │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Event         │    │   Metrics       │    │   Configuration │
│   Processing    │    │   Collection    │    │   Management    │
│                 │    │   (Prometheus)  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Dependencies

- **nostr v0.32** - Core Nostr protocol implementation
- **nostr-database v0.32** - Database abstractions
- **axum v0.7** - Async web framework with WebSocket support
- **sqlx v0.7** - Async PostgreSQL driver
- **prometheus v0.13** - Metrics collection
- **tokio v1.34** - Async runtime

## Environment Variables

```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/pleb_one

# Server
PORT=8080

# Relay Information (NIP-11)
RELAY_NAME="Pleb.One Relay"
RELAY_DESCRIPTION="A community-owned Nostr relay"
RELAY_PUBKEY="optional_relay_pubkey"
RELAY_CONTACT="admin@pleb.one"
```

## Rate Limits (Configurable)

- **Events**: 60 per minute per IP
- **Queries**: 120 per minute per IP  
- **Connections**: 10 concurrent per IP

## Running the Relay

1. **Set up PostgreSQL database**:
   ```sql
   CREATE DATABASE pleb_one;
   ```

2. **Set environment variables**:
   ```bash
   export DATABASE_URL="postgresql://postgres:password@localhost:5432/pleb_one"
   export PORT=8080
   ```

3. **Run the relay**:
   ```bash
   cargo run
   ```

4. **Monitor metrics**:
   - Metrics endpoint: `http://localhost:8080/metrics`
   - WebSocket endpoint: `ws://localhost:8080/`

## Database Schema

The relay automatically creates the following tables:

### events
- `id` (TEXT PRIMARY KEY) - Event ID
- `pubkey` (TEXT) - Author public key
- `created_at` (BIGINT) - Unix timestamp
- `kind` (INTEGER) - Event kind
- `tags` (JSONB) - Event tags
- `content` (TEXT) - Event content
- `sig` (TEXT) - Event signature
- `raw_event` (JSONB) - Complete event JSON
- `indexed_at` (TIMESTAMP) - When stored

**Indexes**:
- `idx_events_pubkey` - For author queries
- `idx_events_kind` - For kind filtering
- `idx_events_created_at` - For time-based queries
- `idx_events_kind_pubkey` - For combined filtering

## Metrics Available

- `relay_active_connections` - Current WebSocket connections
- `relay_total_connections` - Total connections made
- `relay_events_received_total` - Events received
- `relay_events_stored_total` - Events successfully stored
- `relay_events_rejected_total` - Events rejected
- `relay_queries_received_total` - Queries processed
- `relay_active_subscriptions` - Active subscriptions
- `relay_rate_limited_*` - Rate limiting statistics
- `relay_database_*` - Database operation metrics

## What's Next: Phase 2

Phase 2 will include:
- **User authentication** and paid tiers (3500/6500 sats monthly)
- **Blossom server integration** for premium file storage
- **Advanced filtering** and search capabilities
- **Cluster support** for horizontal scaling
- **Redis caching** for improved performance
- **NIP-05 verification** and identity services

## Contributing

This relay is part of the Pleb.One community ownership initiative. Built with radical transparency and community input.

## License

[License details to be determined by community governance]
