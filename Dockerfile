# Start with Node base (comes with npm)
FROM node:22

# Install only what's needed
RUN apt-get update && \
    apt-get install -y \
        python3 \
        python3-venv \
        python3-pip \
        chromium \
        && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Install Puppeteer dependencies
WORKDIR /app/google-scraper
RUN npm install

# Set up Python virtualenv
WORKDIR /app
RUN python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --upgrade pip && \
    if [ -f requirements.txt ]; then /opt/venv/bin/pip install -r requirements.txt; fi

# Use Python virtualenv by default
ENV PATH="/opt/venv/bin:$PATH"

# Default working dir for CLI shell or orchestrator
WORKDIR /app

# Start a shell session by default
ENTRYPOINT ["bash"]
