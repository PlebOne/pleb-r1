#!/bin/bash

# Pleb-R1 Production Deployment Script
# Usage: ./deploy.sh [dev|prod]

set -e

MODE=${1:-dev}
PROJECT_DIR="/home/plebone/pleb-r1"

echo "🚀 Starting Pleb-R1 Deployment (Mode: $MODE)"
echo "================================================"

# Change to project directory
cd "$PROJECT_DIR"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not available"
    exit 1
fi

echo "✅ Docker and Docker Compose are available"

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.docker .env
    echo "⚠️  Please edit .env with your production values before continuing"
    echo "   Key variables to set:"
    echo "   - POSTGRES_PASSWORD"
    echo "   - REDIS_PASSWORD"
    echo "   - RELAY_NAME"
    echo "   - RELAY_CONTACT"
    if [ "$MODE" = "prod" ]; then
        echo "❌ Please configure .env for production deployment"
        exit 1
    fi
fi

# Pull latest images
echo "📦 Pulling Docker images..."
docker compose pull postgres redis

# Stop existing services
echo "🛑 Stopping existing services..."
docker compose down 2>/dev/null || true

# Start services based on mode
if [ "$MODE" = "prod" ]; then
    echo "🏭 Starting production services..."
    docker compose up -d postgres redis
    echo "⏳ Waiting for databases to be ready..."
    sleep 10
    
    # Check database health
    echo "🔍 Checking database health..."
    docker compose exec postgres pg_isready -U pleb_user -d pleb_r1 || {
        echo "❌ PostgreSQL is not ready"
        exit 1
    }
    
    echo "✅ PostgreSQL is ready"
    
    # Build and start relay
    echo "🔨 Building relay engine..."
    docker compose build relay-engine
    docker compose up -d relay-engine
    
else
    echo "🧪 Starting development services..."
    docker compose up -d postgres redis
    echo "⏳ Waiting for databases to be ready..."
    sleep 10
    
    echo "🔨 Building and starting development server..."
    cd services/relay-engine
    cargo build --bin dev-server
    echo "✅ Development server built successfully"
    echo "💡 To start the dev server, run: cargo run --bin dev-server"
    cd ../..
fi

# Show status
echo ""
echo "📊 Service Status:"
echo "=================="
docker compose ps

echo ""
echo "🔗 Available Endpoints:"
echo "======================"
if [ "$MODE" = "prod" ]; then
    echo "• WebSocket Relay: ws://localhost:8080"
    echo "• Relay Info: http://localhost:8080"
    echo "• Metrics: http://localhost:8080/metrics"
else
    echo "• Development Server: http://localhost:8080"
    echo "• Landing Page: http://localhost:8081/landing.html"
    echo "• API Test: http://localhost:8081/api-test.html"
fi

echo ""
echo "🎉 Deployment completed successfully!"
echo ""

if [ "$MODE" = "prod" ]; then
    echo "🔧 Next Steps for Production:"
    echo "• Set up SSL/TLS certificates"
    echo "• Configure domain name"
    echo "• Set up monitoring alerts"
    echo "• Test WebSocket connections"
else
    echo "🔧 Next Steps for Development:"
    echo "• Start dev server: cd services/relay-engine && cargo run --bin dev-server"
    echo "• Test authentication API"
    echo "• Check landing page functionality"
fi

echo ""
echo "📚 For more information, see:"
echo "• README.md"
echo "• PRODUCTION_READINESS_ASSESSMENT.md"
echo "• DOCKER.md"
