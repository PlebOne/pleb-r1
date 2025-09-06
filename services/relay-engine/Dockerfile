# Multi-stage build for production-ready Rust binaries
FROM rust:1.89 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY services/relay-engine/Cargo.toml ./services/relay-engine/

# Create dummy source files to cache dependencies
RUN mkdir -p services/relay-engine/src \
    && mkdir -p services/relay-engine/benches \
    && echo "fn main() {}" > services/relay-engine/src/main.rs \
    && echo "fn main() {}" > services/relay-engine/src/dev_main.rs \
    && echo "" > services/relay-engine/src/lib.rs \
    && echo "fn main() {}" > services/relay-engine/benches/relay_benchmarks.rs

# Build dependencies (this layer will be cached)
WORKDIR /usr/src/app/services/relay-engine
RUN cargo build --release --bin relay-engine
RUN cargo build --release --bin dev-server

# Copy actual source code
COPY services/ /usr/src/app/services/

# Build the actual application
RUN touch src/main.rs src/dev_main.rs src/lib.rs \
    && cargo build --release --bin relay-engine \
    && cargo build --release --bin dev-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r appuser && useradd -r -g appuser -m -d /app appuser

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /usr/src/app/target/release/relay-engine /app/
COPY --from=builder /usr/src/app/target/release/dev-server /app/

# Create data directory for SQLite and logs
RUN mkdir -p /app/data /app/logs && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Environment variables with defaults
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app/data/relay.db
ENV PORT=8080
ENV RELAY_NAME="Pleb.One Relay"
ENV RELAY_DESCRIPTION="A community-owned Nostr relay"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${PORT}/metrics || exit 1

# Default command (can be overridden)
CMD ["./relay-engine"]
