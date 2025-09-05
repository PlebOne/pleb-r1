# Docker Deployment Guide

This guide covers deploying Pleb.One using Docker containers for easy development and production deployment.

## 🚀 Quick Start

### Development Environment

```bash
# Start development environment
./docker-manage.sh dev

# Or on Windows:
./docker-manage.ps1 dev
```

This starts:
- **Development Relay** with SQLite on port 8080
- **Frontend** on port 3000
- **Database Viewer** on port 8081

### Production Environment

```bash
# Copy environment template
cp .env.docker .env

# Edit configuration
nano .env

# Start production stack
./docker-manage.sh prod
```

## 🏗️ Architecture

### Services Overview

| Service | Purpose | Port | Health Check |
|---------|---------|------|--------------|
| `relay-engine` | Main Nostr relay | 8080 | `/metrics` |
| `community-web` | Frontend interface | 80/443 | `/health` |
| `postgres` | Database | 5432 | `pg_isready` |
| `redis` | Caching | 6379 | `redis-cli ping` |
| `prometheus` | Metrics collection | 9090 | Built-in |
| `grafana` | Metrics visualization | 3000 | Built-in |

### Docker Images

1. **Relay Engine** (`services/relay-engine/Dockerfile`)
   - Multi-stage Rust build
   - Optimized for production
   - Supports both binaries: `relay-engine` and `dev-server`

2. **Frontend** (`services/community-web/Dockerfile`)
   - Node.js build stage
   - Nginx serving stage
   - Includes landing page and React app

## 📋 Configuration

### Environment Variables

Create `.env` from `.env.docker` template:

```env
# Database passwords
POSTGRES_PASSWORD=secure_password_here
REDIS_PASSWORD=redis_password_here

# Relay configuration
RELAY_PUBKEY=your_relay_public_key
RELAY_CONTACT=admin@yourdomain.com

# Monitoring
GRAFANA_PASSWORD=grafana_admin_password

# SSL (for production)
DOMAIN=relay.yourdomain.com
EMAIL=admin@yourdomain.com
```

### Database Configuration

**Development (SQLite):**
```yaml
environment:
  DATABASE_URL: sqlite:/app/data/dev.db
```

**Production (PostgreSQL):**
```yaml
environment:
  DATABASE_URL: postgresql://pleb_user:password@postgres:5432/pleb_one
```

## 🛠️ Docker Compose Files

### 1. Production (`docker-compose.yml`)

Full production stack with:
- PostgreSQL database
- Redis caching
- Main relay engine
- Frontend with nginx
- Optional monitoring stack

### 2. Development (`docker-compose.dev.yml`)

Simplified development environment:
- SQLite database
- Development server with auth API
- Simple nginx frontend
- Fast iteration and debugging

### 3. Profiles

Use Docker Compose profiles for optional services:

```bash
# Start with monitoring
docker-compose --profile monitoring up -d

# Start development tools
docker-compose --profile development up -d

# Start database tools
docker-compose --profile db-tools up -d
```

## 🚀 Deployment Commands

### Management Scripts

**Windows PowerShell:**
```powershell
./docker-manage.ps1 dev        # Development environment
./docker-manage.ps1 prod       # Production environment
./docker-manage.ps1 monitoring # With monitoring stack
./docker-manage.ps1 stop       # Stop all services
./docker-manage.ps1 clean      # Clean up everything
./docker-manage.ps1 logs       # View logs
./docker-manage.ps1 status     # Service status
```

**Unix/Linux/macOS:**
```bash
./docker-manage.sh dev         # Development environment
./docker-manage.sh prod        # Production environment
./docker-manage.sh monitoring  # With monitoring stack
./docker-manage.sh stop        # Stop all services
./docker-manage.sh clean       # Clean up everything
./docker-manage.sh logs        # View logs
./docker-manage.sh status      # Service status
./docker-manage.sh build       # Build images
```

### Manual Docker Compose

```bash
# Development
docker-compose -f docker-compose.dev.yml up -d

# Production
docker-compose up -d

# With monitoring
docker-compose --profile monitoring up -d

# Stop and remove
docker-compose down -v
```

## 🔍 Monitoring and Debugging

### Health Checks

All services include health checks:

```bash
# Check service health
docker-compose ps

# View health check logs
docker inspect --format='{{json .State.Health}}' pleb-one-relay
```

### Logs

```bash
# All logs
docker-compose logs -f

# Specific service
docker-compose logs -f relay-engine

# Last 100 lines
docker-compose logs --tail=100 relay-engine
```

### Metrics

