# Multi-stage build for multi-platform support
# Build with: docker buildx build --platform linux/amd64,linux/arm64 --push -t ghcr.io/hongkongkiwi/research-master-mcp:latest .

# Build stage for x86_64
FROM --platform=$BUILDPLATFORM rust:1-alpine AS builder-amd64

WORKDIR /build

# Install dependencies for building (perl and make needed for OpenSSL)
RUN apk add --no-cache musl-dev openssl-dev clang perl make

# Add cross-compilation target
RUN rustup target add x86_64-unknown-linux-musl

# Copy source and build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Build stage for aarch64
FROM --platform=$BUILDPLATFORM rust:1-alpine AS builder-arm64

WORKDIR /build

# Install dependencies for building (perl and make needed for OpenSSL)
RUN apk add --no-cache musl-dev openssl-dev clang perl make

# Add cross-compilation target and install cross-compiler
RUN rustup target add aarch64-unknown-linux-musl && \
    apk add --no-cache gcc-cross-embedded

# Copy source and build
COPY . .
RUN cargo build --release --target aarch64-unknown-linux-musl

# Runtime stage
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binaries from both architectures
COPY --from=builder-amd64 /build/target/x86_64-unknown-linux-musl/release/research-master-mcp /usr/local/bin/research-master-mcp-amd64
COPY --from=builder-arm64 /build/target/aarch64-unknown-linux-musl/release/research-master-mcp /usr/local/bin/research-master-mcp-arm64

# Create a wrapper script that selects the right binary at runtime
RUN echo '#!/bin/sh \
if [ "$(uname -m)" = "aarch64" ]; then \
    exec /usr/local/bin/research-master-mcp-arm64 "$@"; \
else \
    exec /usr/local/bin/research-master-mcp-amd64 "$@"; \
fi' > /usr/local/bin/research-master-mcp && \
    chmod +x /usr/local/bin/research-master-mcp

# Create downloads directory
RUN mkdir /downloads && chown appuser:appgroup /downloads

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["research-master-mcp"]
CMD ["serve"]
