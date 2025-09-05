# Pleb.One Development Makefile
# Provides commands for building, testing, and running the entire project

.PHONY: help setup build test clean dev prod docker-build docker-up docker-down lint format check docs

# Default target
help: ## Show this help message
	@echo "Pleb.One Development Commands"
	@echo "============================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Environment Setup
setup: ## Set up the development environment
	@echo "🚀 Setting up Pleb.One development environment..."
	@echo "📦 Installing Rust dependencies..."
	cd services/relay-engine && cargo fetch
	cd services/nostr-types && cargo fetch
	cd services/storage-layer && cargo fetch
	@echo "🔧 Installing Go dependencies..."
	cd services/api-gateway && go mod download
	cd services/billing-service && go mod download
	@echo "📱 Installing Node.js dependencies..."
	cd frontend/web && npm install
	@echo "🎯 Creating configuration files..."
	cp config/default.toml.example config/default.toml || echo "Config already exists"
	cp .env.example .env || echo ".env already exists"
	@echo "✅ Setup complete! Run 'make dev' to start development servers."

# Build Commands
build: build-rust build-go build-web ## Build all services

build-rust: ## Build Rust services
	@echo "🦀 Building Rust services..."
	cargo build --workspace --release

build-go: ## Build Go services
	@echo "🐹 Building Go services..."
	cd services/api-gateway && go build -o bin/api-gateway cmd/main.go
	cd services/billing-service && go build -o bin/billing-service cmd/main.go

build-web: ## Build web frontend
	@echo "⚛️ Building web frontend..."
	cd frontend/web && npm run build

# Development Commands
dev: ## Start all development servers
	@echo "🔥 Starting development environment..."
	docker-compose -f infrastructure/docker/docker-compose.yml up -d postgres redis nats
	@echo "⏳ Waiting for databases to be ready..."
	@sleep 5
	@echo "🚀 Starting services..."
	$(MAKE) -j7 dev-relay dev-api dev-billing dev-analytics dev-analytics-api dev-blossom dev-web

dev-relay: ## Start relay engine (Rust)
	@echo "🦀 Starting relay engine..."
	cd services/relay-engine && cargo run

dev-api: ## Start API gateway (Go)
	@echo "🐹 Starting API gateway..."
	cd services/api-gateway && go run cmd/main.go

dev-billing: ## Start billing service (Go)
	@echo "💰 Starting billing service..."
	cd services/billing-service && go run cmd/main.go

dev-analytics: ## Start analytics service (Rust)
	@echo "📊 Starting analytics service..."
	cd services/analytics-service && cargo run

dev-analytics-api: ## Start analytics API (Go)
	@echo "📈 Starting analytics API..."
	cd services/analytics-api && go run main.go

dev-blossom: ## Start blossom server (Go)
	@echo "🌸 Starting blossom server..."
	cd services/blossom-server && go run main.go

dev-web: ## Start web frontend (Next.js)
	@echo "⚛️ Starting web frontend..."
	cd frontend/web && npm run dev

# Testing Commands
test: test-rust test-go test-web ## Run all tests

test-rust: ## Run Rust tests
	@echo "🦀 Running Rust tests..."
	cargo test --workspace

test-go: ## Run Go tests
	@echo "🐹 Running Go tests..."
	cd services/api-gateway && go test ./...
	cd services/billing-service && go test ./...

test-web: ## Run web frontend tests
	@echo "⚛️ Running web tests..."
	cd frontend/web && npm test

test-integration: ## Run integration tests
	@echo "🔗 Running integration tests..."
	docker-compose -f infrastructure/docker/docker-compose.yml up -d
	@sleep 10
	./scripts/integration-tests.sh
	docker-compose -f infrastructure/docker/docker-compose.yml down

# Code Quality Commands
lint: lint-rust lint-go lint-web ## Run all linters

lint-rust: ## Run Rust linter
	@echo "🦀 Linting Rust code..."
	cargo clippy --workspace -- -D warnings

lint-go: ## Run Go linter
	@echo "🐹 Linting Go code..."
	cd services/api-gateway && golangci-lint run
	cd services/billing-service && golangci-lint run

lint-web: ## Run web linter
	@echo "⚛️ Linting web code..."
	cd frontend/web && npm run lint

format: format-rust format-go format-web ## Format all code

format-rust: ## Format Rust code
	@echo "🦀 Formatting Rust code..."
	cargo fmt --all

format-go: ## Format Go code
	@echo "🐹 Formatting Go code..."
	cd services/api-gateway && go fmt ./...
	cd services/billing-service && go fmt ./...

