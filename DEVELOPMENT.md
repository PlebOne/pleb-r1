# Development Guide - Pleb-R1

## 🚀 Getting Started

This guide will help you set up and run the Pleb-R1 relay and authentication system for development.

## 📋 Prerequisites

- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Node.js** (v18+) - [Install Node.js](https://nodejs.org/)
- **Git** - [Install Git](https://git-scm.com/)
- **PowerShell** (Windows) or **Bash** (Unix-like systems)

## 🏗️ Project Architecture

### Services Overview

1. **Main Relay** (`services/relay-engine/src/main.rs`)
   - Production Nostr relay server
   - WebSocket protocol implementation
   - Full database connectivity (SQLite/PostgreSQL)
   - Prometheus metrics endpoint

2. **Development Authentication Server** (`services/relay-engine/src/dev_main.rs`)
   - Simplified server for frontend development
   - REST API for user authentication
   - CORS enabled for local development
   - Mock data for testing

3. **Public Landing Page** (`services/community-web/landing.html`)
   - Marketing website for public users
   - User registration and login forms
   - Pricing tier display
   - Real API integration

4. **React Community Platform** (`services/community-web/src/`)
   - Advanced community management interface
   - Currently pending npm dependency resolution

## 🔧 Development Setup

### 1. Clone Repository
```bash
git clone https://github.com/PlebOne/pleb-r1.git
cd pleb-r1
```

### 2. Set Up Rust Environment
```bash
cd services/relay-engine

# Build all components
cargo build

# Run tests to verify setup
cargo test
```

### 3. Database Setup

#### Option A: SQLite (Recommended for Development)
```bash
# Create database directory and file
mkdir -p data
touch data/relay.db

# Set environment variable (optional, has defaults)
export DATABASE_URL="sqlite:./data/relay.db"
```

#### Option B: PostgreSQL (Production)
```bash
# Install PostgreSQL and create database
createdb pleb_one

# Set environment variable
export DATABASE_URL="postgresql://username:password@localhost:5432/pleb_one"
```

### 4. Environment Configuration

Create a `.env` file in `services/relay-engine/`:

```env
# Database Configuration
DATABASE_URL=sqlite:./data/relay.db
# DATABASE_URL=postgresql://username:password@localhost:5432/pleb_r1

# Relay Configuration
RELAY_NAME="Pleb-R1 Development Relay"
RELAY_DESCRIPTION="A community-owned Nostr relay (Development)"
RELAY_PUBKEY="your_relay_pubkey_here"
RELAY_CONTACT="dev@pleb-r1.com"

# Server Configuration
PORT=8080

# Development Settings
RUST_LOG=info
```

## 🚀 Running the Services

### Development Authentication Server (Recommended for Frontend Work)

```bash
cd services/relay-engine
cargo run --bin dev-server
```

**Features:**
- Runs on `http://localhost:8080`
- Authentication API endpoints
- CORS enabled for frontend development
- Simplified dependencies (no metrics complications)
- Mock data responses

**Endpoints:**
- `POST /api/auth/signup` - User registration
- `POST /api/auth/login` - User authentication
- `GET /api/metrics/*` - Various metrics endpoints

### Production Relay Server

```bash
cd services/relay-engine
cargo run --bin relay-engine
```

**Features:**
- Full Nostr WebSocket protocol
- Complete database integration
- Prometheus metrics on `/metrics`
- Production-ready configuration

### Public Landing Page

Simply open `services/community-web/landing.html` in your web browser, or serve it locally:

```bash
# Simple HTTP server
cd services/community-web
python -m http.server 3000
# Then visit http://localhost:3000/landing.html
```

**Features:**
- Professional marketing design
- Three pricing tiers (Community/Pro/Enterprise)
- Working signup/login forms
- Real API integration with authentication server

## 🧪 Testing the System

### 1. Test Authentication API

Start the development server and test the endpoints:

```bash
# Start dev server
cargo run --bin dev-server

# Test signup (in another terminal)
curl -X POST http://localhost:8080/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "firstName": "Test",
    "lastName": "User",
    "email": "test@example.com",
    "plan": "Community"
  }'

# Test login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpass"
  }'
```

### 2. Test Landing Page Registration

1. Start the development server: `cargo run --bin dev-server`
2. Open `services/community-web/landing.html` in your browser
3. Click "Get Started" and fill out the registration form
4. Check the browser console for API responses

### 3. Test Production Relay

```bash
# Start production relay
cargo run --bin relay-engine

# Test WebSocket connection (using wscat or similar)
wscat -c ws://localhost:8080

# Send a Nostr message
["EVENT", {"id": "test", "pubkey": "...", "created_at": 1693926000, "kind": 1, "tags": [], "content": "Hello Nostr!", "sig": "..."}]
```

## 🐛 Troubleshooting

### Common Issues

1. **Database Connection Errors**
   ```bash
   # For SQLite, ensure file exists:
   touch services/relay-engine/data/relay.db
   
   # Check permissions
   ls -la services/relay-engine/data/
   ```

2. **Port Already in Use**
   ```bash
   # Find process using port 8080
   netstat -ano | findstr :8080  # Windows
   lsof -i :8080                 # Unix
   
   # Kill the process or change PORT in .env
   ```

3. **Cargo Build Warnings**
   - Unused import warnings are normal and don't affect functionality
   - Run `cargo fix --bin dev-server` to auto-fix some warnings

4. **npm Dependencies (React Platform)**
   - Currently a known issue
   - Use the landing page and authentication server for testing
   - Resolution pending for full React development environment

### Build Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Update Rust toolchain
rustup update

# Check for conflicting dependencies
cargo tree
```

### Database Issues

```bash
# Reset SQLite database
rm services/relay-engine/data/relay.db
touch services/relay-engine/data/relay.db

# Check PostgreSQL connection
psql $DATABASE_URL -c "SELECT 1;"
```

## 📁 File Structure Details

### Key Files

- `services/relay-engine/src/main.rs` - Production relay entry point
- `services/relay-engine/src/dev_main.rs` - Development server with auth API
- `services/relay-engine/src/config.rs` - Configuration management
- `services/relay-engine/Cargo.toml` - Rust dependencies and metadata
- `services/community-web/landing.html` - Public landing page
- `services/community-web/package.json` - Node.js dependencies (React platform)

### Generated Files

- `target/` - Rust build artifacts (gitignored)
- `services/relay-engine/data/` - SQLite database files
- `node_modules/` - npm dependencies (gitignored)

## 🔄 Development Workflow

### Making Changes

1. **Backend Changes** (Rust)
   ```bash
   cd services/relay-engine
   # Edit src/dev_main.rs or src/main.rs
   cargo run --bin dev-server  # Test changes
   ```

2. **Frontend Changes** (HTML/CSS/JS)
   ```bash
   # Edit services/community-web/landing.html
   # Refresh browser to see changes
   ```

3. **Configuration Changes**
   ```bash
   # Edit .env file
   # Restart server to apply changes
   ```

### Running Tests

```bash
cd services/relay-engine
cargo test                    # Run all tests
cargo test --bin dev-server   # Test specific binary
```

### Code Formatting

```bash
cargo fmt                     # Format Rust code
cargo clippy                  # Lint and suggestions
```

## 🚀 Next Steps

### Immediate Development Priorities

1. **Resolve npm Dependencies** - Fix React platform development environment
2. **Database Integration** - Connect authentication to persistent storage
3. **Email Verification** - Add email confirmation to registration
4. **Testing** - Add comprehensive test coverage

### Production Deployment

1. **Environment Setup** - Configure production environment variables
2. **Database Migration** - Set up PostgreSQL with proper schema
3. **SSL/TLS** - Configure HTTPS and WSS protocols
4. **Monitoring** - Set up logging and metrics collection

## 📞 Getting Help

- **Issues**: Report bugs or ask questions in GitHub Issues
- **Documentation**: Check README.md and inline code comments
- **Community**: Join our Discord for real-time discussion

---

**Happy coding! 🚀**
