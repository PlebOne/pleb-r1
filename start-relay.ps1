#!/usr/bin/env pwsh
# Quick Start Script for Pleb.One Relay

Write-Host "🚀 Starting Pleb.One Relay..." -ForegroundColor Green

# Create data directory if it doesn't exist
if (-not (Test-Path "data")) {
    New-Item -ItemType Directory -Force -Path "data" | Out-Null
    Write-Host "📁 Created data directory" -ForegroundColor Yellow
}

# Set up SQLite database
$dbPath = "data\relay.db"
if (-not (Test-Path $dbPath)) {
    # Create empty SQLite database file
    New-Item -ItemType File -Path $dbPath -Force | Out-Null
    Write-Host "🗄️ Created SQLite database file" -ForegroundColor Yellow
}

# Set environment variables
$env:DATABASE_URL = "sqlite:$((Get-Location).Path)\$dbPath"
$env:RUST_LOG = "info"

Write-Host "🔧 Database URL: $env:DATABASE_URL" -ForegroundColor Cyan
Write-Host "🏗️ Building relay..." -ForegroundColor Yellow

# Build the relay
cargo build --bin relay-engine --quiet

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green
Write-Host "🌐 Starting relay on http://localhost:8080..." -ForegroundColor Green
Write-Host "📊 Metrics: http://localhost:8080/metrics" -ForegroundColor Cyan
Write-Host "📈 API: http://localhost:8080/api/metrics/all" -ForegroundColor Cyan
Write-Host "" 
Write-Host "Press Ctrl+C to stop the relay" -ForegroundColor Gray

# Run the relay
cargo run --bin relay-engine
