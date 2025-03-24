# Use official Rust image as base
FROM rust:1.84-bullseye

# Install only required system deps
RUN apt-get update && \
    apt-get install -y \
        chromium \
        python3 \
        python3-venv \
        python3-pip \
        && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy all source code into the container
COPY . .

# Install Node.js dependencies
WORKDIR /app/puppeteer
RUN npm install

# Install Python dependencies via virtualenv
WORKDIR /app
RUN python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --upgrade pip && \
    if [ -f requirements.txt ]; then /opt/venv/bin/pip install -r requirements.txt; fi

ENV PATH="/opt/venv/bin:$PATH"

# Build the Rust scraper
WORKDIR /app/rust_scraper
RUN cargo build --release

# Default back to root for the Python script
WORKDIR /app

# Run the orchestrator (query can be overridden in docker-compose)
ENTRYPOINT ["python3", "run.py"]