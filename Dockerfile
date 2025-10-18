# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/mail-rs

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY mail-core/Cargo.toml ./mail-core/
COPY mail-smtp/Cargo.toml ./mail-smtp/
COPY mail-builder/Cargo.toml ./mail-builder/

# Copy source code
COPY mail-core/src ./mail-core/src
COPY mail-smtp/src ./mail-smtp/src
COPY mail-builder/src ./mail-builder/src
COPY src ./src

# Build the project
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /usr/src/mail-rs/target/release/mail-rs /usr/local/bin/mail-rs

# Create a non-root user
RUN useradd -m -u 1000 mailrs && \
    chown mailrs:mailrs /usr/local/bin/mail-rs

USER mailrs

WORKDIR /home/mailrs

# Default command
CMD ["mail-rs"]
