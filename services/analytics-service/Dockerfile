# Multi-stage build for production optimization
FROM rust:1.73-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY services/analytics-service/Cargo.toml ./services/analytics-service/
COPY services/nostr-types ./services/nostr-types
COPY services/storage-layer ./services/storage-layer
COPY services/config-manager ./services/config-manager

# Create dummy main files for dependency caching
RUN mkdir -p services/analytics-service/src && \
    echo "fn main() {}" > services/analytics-service/src/main.rs

# Build dependencies (this layer will be cached)
RUN cd services/analytics-service && cargo build --release

# Copy actual source code
COPY services/analytics-service/src ./services/analytics-service/src

# Build the actual application
RUN cd services/analytics-service && cargo build --release

# Development stage
FROM rust:1.73-slim as development

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the workspace
COPY . .

# Install cargo-watch for development
RUN cargo install cargo-watch

EXPOSE 8003
EXPOSE 9094

CMD ["cargo", "watch", "-x", "run", "--workdir", "services/analytics-service"]

# Production stage
FROM debian:bookworm-slim as production

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/analytics-service /usr/local/bin/analytics-service

# Create non-root user
RUN groupadd -r analytics && useradd -r -g analytics analytics
USER analytics

EXPOSE 8003
EXPOSE 9094

CMD ["analytics-service"]
