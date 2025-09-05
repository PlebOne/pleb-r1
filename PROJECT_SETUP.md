# Pleb.One Project Setup Guide

## Quick Start

This guide will help you set up the Pleb.One development environment and get started with the initial implementation.

## Prerequisites

### Required Tools
- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Go** (1.21+): Download from [golang.org](https://golang.org/download/)
- **Node.js** (18+): Download from [nodejs.org](https://nodejs.org/)
- **Docker** & **Docker Compose**: For local development
- **PostgreSQL** (15+): Database server
- **Redis** (7+): Caching and session store

### Development Environment Setup

#### 1. Clone and Initialize Repository
```bash
git clone https://github.com/pleb-one/pleb-one.git
cd pleb-one
make setup
```

#### 2. Environment Configuration
```bash
# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

#### 3. Start Development Services
```bash
# Start all services with Docker Compose
docker-compose up -d

# Or start individual services
make dev-relay      # Rust relay engine
make dev-api        # Go API services
make dev-web        # Next.js frontend
```

## Project Structure

```
pleb-one/
├── .github/
│   └── workflows/          # CI/CD pipelines
├── services/
│   ├── relay-engine/       # Rust - Core Nostr relay
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── websocket.rs
│   │   │   ├── event.rs
│   │   │   └── storage.rs
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   ├── api-gateway/        # Go - Main API gateway
│   │   ├── cmd/
│   │   ├── internal/
│   │   ├── pkg/
│   │   ├── go.mod
│   │   └── Dockerfile
│   ├── user-service/       # Go - User management & subscriptions
│   ├── billing-service/    # Go - Payment processing & Lightning
│   ├── community-service/  # Go - Community features
│   └── analytics-service/  # Go - Metrics & analytics
├── frontend/
│   ├── web/               # Next.js web application
│   │   ├── src/
│   │   │   ├── app/
│   │   │   ├── components/
│   │   │   └── lib/
│   │   ├── package.json
│   │   └── Dockerfile
│   └── mobile/            # React Native (future)
├── infrastructure/
│   ├── docker/
│   │   ├── docker-compose.yml
│   │   └── docker-compose.prod.yml
│   ├── kubernetes/        # K8s manifests
│   └── terraform/         # Infrastructure as code
├── docs/                  # Documentation
├── scripts/              # Development & deployment scripts
├── Makefile              # Build automation
└── README.md
```

## Initial Implementation Steps

### Step 1: Set Up Relay Engine (Rust)

#### Create Cargo Workspace
```toml
# Cargo.toml
[workspace]
members = ["services/relay-engine"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.20"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### Basic Relay Implementation
```rust
// services/relay-engine/src/main.rs
use anyhow::Result;
use tracing::{info, error};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod websocket;
mod event;
mod storage;

use websocket::RelayServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = load_config()?;
    
    // Initialize database connection
    let db_pool = storage::init_db_pool(&config.database_url).await?;
    
    // Start relay server
    let server = RelayServer::new(config.port, db_pool);
    
    info!("Starting Pleb.One relay on port {}", config.port);
    server.start().await?;
    
    Ok(())
}

fn load_config() -> Result<Config> {
    Ok(Config {
        port: std::env::var("RELAY_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?,
        database_url: std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set"),
    })
}

struct Config {
    port: u16,
    database_url: String,
}
```

### Step 2: Set Up API Gateway (Go)

#### Initialize Go Module
```go
// services/api-gateway/go.mod
module github.com/pleb-one/api-gateway

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/golang-jwt/jwt/v5 v5.0.0
    github.com/redis/go-redis/v9 v9.0.5
    github.com/lib/pq v1.10.9
    github.com/google/uuid v1.3.0
)
```

#### Basic API Server
```go
// services/api-gateway/cmd/main.go
package main

import (
    "log"
    "net/http"
    "os"
    
    "github.com/gin-gonic/gin"
    "github.com/pleb-one/api-gateway/internal/handlers"
    "github.com/pleb-one/api-gateway/internal/middleware"
    "github.com/pleb-one/api-gateway/internal/services"
)

func main() {
    // Initialize services
    userService := services.NewUserService()
    relayService := services.NewRelayService()
    
    // Initialize Gin router
    r := gin.Default()
    
    // Middleware
    r.Use(middleware.CORS())
    r.Use(middleware.Logger())
    r.Use(middleware.Recovery())
    
    // Health check
    r.GET("/health", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{"status": "healthy"})
    })
    
    // API routes
    api := r.Group("/api/v1")
    {
        // User routes
        users := api.Group("/users")
        users.POST("/register", handlers.RegisterUser(userService))
        users.GET("/:pubkey", handlers.GetUser(userService))
        
        // Relay routes  
        relay := api.Group("/relay")
        relay.GET("/status", handlers.GetRelayStatus(relayService))
        relay.GET("/metrics", handlers.GetRelayMetrics(relayService))
    }
    
    port := os.Getenv("PORT")
    if port == "" {
        port = "8080"
    }
    
    log.Printf("Starting API Gateway on port %s", port)
    r.Run(":" + port)
}
```

### Step 3: Set Up Frontend (Next.js)

#### Initialize Next.js Project
```json
{
  "name": "pleb-one-web",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "14.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "@tanstack/react-query": "^5.0.0",
    "zustand": "^4.4.0",
    "tailwindcss": "^3.3.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "typescript": "^5.0.0"
  }
}
```

#### Basic Layout
```tsx
// frontend/web/src/app/layout.tsx
import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Pleb.One - By Plebs, For Plebs',
  description: 'Community-owned Nostr infrastructure',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="bg-gray-50 dark:bg-gray-900">
        <header className="bg-white dark:bg-gray-800 shadow">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex items-center">
                <h1 className="text-xl font-bold text-orange-500">
                  Pleb.One
                </h1>
              </div>
              <div className="flex items-center space-x-4">
                <nav>
                  <a href="/dashboard" className="text-gray-700 hover:text-orange-500">
                    Dashboard
                  </a>
                  <a href="/community" className="text-gray-700 hover:text-orange-500 ml-4">
                    Community
                  </a>
                </nav>
              </div>
            </div>
          </div>
        </header>
        
        <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          {children}
        </main>
        
        <footer className="bg-white dark:bg-gray-800 border-t mt-12">
          <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
            <p className="text-center text-gray-500">
              Built by plebs, for plebs. Open source and community-owned.
            </p>
          </div>
        </footer>
      </body>
    </html>
  )
}
```

### Step 4: Development Automation

#### Makefile for Build Automation
```makefile
# Makefile
.PHONY: setup dev build test clean docker-build docker-up

# Setup development environment
setup:
	@echo "Setting up Pleb.One development environment..."
	cd services/relay-engine && cargo check
	cd services/api-gateway && go mod tidy
	cd frontend/web && npm install
	@echo "Setup complete!"

# Development servers
dev-relay:
	cd services/relay-engine && cargo run

dev-api:
	cd services/api-gateway && go run cmd/main.go

dev-web:
	cd frontend/web && npm run dev

# Build all services
build:
	cd services/relay-engine && cargo build --release
	cd services/api-gateway && go build -o bin/api-gateway cmd/main.go
	cd frontend/web && npm run build

# Run tests
test:
	cd services/relay-engine && cargo test
	cd services/api-gateway && go test ./...
	cd frontend/web && npm test

# Docker operations
docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

# Clean build artifacts
clean:
	cd services/relay-engine && cargo clean
	cd services/api-gateway && rm -rf bin/
	cd frontend/web && rm -rf .next/
```

#### Docker Compose for Local Development
```yaml
# infrastructure/docker/docker-compose.yml
version: '3.8'

services:
  postgres:
    image: timescale/timescaledb:latest-pg15
    environment:
      POSTGRES_DB: pleb_one
      POSTGRES_USER: pleb_one
      POSTGRES_PASSWORD: development
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  nats:
    image: nats:2.10-alpine
    ports:
      - "4222:4222"
      - "8222:8222"
    command: "--http_port 8222"

  relay-engine:
    build:
      context: ../../services/relay-engine
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://pleb_one:development@postgres:5432/pleb_one
      REDIS_URL: redis://redis:6379
      RUST_LOG: info
    depends_on:
      - postgres
      - redis
      - nats

  api-gateway:
    build:
      context: ../../services/api-gateway
      dockerfile: Dockerfile
    ports:
      - "8081:8080"
    environment:
      DATABASE_URL: postgres://pleb_one:development@postgres:5432/pleb_one
      REDIS_URL: redis://redis:6379
      RELAY_URL: ws://relay-engine:8080
    depends_on:
      - postgres
      - redis
      - relay-engine

  web:
    build:
      context: ../../frontend/web
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      NEXT_PUBLIC_API_URL: http://localhost:8081
      NEXT_PUBLIC_RELAY_URL: ws://localhost:8080
    depends_on:
      - api-gateway

volumes:
  postgres_data:
  redis_data:
```

## Getting Started Checklist

### Initial Setup
- [ ] Install prerequisites (Rust, Go, Node.js, Docker)
- [ ] Clone repository and run `make setup`
- [ ] Copy `.env.example` to `.env` and configure
- [ ] Start services with `docker-compose up -d`
- [ ] Verify all services are running

### Development Workflow
- [ ] Make changes to code
- [ ] Run tests with `make test`
- [ ] Test locally with `make dev-*`
- [ ] Commit changes with conventional commits
- [ ] Push to feature branch
- [ ] Open pull request

### Next Steps
1. **Week 1**: Complete basic relay implementation
2. **Week 2**: Implement subscription and billing API
3. **Week 3**: Build dashboard frontend
4. **Week 4**: Add community features

## Community Involvement

### Contributing
- Join our Discord: [discord.gg/pleb-one](https://discord.gg/pleb-one)
- Follow development: [github.com/pleb-one/pleb-one](https://github.com/pleb-one/pleb-one)
- Weekly community calls: Fridays 2PM UTC

### Getting Help
- Technical questions: GitHub Discussions
- Community chat: Discord #dev-help
- Documentation: [docs.pleb.one](https://docs.pleb.one)

This setup guide provides everything needed to get started with Pleb.One development. The modular architecture allows team members to work on different components independently while maintaining integration through well-defined APIs.
