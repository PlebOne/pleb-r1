#!/usr/bin/env pwsh
# NrelayOne - SQLite Relay Startup Script

Write-Host "🚀 Starting Pleb.One Nostr Relay with SQLite..." -ForegroundColor Green

# Set environment variables
$env:DATABASE_URL = "sqlite::memory:"
$env:RUST_LOG = "info"

# Build and run the relay
Write-Host "Building relay..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful! Starting relay..." -ForegroundColor Green
    Write-Host ""
    Write-Host "📡 Relay will be available at:" -ForegroundColor Cyan
    Write-Host "   WebSocket: ws://localhost:8080" -ForegroundColor White
    Write-Host "   Relay Info: http://localhost:8080" -ForegroundColor White
    Write-Host "   Metrics: http://localhost:8080/metrics" -ForegroundColor White
    Write-Host ""
    Write-Host "Press Ctrl+C to stop the relay" -ForegroundColor Yellow
    Write-Host "----------------------------------------" -ForegroundColor Gray
    
    cargo run --release
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
