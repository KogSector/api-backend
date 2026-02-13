# =============================================================================
# ConFuse API Backend - Multi-stage Dockerfile
# =============================================================================
# Build from workspace root: podman build -f api-backend/Dockerfile -t confuse/api-backend:latest .
# Run: podman run -p 8088:8088 --env-file .env confuse/api-backend:latest
# =============================================================================

# Stage 1: Build - Use latest stable Rust
FROM rust:1.92.0-slim AS builder

WORKDIR /workspace

# Install build dependencies for OpenSSL and Kafka
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    libsasl2-dev \
    librdkafka-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy shared middleware library
COPY ../shared-middleware ./shared-middleware

# Copy api-backend source
COPY api-backend ./api-backend

WORKDIR /workspace/api-backend

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
    dumb-init \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 confuse

WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/api-backend/target/release/api-backend /usr/local/bin/api-backend
RUN chmod +x /usr/local/bin/api-backend

# Switch to non-root user
USER confuse

# Environment defaults
ENV PORT=8000
ENV RUST_LOG=info,api_backend=debug
ENV OTEL_SERVICE_NAME=api-backend

# Expose port
EXPOSE 8000

# Health check optimized for Azure Container Apps
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Use dumb-init as PID 1 for proper signal handling
ENTRYPOINT ["dumb-init", "--"]

# Run
CMD ["api-backend"]
