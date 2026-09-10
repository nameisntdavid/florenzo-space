# Multi-stage Dockerfile for a tiny production image.
#
# Stage 1: Build the Rust binary
# Stage 2: Copy just the binary into a minimal image
#
# This keeps the final image ~20MB instead of ~1GB (no Rust compiler included).

# Build stage
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Copy manifests first (for Docker layer caching).
# If only Cargo.toml changes, Docker reuses the cached dependency layer.
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to pre-build dependencies
# This step caches `cargo build` of dependencies, so they aren't rebuilt
# every time you change your source code.
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src/ src/
COPY templates/ templates/
COPY static/ static/

# Touch main.rs so cargo rebuilds it (not the cached dummy)
RUN touch src/main.rs && cargo build --release

# Runtime stage — minimal image with just the binary
FROM debian:bookworm-slim

# Install CA certificates (needed for HTTPS requests to Sanity API)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/personal-blog .

# Copy static files and templates (needed at runtime for Askama)
COPY --from=builder /app/templates/ templates/
COPY --from=builder /app/static/ static/

EXPOSE 3000

CMD ["./personal-blog"]