format-web: ## Format web code
	@echo "⚛️ Formatting web code..."
	cd frontend/web && npm run format

check: lint test ## Run linting and tests

# Docker Commands
docker-build: ## Build all Docker images
	@echo "🐳 Building Docker images..."
	docker-compose -f infrastructure/docker/docker-compose.yml build

docker-up: ## Start all services with Docker
	@echo "🐳 Starting services with Docker..."
	docker-compose -f infrastructure/docker/docker-compose.yml up -d

docker-down: ## Stop all Docker services
	@echo "🐳 Stopping Docker services..."
	docker-compose -f infrastructure/docker/docker-compose.yml down

docker-logs: ## Show Docker logs
	docker-compose -f infrastructure/docker/docker-compose.yml logs -f

docker-clean: ## Clean Docker containers and volumes
	@echo "🐳 Cleaning Docker containers and volumes..."
	docker-compose -f infrastructure/docker/docker-compose.yml down -v
	docker system prune -f

# Production Commands
prod: ## Build for production
	@echo "🚀 Building for production..."
	ENVIRONMENT=production $(MAKE) build

deploy-staging: ## Deploy to staging environment
	@echo "🚀 Deploying to staging..."
	./scripts/deploy-staging.sh

deploy-production: ## Deploy to production environment
	@echo "🚀 Deploying to production..."
	./scripts/deploy-production.sh

# Database Commands
db-migrate: ## Run database migrations
	@echo "📊 Running database migrations..."
	cd services/relay-engine && cargo run -- migrate

db-reset: ## Reset database (WARNING: destroys all data)
	@echo "⚠️ Resetting database..."
	docker-compose -f infrastructure/docker/docker-compose.yml stop postgres
	docker-compose -f infrastructure/docker/docker-compose.yml rm -f postgres
	docker volume rm docker_postgres_data
	docker-compose -f infrastructure/docker/docker-compose.yml up -d postgres
	@sleep 5
	$(MAKE) db-migrate

db-backup: ## Create database backup
	@echo "📊 Creating database backup..."
	./scripts/db-backup.sh

db-restore: ## Restore database from backup
	@echo "📊 Restoring database..."
	./scripts/db-restore.sh

# Monitoring Commands
metrics: ## Open metrics dashboard
	@echo "📊 Opening metrics dashboard..."
	open http://localhost:3001

logs: ## Show application logs
	@echo "📋 Showing logs..."
	docker-compose -f infrastructure/docker/docker-compose.yml logs -f relay-engine api-gateway billing-service

monitor: ## Start monitoring stack
	@echo "📊 Starting monitoring stack..."
	docker-compose -f infrastructure/docker/docker-compose.yml up -d prometheus grafana jaeger

# Documentation Commands
docs: ## Generate and serve documentation
	@echo "📚 Generating documentation..."
	cd services/api-gateway && swag init -g cmd/main.go
	@echo "📖 Documentation available at:"
	@echo "  - API Docs: http://localhost:8000/swagger/index.html"
	@echo "  - Rust Docs: cargo doc --open"

docs-rust: ## Generate Rust documentation
	@echo "🦀 Generating Rust documentation..."
	cargo doc --workspace --open

docs-go: ## Generate Go documentation
	@echo "🐹 Generating Go documentation..."
	cd services/api-gateway && swag init -g cmd/main.go
	cd services/billing-service && swag init -g cmd/main.go

# Utility Commands
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	cd services/api-gateway && rm -rf bin/
	cd services/billing-service && rm -rf bin/
	cd frontend/web && rm -rf .next/ dist/

install-tools: ## Install development tools
	@echo "🛠️ Installing development tools..."
	@echo "Installing Rust tools..."
	rustup component add rustfmt clippy
	@echo "Installing Go tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install github.com/swaggo/swag/cmd/swag@latest
	@echo "Installing Node.js tools..."
	npm install -g prettier eslint

security-scan: ## Run security scans
	@echo "🔒 Running security scans..."
	cargo audit
	cd services/api-gateway && govulncheck ./...
	cd services/billing-service && govulncheck ./...
	cd frontend/web && npm audit

release: ## Create a new release
	@echo "🎉 Creating release..."
	./scripts/release.sh

# Quick status check
status: ## Show service status
	@echo "📊 Service Status:"
	@echo "=================="
	@docker-compose -f infrastructure/docker/docker-compose.yml ps

# Performance testing
perf: ## Run performance tests
	@echo "⚡ Running performance tests..."
	./scripts/performance-tests.sh

# Load testing
load-test: ## Run load tests
	@echo "🔥 Running load tests..."
	./scripts/load-tests.sh
