# Only specific directories (e.g., /home/appuser/app/output or 
# /home/appuser/data) are meant to be writable. If an attacker 
# compromises the container, their ability to modify the code 
# or system libraries is limited.
# In production, you should mount these directories as separate 
# volumes so that the rest of the container filesystem remains read-only.

# ──────────────────────────────────────────────────────────
# 1. Base Image & System Packages
# ──────────────────────────────────────────────────────────
FROM python:3.12

USER root

# Update package lists and install build tools and libraries needed for:
#   - Building Rust code
#   - Running Puppeteer (Chromium) in headless mode
#   - Installing Node.js 22.x via NodeSource
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates gnupg \
    wget unzip \
    fonts-liberation libnss3 libatk1.0-0 libatk-bridge2.0-0 \
    libcups2 libdrm2 libxcomposite1 libxdamage1 libxrandr2 libgbm1 \
    libasound2 libpangocairo-1.0-0 libpangoft2-1.0-0 libxss1 libxshmfence1 \
    libgtk-3-0 libx11-xcb1 libxext6 libxfixes3 xvfb \
 && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
 && apt-get install -y nodejs \
 && rm -rf /var/lib/apt/lists/*

# ──────────────────────────────────────────────────────────
# 2. Create Non-Root User and Set Up Directories
# ──────────────────────────────────────────────────────────
RUN useradd -m -u 1000 appuser \
 && mkdir -p /home/appuser/app \
 && mkdir -p /home/appuser/data \
 && chown -R appuser:appuser /home/appuser/app /home/appuser/data

# ──────────────────────────────────────────────────────────
# 3. Switch to Non-Root User & Set Workdir
# ──────────────────────────────────────────────────────────
USER appuser
WORKDIR /home/appuser/app

# ──────────────────────────────────────────────────────────
# 4. Install Rust and Python Virtual Environment
# ──────────────────────────────────────────────────────────
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/home/appuser/.cargo/bin:$PATH"

RUN python3 -m venv /home/appuser/venv
ENV PATH="/home/appuser/venv/bin:$PATH"

# ──────────────────────────────────────────────────────────
# 5. Copy the Entire Project and Install Dependencies
# ──────────────────────────────────────────────────────────
# This copies your entire repository (which should include:
#   requirements.txt, package.json, package-lock.json, Cargo.toml, src/, etc.)
COPY --chown=appuser:appuser . .

# Install Python dependencies (from requirements.txt)
RUN pip install --upgrade pip && pip install -r requirements.txt

# Install Node dependencies (e.g. Puppeteer)
RUN npm install

# ──────────────────────────────────────────────────────────
# 6. Build the Rust Project
# ──────────────────────────────────────────────────────────
RUN cargo build --release

# ──────────────────────────────────────────────────────────
# 7. Harden the Code (Optional for Prod)
# ──────────────────────────────────────────────────────────
# Make all files in your source directory read-only (adjust as needed)
# WARNING: Ensure that directories that need to be written to (logs, output, etc.) are NOT locked down.
RUN find /home/appuser/app/src -type f -exec chmod 444 {} \; && \
    find /home/appuser/app/src -type d -exec chmod 555 {} \;

# ──────────────────────────────────────────────────────────
# 8. Final Setup: Define the Entrypoint
# ──────────────────────────────────────────────────────────
# The container will start at the project root, but run the Python script in src/
WORKDIR /home/appuser/app
ENTRYPOINT ["python3", "src/run.py"]