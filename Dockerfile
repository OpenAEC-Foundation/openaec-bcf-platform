# Stage 1: Build the Rust binary
FROM rust:1.86-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/bcf-core/Cargo.toml crates/bcf-core/Cargo.toml
COPY crates/bcf-server/Cargo.toml crates/bcf-server/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p crates/bcf-core/src crates/bcf-server/src && \
    echo "pub mod types;" > crates/bcf-core/src/lib.rs && \
    echo "pub struct Dummy;" > crates/bcf-core/src/types.rs && \
    echo "fn main() {}" > crates/bcf-server/src/main.rs

# Copy migrations (needed for sqlx compile-time checks)
COPY migrations/ migrations/

# Build dependencies only (cached layer)
RUN cargo build --release -p bcf-server 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Touch source files to invalidate cache, then build
RUN touch crates/bcf-core/src/lib.rs crates/bcf-server/src/main.rs && \
    cargo build --release -p bcf-server

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r bcf && useradd -r -g bcf -d /app bcf

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/bcf-server /app/bcf-server

# Copy migrations for runtime migration
COPY migrations/ /app/migrations/

# Create data directory for snapshots
RUN mkdir -p /app/data/snapshots && chown -R bcf:bcf /app

USER bcf

EXPOSE 3000

ENV RUST_LOG=bcf_server=info,tower_http=info

CMD ["/app/bcf-server"]
