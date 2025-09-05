# Multi-stage build for production optimization
FROM golang:1.21-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache git ca-certificates tzdata

# Copy go mod files
COPY go.mod go.sum ./
RUN go mod download

# Copy source code
COPY . .

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o blossom-server .

# Development stage
FROM golang:1.21-alpine as development

WORKDIR /app

# Install development dependencies
RUN apk add --no-cache git ca-certificates tzdata curl

# Install air for hot reloading
RUN go install github.com/cosmtrek/air@latest

# Copy go mod files and download dependencies
COPY go.mod go.sum ./
RUN go mod download

# Copy source code
COPY . .

EXPOSE 8004
EXPOSE 9095

# Use air for hot reloading in development
CMD ["air", "-c", ".air.toml"]

# Production stage
FROM alpine:3.18 as production

WORKDIR /app

# Install runtime dependencies
RUN apk --no-cache add ca-certificates tzdata curl
RUN addgroup -g 1001 -S blossom && adduser -u 1001 -S blossom -G blossom

# Copy the binary from builder stage
COPY --from=builder /app/blossom-server /usr/local/bin/blossom-server

# Switch to non-root user
USER blossom

EXPOSE 8004
EXPOSE 9095

CMD ["blossom-server"]
