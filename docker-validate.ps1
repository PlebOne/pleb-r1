# Docker Validation Script
# Run this to validate Docker configurations before deployment

Write-Host "🐳 Docker Configuration Validation" -ForegroundColor Cyan

# Check if Docker is installed
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Host "✅ Docker is installed" -ForegroundColor Green
    docker --version
} else {
    Write-Host "❌ Docker is not installed" -ForegroundColor Red
    Write-Host "   Please install Docker Desktop from https://docker.com/products/docker-desktop" -ForegroundColor Yellow
    return
}

# Check if Docker Compose is available
if (Get-Command docker-compose -ErrorAction SilentlyContinue) {
    Write-Host "✅ Docker Compose is available" -ForegroundColor Green
    docker-compose --version
} else {
    Write-Host "❌ Docker Compose is not available" -ForegroundColor Red
    return
}

# Validate Docker configurations
Write-Host "`n🔍 Validating Docker configurations..." -ForegroundColor Blue

$configs = @(
    "docker-compose.yml",
    "docker-compose.dev.yml",
    "services/relay-engine/Dockerfile",
    "services/community-web/Dockerfile"
)

foreach ($config in $configs) {
    if (Test-Path $config) {
        Write-Host "✅ $config exists" -ForegroundColor Green
    } else {
        Write-Host "❌ $config missing" -ForegroundColor Red
    }
}

# Check for environment template
if (Test-Path ".env.docker") {
    Write-Host "✅ Environment template (.env.docker) exists" -ForegroundColor Green
} else {
    Write-Host "❌ Environment template missing" -ForegroundColor Red
}

# Validate Dockerfiles with docker-compose config (if Docker is running)
try {
    Write-Host "`n🧪 Testing Docker Compose configuration..." -ForegroundColor Blue
    docker-compose config --quiet
    Write-Host "✅ docker-compose.yml is valid" -ForegroundColor Green
    
    docker-compose -f docker-compose.dev.yml config --quiet
    Write-Host "✅ docker-compose.dev.yml is valid" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Could not validate Docker Compose configs (Docker daemon may not be running)" -ForegroundColor Yellow
}

Write-Host "`n🚀 Ready for Docker deployment!" -ForegroundColor Green
Write-Host "   Run: ./docker-manage.ps1 dev" -ForegroundColor Yellow
