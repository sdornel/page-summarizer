# While piping a curl command directly into a shell to install Rust (via rustup) is common, it inherently
# carries risk because it executes remote code without additional verification. In a highly secure environment,
# you might want to verify checksums or signatures for the installation script to further ensure its integrity.

# also consider pinning version

# ──────────────────────────────────────────────────────────
# 1. Builder Stage (Rust + Node.js)
# ──────────────────────────────────────────────────────────
FROM docker.io/rust:1.85-bookworm AS builder

# Install essential build tools
RUN apt-get update && apt-get install -y \
    curl \
    gnupg \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g puppeteer@24.4.0 \
    && rm -rf /var/lib/apt/lists/*

# Configure Rust cache
WORKDIR /build
COPY . .
RUN cargo build --release && \
    mv target/release/scraper /scraper-bin

# ──────────────────────────────────────────────────────────
# 2. Runtime Stage (Security-Hardened)
# ──────────────────────────────────────────────────────────
FROM docker.io/debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    python3 \
    python3-pip \
    python3-venv \
    libssl3 \
    chromium \
    libnss3 libxss1 libasound2 libxtst6 libgtk-3-0 libgbm1 \
    && rm -rf /var/lib/apt/lists/*

# Copy artifacts
COPY --from=builder /scraper-bin /app/scraper
COPY src/run.py /app/src/

COPY src/scrapers-js/ /app/src/scrapers-js/

COPY requirements.txt /app/

# # Set executable permission on the binary
# RUN chmod 755 /app/scraper

# Create secure user and directories
RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /bin/false appuser && \
    mkdir -p /app/output && \
    chown -R appuser:appgroup /app

# Copy entire project (filtered by .dockerignore)
# COPY --from=builder --chown=appuser:appgroup /build /app

# Security hardening and permissions
RUN find /app -type d -exec chmod 755 {} + \
    && find /app -type f -exec chmod 644 {} + \
    && chmod 755 /app/src/run.py \
    && chmod 750 /app/output \
    && rm -rf /app/node_modules /app/target

# Apply executable permission for the binary
RUN chmod 755 /app/scraper
# Apply executable permission for the backup page opener
RUN chmod 755 /app/src/scrapers-js/backup-page-opener.js

# Switch to non-root user and set working directory
USER appuser
WORKDIR /app

# Create and activate a virtual environment, then install Python dependencies
RUN python3 -m venv venv \
    && . venv/bin/activate \
    && pip install --no-cache-dir -r requirements.txt

# Update PATH to use the virtual environment's Python
ENV PATH="/app/venv/bin:$PATH"

VOLUME ["/app/output"]
ENTRYPOINT ["python3", "/app/src/run.py"]