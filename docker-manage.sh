#!/bin/bash
# Docker deployment scripts for Pleb-R1

set -e

COMMAND=${1:-dev}
SERVICE=$2

echo "🐳 Pleb-R1 Docker Management"

case $COMMAND in
    "dev")
        echo "🚀 Starting development environment..."
        docker-compose -f docker-compose.dev.yml up -d
        echo "✅ Development environment started!"
        echo "🌐 Landing page: http://localhost:3000"
        echo "🔌 API server: http://localhost:8080"
        echo "📊 DB viewer: http://localhost:8081"
        ;;
    
    "prod")
        echo "🏭 Starting production environment..."
        if [ ! -f ".env" ]; then
            echo "⚠️  Creating .env file from template..."
            cp .env.docker .env
            echo "📝 Please edit .env file with your configuration!"
            exit 1
        fi
        docker-compose up -d
        echo "✅ Production environment started!"
        echo "🌐 Web interface: http://localhost"
        echo "🔌 Relay: ws://localhost:8080"
        ;;
    
    "monitoring")
        echo "📊 Starting with monitoring stack..."
        docker-compose --profile monitoring up -d
        echo "✅ Monitoring stack started!"
        echo "📈 Prometheus: http://localhost:9090"
        echo "📊 Grafana: http://localhost:3000"
        ;;
    
    "stop")
        echo "🛑 Stopping all services..."
        docker-compose down
        docker-compose -f docker-compose.dev.yml down
        echo "✅ All services stopped!"
        ;;
    
    "clean")
        echo "🧹 Cleaning up Docker resources..."
        docker-compose down -v --remove-orphans
        docker-compose -f docker-compose.dev.yml down -v --remove-orphans
        docker system prune -f
        echo "✅ Cleanup complete!"
        ;;
    
    "logs")
        if [ -n "$SERVICE" ]; then
            echo "📋 Showing logs for $SERVICE..."
            docker-compose logs -f "$SERVICE"
        else
            echo "📋 Showing all logs..."
            docker-compose logs -f
        fi
        ;;
    
    "status")
        echo "📊 Docker services status:"
        docker-compose ps
        echo ""
        echo "📊 Development services:"
        docker-compose -f docker-compose.dev.yml ps
        ;;
    
    "build")
        echo "🔨 Building all images..."
        docker-compose build
        docker-compose -f docker-compose.dev.yml build
        echo "✅ Build complete!"
        ;;
    
    *)
        cat << EOF
🐳 Pleb.One Docker Management

Usage: ./docker-manage.sh [command] [service]

Commands:
  dev         Start development environment (default)
  prod        Start production environment  
  monitoring  Start with monitoring (Prometheus + Grafana)
  stop        Stop all services
  clean       Stop and remove all containers/volumes
  logs        Show logs (optionally for specific service)
  status      Show status of all services
  build       Build all Docker images

Examples:
  ./docker-manage.sh dev
  ./docker-manage.sh prod
  ./docker-manage.sh logs relay-engine
  ./docker-manage.sh status
EOF
        ;;
esac
