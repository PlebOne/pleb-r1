#!/usr/bin/env pwsh

# Comprehensive test runner for NrelayOne relay engine
# This script runs all tests, benchmarks, and generates coverage reports

param(
    [switch]$Unit,
    [switch]$Integration,
    [switch]$E2E,
    [switch]$Bench,
    [switch]$Coverage,
    [switch]$All,
    [switch]$Help
)

function Show-Help {
    Write-Host @"
NrelayOne Relay Engine Test Runner

Usage: .\test.ps1 [OPTIONS]

Options:
    -Unit        Run unit tests only
    -Integration Run integration tests only  
    -E2E         Run end-to-end tests only
    -Bench       Run performance benchmarks
    -Coverage    Generate test coverage report
    -All         Run all tests and benchmarks (default)
    -Help        Show this help message

Examples:
    .\test.ps1                    # Run all tests
    .\test.ps1 -Unit             # Run only unit tests
    .\test.ps1 -Coverage         # Generate coverage report
    .\test.ps1 -Bench            # Run benchmarks
"@
}

function Test-Prerequisites {
    Write-Host "🔍 Checking prerequisites..." -ForegroundColor Cyan
    
    # Check if Rust is installed
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Cargo not found. Please install Rust." -ForegroundColor Red
        exit 1
    }
    
    # Check if we're in the right directory
    if (!(Test-Path "Cargo.toml")) {
        Write-Host "❌ Cargo.toml not found. Please run this script from the relay-engine directory." -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✅ Prerequisites check passed" -ForegroundColor Green
}

function Run-UnitTests {
    Write-Host "🧪 Running unit tests..." -ForegroundColor Cyan
    
    $env:RUST_LOG = "debug"
    $env:RUST_BACKTRACE = "1"
    
    cargo test --lib -- --test-threads=1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Unit tests failed" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ Unit tests passed" -ForegroundColor Green
    return $true
}

function Run-IntegrationTests {
    Write-Host "🔗 Running integration tests..." -ForegroundColor Cyan
    
    $env:RUST_LOG = "debug"
    $env:RUST_BACKTRACE = "1"
    
    # Run database integration tests
    cargo test database_integration --test database_integration -- --nocapture
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Database integration tests failed" -ForegroundColor Red
        return $false
    }
    
    # Run WebSocket integration tests
    cargo test websocket_integration --test websocket_integration -- --nocapture
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ WebSocket integration tests failed" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ Integration tests passed" -ForegroundColor Green
    return $true
}

function Run-E2ETests {
    Write-Host "🌐 Running end-to-end tests..." -ForegroundColor Cyan
    
    $env:RUST_LOG = "info"
    $env:RUST_BACKTRACE = "1"
    
    cargo test e2e_integration --test e2e_integration -- --nocapture
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ End-to-end tests failed" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ End-to-end tests passed" -ForegroundColor Green
    return $true
}

function Run-Benchmarks {
    Write-Host "📊 Running performance benchmarks..." -ForegroundColor Cyan
    
    cargo bench --bench relay_benchmarks
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Benchmarks failed" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ Benchmarks completed" -ForegroundColor Green
    Write-Host "📈 Benchmark results saved to target/criterion/" -ForegroundColor Blue
    return $true
}

function Generate-Coverage {
    Write-Host "📋 Generating test coverage report..." -ForegroundColor Cyan
    
    # Check if cargo-tarpaulin is installed
    if (!(Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue)) {
        Write-Host "📦 Installing cargo-tarpaulin..." -ForegroundColor Yellow
        cargo install cargo-tarpaulin
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Failed to install cargo-tarpaulin" -ForegroundColor Red
            return $false
        }
    }
    
    $env:RUST_LOG = "error"  # Reduce noise in coverage output
    
    cargo tarpaulin --out Html --output-dir target/coverage --skip-clean --timeout 300
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Coverage generation failed" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✅ Coverage report generated" -ForegroundColor Green
    Write-Host "📊 Coverage report available at target/coverage/tarpaulin-report.html" -ForegroundColor Blue
    return $true
}

function Show-TestSummary {
    param([bool[]]$Results, [string[]]$TestNames)
    
    Write-Host "`n📋 Test Summary" -ForegroundColor Cyan
    Write-Host "=" * 50
    
    $passed = 0
    $failed = 0
    
    for ($i = 0; $i -lt $Results.Length; $i++) {
        if ($Results[$i]) {
            Write-Host "✅ $($TestNames[$i])" -ForegroundColor Green
            $passed++
        } else {
            Write-Host "❌ $($TestNames[$i])" -ForegroundColor Red
            $failed++
        }
    }
    
    Write-Host "=" * 50
    Write-Host "Total: $($passed + $failed) | Passed: $passed | Failed: $failed" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Red" })
    
    if ($failed -eq 0) {
        Write-Host "🎉 All tests passed!" -ForegroundColor Green
    } else {
        Write-Host "💥 Some tests failed. Please check the output above." -ForegroundColor Red
    }
}

# Main execution
if ($Help) {
    Show-Help
    exit 0
}

# Set default behavior
if (-not ($Unit -or $Integration -or $E2E -or $Bench -or $Coverage)) {
    $All = $true
}

Test-Prerequisites

$results = @()
$testNames = @()

# Run selected tests
if ($Unit -or $All) {
    $result = Run-UnitTests
    $results += $result
    $testNames += "Unit Tests"
}

if ($Integration -or $All) {
    $result = Run-IntegrationTests
    $results += $result
    $testNames += "Integration Tests"
}

if ($E2E -or $All) {
    $result = Run-E2ETests
    $results += $result
    $testNames += "End-to-End Tests"
}

if ($Bench -or $All) {
    $result = Run-Benchmarks
    $results += $result
    $testNames += "Performance Benchmarks"
}

if ($Coverage -or $All) {
    $result = Generate-Coverage
    $results += $result
    $testNames += "Coverage Report"
}

# Show summary
if ($results.Count -gt 0) {
    Show-TestSummary -Results $results -TestNames $testNames
    
    # Exit with error code if any tests failed
    if ($results -contains $false) {
        exit 1
    }
}
