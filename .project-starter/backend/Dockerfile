# syntax=docker/dockerfile:1.4

#=============================================================================
# Chef stage - Generate recipe for dependency caching
#=============================================================================
FROM rust:1.75-alpine AS chef

# Install cargo-chef and build essentials
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static && \
    cargo install cargo-chef

WORKDIR /app

#=============================================================================
# Planner stage - Analyze dependencies
#=============================================================================
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Generate dependency recipe
RUN cargo chef prepare --recipe-path recipe.json

#=============================================================================
# Builder stage - Build with aggressive optimizations
#=============================================================================
FROM chef AS builder

# Install mold linker (5-10x faster linking) and sccache
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    clang \
    mold \
    git

# Install sccache for build caching
RUN cargo install sccache --locked

# Copy recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies with cache mounts (this layer is cached!)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    RUSTC_WRAPPER=sccache \
    CARGO_INCREMENTAL=0 \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build application with aggressive optimizations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    RUSTC_WRAPPER=sccache \
    RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=mold -C codegen-units=1 -C opt-level=3" \
    CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_RELEASE_LTO=true \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    cargo build --release --locked && \
    strip target/release/project-starter-api

# Show sccache stats
RUN sccache --show-stats || true

#=============================================================================
# Runtime stage - Minimal production image
#=============================================================================
FROM alpine:3.19 AS runtime

# Install only runtime dependencies (no build tools!)
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Create non-root user for security
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

WORKDIR /app

# Copy binary and migrations
COPY --from=builder /app/target/release/project-starter-api /app/project-starter-api
COPY --from=builder /app/migrations /app/migrations

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/project-starter-api", "--health-check"] || exit 1

# Run the application
CMD ["/app/project-starter-api"]
