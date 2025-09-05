#!/usr/bin/env pwsh
# Docker deployment scripts for Pleb-R1

param(
    [Parameter(Position=0)]
    [ValidateSet("dev", "prod", "monitoring", "stop", "clean", "logs", "status")]
    [string]$Command = "dev",
    
    [Parameter()]
    [string]$Service = ""
)

Write-Host "🐳 Pleb-R1 Docker Management" -ForegroundColor Cyan

switch ($Command) {
    "dev" {
        Write-Host "🚀 Starting development environment..." -ForegroundColor Green
        docker-compose -f docker-compose.dev.yml up -d
        Write-Host "✅ Development environment started!" -ForegroundColor Green
        Write-Host "🌐 Landing page: http://localhost:3000" -ForegroundColor Yellow
        Write-Host "🔌 API server: http://localhost:8080" -ForegroundColor Yellow
        Write-Host "📊 DB viewer: http://localhost:8081" -ForegroundColor Yellow
    }
    
    "prod" {
        Write-Host "🏭 Starting production environment..." -ForegroundColor Green
        if (-not (Test-Path ".env")) {
            Write-Host "⚠️  Creating .env file from template..." -ForegroundColor Yellow
            Copy-Item ".env.docker" ".env"
            Write-Host "📝 Please edit .env file with your configuration!" -ForegroundColor Red
            return
        }
        docker-compose up -d
        Write-Host "✅ Production environment started!" -ForegroundColor Green
        Write-Host "🌐 Web interface: http://localhost" -ForegroundColor Yellow
        Write-Host "🔌 Relay: ws://localhost:8080" -ForegroundColor Yellow
    }
    
    "monitoring" {
        Write-Host "📊 Starting with monitoring stack..." -ForegroundColor Green
        docker-compose --profile monitoring up -d
        Write-Host "✅ Monitoring stack started!" -ForegroundColor Green
        Write-Host "📈 Prometheus: http://localhost:9090" -ForegroundColor Yellow
        Write-Host "📊 Grafana: http://localhost:3000" -ForegroundColor Yellow
    }
    
    "stop" {
        Write-Host "🛑 Stopping all services..." -ForegroundColor Red
        docker-compose down
        docker-compose -f docker-compose.dev.yml down
        Write-Host "✅ All services stopped!" -ForegroundColor Green
    }
    
    "clean" {
        Write-Host "🧹 Cleaning up Docker resources..." -ForegroundColor Yellow
        docker-compose down -v --remove-orphans
        docker-compose -f docker-compose.dev.yml down -v --remove-orphans
        docker system prune -f
        Write-Host "✅ Cleanup complete!" -ForegroundColor Green
    }
    
    "logs" {
        if ($Service) {
            Write-Host "📋 Showing logs for $Service..." -ForegroundColor Blue
            docker-compose logs -f $Service
        } else {
            Write-Host "📋 Showing all logs..." -ForegroundColor Blue
            docker-compose logs -f
        }
    }
    
    "status" {
        Write-Host "📊 Docker services status:" -ForegroundColor Blue
        docker-compose ps
        Write-Host "`n📊 Development services:" -ForegroundColor Blue
        docker-compose -f docker-compose.dev.yml ps
    }
    
    default {
        Write-Host @"
🐳 Pleb.One Docker Management

Usage: ./docker-manage.ps1 [command] [service]

Commands:
  dev         Start development environment (default)
  prod        Start production environment  
  monitoring  Start with monitoring (Prometheus + Grafana)
  stop        Stop all services
  clean       Stop and remove all containers/volumes
  logs        Show logs (optionally for specific service)
  status      Show status of all services

Examples:
  ./docker-manage.ps1 dev
  ./docker-manage.ps1 prod
  ./docker-manage.ps1 logs relay-engine
  ./docker-manage.ps1 status
"@ -ForegroundColor White
    }
}
