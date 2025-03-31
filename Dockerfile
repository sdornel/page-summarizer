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
    # && npm install -g puppeteer@24.4.0 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy Node.js files and install Puppeteer locally
COPY package.json package-lock.json* ./
ENV PUPPETEER_CACHE_DIR=/build/.puppeteer-cache
RUN npm install --omit=dev \
    && npx puppeteer browsers install chrome

# Copy the rest of the project (after installing Node deps)
COPY . .

# Build Rust project
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
    nodejs \
    && rm -rf /var/lib/apt/lists/*


# Set environment variables to point Puppeteer to system Chromium and ensure Chrome has a writable profile/crashpad directory
ENV PUPPETEER_SKIP_DOWNLOAD=1
ENV PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium
ENV XDG_CONFIG_HOME=/tmp/.chromium
ENV XDG_CACHE_HOME=/tmp/.chromium
ENV CHROME_CRASHPAD_DATABASE=/tmp/.chromium/Crashpad

# Create the Crashpad directory (and ensure /tmp/.chromium is writable)
RUN mkdir -p /tmp/.chromium/Crashpad && chmod -R 777 /tmp/.chromium

# (Optional) Create the directory for Chrome data
RUN mkdir -p /tmp/.chromium && chown -R root:root /tmp/.chromium

# Copy the Rust binary from the builder stage
COPY --from=builder /scraper-bin /app/scraper
# COPY src/run.py /app/src/

# Copy the entire src folder
COPY src/ /app/src/

# COPY src/scrapers-js/ /app/src/scrapers-js/

COPY requirements.txt /app/

# # Set executable permission on the binary
# RUN chmod 755 /app/scraper

# Create secure user and directories
RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /bin/false appuser && \
    mkdir -p /app/output && \
    chown -R appuser:appgroup /app

# Copy entire project (filtered by .dockerignore)
COPY --from=builder --chown=appuser:appgroup /build /app
COPY --from=builder --chown=appuser:appgroup /build/.puppeteer-cache /app/puppeteer-cache
ENV PUPPETEER_CACHE_DIR=/app/puppeteer-cache
RUN chmod -R go-w /app/puppeteer-cache && \
    find /app/puppeteer-cache -type f -name chrome -exec chmod 755 {} +

COPY --from=builder /scraper-bin /app/scraper

# Security hardening and permissions
RUN find /app -type d -exec chmod 755 {} + \
    && find /app -type f -exec chmod 644 {} + \
    && chmod 755 /app/src/run.py \
    && chmod 750 /app/output 
    # \
    # && rm -rf /app/node_modules /app/target

# Apply executable permission for the binary
RUN chmod 755 /app/scraper

# Apply exec permission for chrome
RUN chmod -R 755 /app/puppeteer-cache/chrome

# Apply executable permission for the backup page opener
RUN chmod 755 /app/src/scrapers-js/backup-page-opener.mjs

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