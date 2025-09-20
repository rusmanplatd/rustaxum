# Multi-stage build for optimized production image
FROM rust:1.90-bookworm AS builder

# Create app directory
WORKDIR /app

# Copy manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/rustaxum /app/rustaxum

# Create necessary directories
RUN mkdir -p storage/logs storage/uploads && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./rustaxum"]