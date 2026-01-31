# =============================================================================
# ConFuse API Backend - Multi-stage Dockerfile
# =============================================================================
# Build: podman build -t confuse/api-backend:latest .
# Run:   podman run -p 8088:8088 --env-file .env confuse/api-backend:latest
# =============================================================================

# Stage 1: Build - Use latest stable Rust
FROM rust:1.84.0-slim AS builder

WORKDIR /app

# Install build dependencies for OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    libsasl2-dev \
    librdkafka-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy all source code (uses .dockerignore)
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime - Use trixie for GLIBC 2.38+ compatibility
FROM docker.io/debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    librdkafka1 \
    libsasl2-2 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 confuse

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api-backend /usr/local/bin/api-backend
RUN chmod +x /usr/local/bin/api-backend

# Switch to non-root user
USER confuse

# Environment defaults
ENV PORT=8088
ENV RUST_LOG=info,api_backend=debug
ENV OTEL_SERVICE_NAME=api-backend

# Expose port
EXPOSE 8088

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8088/health || exit 1

# Run
CMD ["api-backend"]
