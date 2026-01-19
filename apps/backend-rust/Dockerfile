# =============================================================================
# Production Dockerfile for Holiday Wheel Rust Backend
# Multi-stage build with security best practices
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
# Using Rust 1.85+ which supports edition 2024 (required by home crate v0.5.12)
FROM rust:1.85-slim AS builder

# Install build dependencies for OpenSSL (vendored), SQLite, and Swagger UI download
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    perl \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/holiday_wheel*

# Copy actual source code and build-time required files
COPY src ./src
COPY migrations ./migrations
COPY openapi.yaml ./

# Build the application
RUN cargo build --release

# Verify the binary exists
RUN test -f /app/target/release/holiday-wheel-backend

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
# - ca-certificates: for HTTPS connections
# - curl: for healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd --gid 1000 appgroup && \
    useradd --uid 1000 --gid appgroup --shell /bin/bash --create-home appuser

WORKDIR /app

# Create necessary directories with proper ownership
# Support both /app/data (default) and /data (common mount point)
RUN mkdir -p /app/data /app/certs /data && \
    chown -R appuser:appgroup /app /data

# Copy the binary from builder
COPY --from=builder --chown=appuser:appgroup /app/target/release/holiday-wheel-backend /app/holiday-wheel-backend

# Copy sample puzzle data if needed (optional, can be mounted)
COPY --chown=appuser:appgroup holiday-puzzles.json sample-puzzles.json ./

# Ensure binary is executable
RUN chmod +x /app/holiday-wheel-backend

# Switch to non-root user
USER appuser

# Expose the application port
EXPOSE 5000

# Environment variables (defaults, override at runtime)
ENV PORT=5000 \
    HOST=0.0.0.0 \
    RUST_LOG=info \
    RUST_BACKTRACE=0 \
    HOST_CODE=holiday \
    DB_PATH=/app/data/puzzles.db \
    EMAIL_ENABLED=false \
    SSL_ENABLED=false \
    WEBAUTHN_RP_ID=localhost \
    WEBAUTHN_RP_NAME="Holiday Wheel" \
    WEBAUTHN_RP_ORIGIN=http://localhost:5000

# Health check - check if the server responds
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/health || exit 1

# Run the application
CMD ["/app/holiday-wheel-backend"]
