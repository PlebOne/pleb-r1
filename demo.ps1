#!/usr/bin/env pwsh
# Quick Relay Demo Script

Write-Host "🧪 Pleb.One Relay Demo" -ForegroundColor Magenta
Write-Host "========================" -ForegroundColor Magenta

# Check if relay is running
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080" -TimeoutSec 5 -ErrorAction Stop
    Write-Host "✅ Relay is running!" -ForegroundColor Green
    
    # Show relay info
    Write-Host "`n📋 Relay Information:" -ForegroundColor Cyan
    $info = $response.Content | ConvertFrom-Json
    Write-Host "   Name: $($info.name)" -ForegroundColor White
    Write-Host "   Description: $($info.description)" -ForegroundColor White
    Write-Host "   Version: $($info.version)" -ForegroundColor White
    Write-Host "   Supported NIPs: $($info.supported_nips -join ', ')" -ForegroundColor White
    
    # Check metrics
    Write-Host "`n📊 Checking Metrics..." -ForegroundColor Cyan
    try {
        $metrics = Invoke-WebRequest -Uri "http://localhost:8080/metrics" -TimeoutSec 5
        $lines = $metrics.Content -split "`n" | Where-Object { $_ -match "^relay_" -and $_ -notmatch "^#" }
        Write-Host "   Available metrics: $($lines.Count)" -ForegroundColor White
        $lines | Select-Object -First 5 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
        if ($lines.Count -gt 5) {
            Write-Host "   ... and $($lines.Count - 5) more" -ForegroundColor Gray
        }
    }
    catch {
        Write-Host "   ⚠️  Metrics endpoint unavailable" -ForegroundColor Yellow
    }
    
    Write-Host "`n🎉 Demo complete! Your relay is working perfectly!" -ForegroundColor Green
    Write-Host "`n💡 To connect clients, use: ws://localhost:8080" -ForegroundColor Cyan
    
}
catch {
    Write-Host "❌ Relay is not running!" -ForegroundColor Red
    Write-Host "💡 Run './run-relay.ps1' to start the relay first" -ForegroundColor Yellow
}
