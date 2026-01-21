# Build stage
# Using Rust 1.92 for Edition 2024 support (matching local version)
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and build.rs (needed for dependency build)
# Note: mock-github-oauth Cargo.toml is needed because it's a workspace member
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY server/build.rs ./server/
COPY mock-github-oauth/Cargo.toml ./mock-github-oauth/

# Create dummy files for dependency caching
RUN mkdir -p server/src mock-github-oauth/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > mock-github-oauth/src/main.rs && \
    echo "" > mock-github-oauth/src/lib.rs

# Build dependencies only (for caching)
# VERGEN_IDEMPOTENT allows build without .git directory (uses placeholder values)
RUN VERGEN_IDEMPOTENT=1 cargo build --release --package arena

# Remove dummy files
RUN rm -rf server/src

# Copy actual source code (only arena, not mock-github-oauth)
COPY server/src ./server/src
COPY server/static ./server/static
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Copy .git directory for vergen to extract real git info in final binary
COPY .git ./.git

# Set SQLX offline mode
ENV SQLX_OFFLINE=true

# Touch the main.rs to ensure rebuild with actual source
RUN touch server/src/main.rs

# Build the application (with real git info from .git)
RUN cargo build --release --package arena

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/arena /app/arena

# Cloud Run uses PORT env var, default to 8080
ENV PORT=8080

# Expose the port
EXPOSE 8080

# Run the application
CMD ["/app/arena"]
