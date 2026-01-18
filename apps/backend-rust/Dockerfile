# Build stage
FROM docker.io/library/rust:1.87-bookworm AS builder

WORKDIR /app

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src target/release/deps/holiday_wheel*

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM docker.io/library/debian:bookworm-slim

# Install required runtime libraries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/holiday-wheel-backend /app/holiday-wheel-backend

# Create data directory
RUN mkdir -p /app/data

EXPOSE 5000

# Environment variables
ENV PORT=5000
ENV RUST_LOG=info
ENV HOST_CODE=holiday
ENV DATABASE_URL=sqlite:/app/data/puzzles.db
ENV EMAIL_ENABLED=false

CMD ["/app/holiday-wheel-backend"]
