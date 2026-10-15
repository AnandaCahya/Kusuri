# ========================================================
# Stage 1: Build binary using official Rust bookworm-slim
# ========================================================
FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy migrations and source code
COPY migrations ./migrations
COPY src ./src

# Build the release binary
RUN cargo build --release

# ========================================================
# Stage 2: Minimal runtime image
# ========================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime utilities
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create application user and upload directory
RUN useradd -m -u 1000 -s /bin/bash kusuri && \
    mkdir -p /app/data/uploads && \
    chown -R kusuri:kusuri /app

# Copy binary from builder
COPY --from=builder /app/target/release/backend /usr/local/bin/kusuri-backend

USER kusuri

# Default environment variables
ENV HOST=0.0.0.0 \
    PORT=8080 \
    UPLOAD_DIR=/app/data/uploads \
    RUST_LOG=info,backend=debug,tower_http=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

ENTRYPOINT ["/usr/local/bin/kusuri-backend"]
