# Multi-stage Dockerfile for a tiny production image.
#
# Stage 1: Build the Rust binary (static files are embedded via rust-embed)
# Stage 2: Copy just the binary into a minimal image

# Build stage
FROM rust:bookworm AS builder

WORKDIR /app

# Copy everything needed for the build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
COPY templates/ templates/

# Build in release mode
RUN cargo build --release

# Runtime stage — minimal image with just the binary
FROM debian:bookworm-slim

# Install CA certificates (needed for HTTPS requests to Sanity API)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/personal-blog .

EXPOSE 3000

CMD ["./personal-blog"]