**Prometheus Metrics:**
- Relay: `http://localhost:8080/metrics`
- Prometheus UI: `http://localhost:9090`

**Grafana Dashboards:**
- UI: `http://localhost:3000`
- Default login: `admin` / `[GRAFANA_PASSWORD]`

## 🗄️ Data Persistence

### Volumes

```yaml
volumes:
  postgres_data:    # PostgreSQL data
  redis_data:       # Redis persistence
  relay_data:       # Relay application data
  relay_logs:       # Application logs
  prometheus_data:  # Metrics data
  grafana_data:     # Dashboard configs
```

### Backup

```bash
# Backup database
docker-compose exec postgres pg_dump -U pleb_user pleb_one > backup.sql

# Backup volumes
docker run --rm -v pleb_one_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres_backup.tar.gz -C /data .
```

### Restore

```bash
# Restore database
docker-compose exec -T postgres psql -U pleb_user pleb_one < backup.sql

# Restore volume
docker run --rm -v pleb_one_postgres_data:/data -v $(pwd):/backup alpine tar xzf /backup/postgres_backup.tar.gz -C /data
```

## 🔒 Production Security

### SSL/TLS Setup

1. **Let's Encrypt with Docker:**
```yaml
services:
  nginx-proxy:
    image: nginxproxy/nginx-proxy
    ports:
      - "80:80"
      - "443:443"
    environment:
      DEFAULT_HOST: ${DOMAIN}
    volumes:
      - certs:/etc/nginx/certs
      - /var/run/docker.sock:/tmp/docker.sock:ro

  letsencrypt:
    image: nginxproxy/acme-companion
    environment:
      DEFAULT_EMAIL: ${EMAIL}
    volumes:
      - certs:/etc/nginx/certs
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

2. **Service Configuration:**
```yaml
environment:
  VIRTUAL_HOST: ${DOMAIN}
  LETSENCRYPT_HOST: ${DOMAIN}
  LETSENCRYPT_EMAIL: ${EMAIL}
```

### Security Best Practices

- **Secrets Management:** Use Docker secrets for production
- **Network Isolation:** Services communicate via internal network
- **Non-root Users:** All containers run as non-root
- **Resource Limits:** Set memory and CPU limits
- **Health Checks:** Monitor service health

## 🚨 Troubleshooting

### Common Issues

1. **Port Conflicts:**
```bash
# Check what's using port
netstat -tulpn | grep :8080

# Use different ports
docker-compose up -d -p 8081:8080
```

2. **Database Connection:**
```bash
# Check database logs
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U pleb_user -d pleb_one -c "SELECT 1;"
```

3. **Build Issues:**
```bash
# Clean build
docker-compose build --no-cache

# Check build logs
docker-compose build relay-engine 2>&1 | tee build.log
```

4. **Memory Issues:**
```bash
# Check resource usage
docker stats

# Set memory limits in docker-compose.yml
deploy:
  resources:
    limits:
      memory: 512M
```

### Debug Mode

**Enable Debug Logging:**
```yaml
environment:
  RUST_LOG: debug
```

**Development Container:**
```bash
# Run interactively
docker-compose exec relay-engine /bin/bash

# Or override command
docker-compose run --rm relay-engine /bin/bash
```

## 📊 Performance Tuning

### Database Optimization

**PostgreSQL Configuration:**
```yaml
command: |
  postgres
  -c shared_preload_libraries=pg_stat_statements
  -c max_connections=200
  -c shared_buffers=256MB
  -c effective_cache_size=1GB
```

### Application Tuning

**Rust Relay Configuration:**
```yaml
environment:
  RUST_LOG: info  # Reduce logging overhead
  DATABASE_POOL_SIZE: 10
  MAX_CONNECTIONS: 1000
```

### Resource Limits

```yaml
deploy:
  resources:
    limits:
      cpus: '2'
      memory: 1G
    reservations:
      cpus: '0.5'
      memory: 512M
```

## 🔄 Updates and Maintenance

### Updating Services

```bash
# Pull latest images
docker-compose pull

# Rebuild custom images
docker-compose build

# Rolling update
docker-compose up -d --force-recreate
```

### Database Migrations

```bash
# Run migrations (when available)
docker-compose exec relay-engine ./migrate

# Or manually
docker-compose exec postgres psql -U pleb_user -d pleb_one -f /migrations/001_initial.sql
```

---

## 📞 Support

- **Docker Issues:** Check logs and health checks first
- **Configuration:** Review environment variables
- **Performance:** Monitor with Prometheus/Grafana
- **Updates:** Follow semantic versioning for safe updates

**Happy Dockerizing! 🐳**
