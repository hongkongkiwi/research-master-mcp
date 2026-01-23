# Runtime-only image: copy prebuilt musl binaries
FROM --platform=$TARGETPLATFORM alpine:3.19 AS runtime

ARG TARGETARCH

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binary for the current target architecture
COPY docker-bin/research-master-mcp-${TARGETARCH} /usr/local/bin/research-master-mcp

# Create downloads directory
RUN mkdir /downloads && chown appuser:appgroup /downloads

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["research-master-mcp"]
CMD ["serve"]
