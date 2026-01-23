# Build stage
FROM rust:1-alpine AS builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu

WORKDIR /build

# Install dependencies for building
RUN apk add --no-cache musl-dev openssl-dev

# Add cross-compilation target for glibc builds
RUN rustup target add ${CARGO_BUILD_TARGET}

# Copy source and build
COPY . .
RUN cargo build --release --target ${CARGO_BUILD_TARGET}

# Runtime stage
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binary from builder
COPY --from=builder /build/target/${CARGO_BUILD_TARGET}/release/research-master-mcp /usr/local/bin/

# Create downloads directory
RUN mkdir /downloads && chown appuser:appgroup /downloads

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["research-master-mcp"]
CMD ["serve"]
