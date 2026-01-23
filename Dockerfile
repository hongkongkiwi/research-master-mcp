# Build stage
FROM rust:1-alpine AS builder

WORKDIR /build

# Install dependencies for building (perl needed for OpenSSL)
RUN apk add --no-cache musl-dev openssl-dev clang perl

# Add cross-compilation target for musl builds
RUN rustup target add x86_64-unknown-linux-musl

# Copy source and build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/research-master-mcp /usr/local/bin/

# Create downloads directory
RUN mkdir /downloads && chown appuser:appgroup /downloads

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["research-master-mcp"]
CMD ["serve"]
