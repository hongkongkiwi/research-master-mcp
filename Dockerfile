# Multi-stage build for multi-platform support
# Build with: docker buildx build --platform linux/amd64,linux/arm64 --push -t ghcr.io/hongkongkiwi/research-master-mcp:latest .

# Build stage (one per target platform)
FROM --platform=$TARGETPLATFORM rust:1-alpine AS builder

WORKDIR /build

ARG TARGETARCH

# Install dependencies for building (perl and make needed for OpenSSL)
RUN apk add --no-cache musl-dev openssl-dev clang perl make pkgconfig

RUN case "$TARGETARCH" in \
    amd64) echo "x86_64-unknown-linux-musl" > /tmp/target;; \
    arm64) echo "aarch64-unknown-linux-musl" > /tmp/target;; \
    *) echo "unsupported TARGETARCH: $TARGETARCH" && exit 1;; \
  esac && \
  rustup target add "$(cat /tmp/target)"

# Copy source and build
COPY . .
RUN cargo build --release --target "$(cat /tmp/target)"

# Runtime stage
FROM --platform=$TARGETPLATFORM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binary for the current target architecture
COPY --from=builder /build/target/*-unknown-linux-musl/release/research-master-mcp /usr/local/bin/research-master-mcp

# Create downloads directory
RUN mkdir /downloads && chown appuser:appgroup /downloads

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["research-master-mcp"]
CMD ["serve"]
