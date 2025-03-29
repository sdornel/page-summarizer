# this must all be hardened before it goes to prod!

FROM python:3.12

# Step 1: Install system packages (as root)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates gnupg \
    wget unzip fonts-liberation libnss3 libatk1.0-0 libatk-bridge2.0-0 \
    libcups2 libdrm2 libxcomposite1 libxdamage1 libxrandr2 libgbm1 \
    libasound2 libpangocairo-1.0-0 libpangoft2-1.0-0 libxss1 libxshmfence1 \
    libgtk-3-0 libx11-xcb1 libxext6 libxfixes3 xvfb \
 && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
 && apt-get install -y nodejs \
 && rm -rf /var/lib/apt/lists/*

# Step 2: Create non-root user
RUN useradd -m -u 1000 appuser

RUN mkdir -p /home/appuser/app/node_modules && chown -R appuser:appuser /home/appuser/app

USER appuser
WORKDIR /home/appuser/app

# Step 3: Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/home/appuser/.cargo/bin:$PATH"

# Step 4: Set up Python virtualenv
RUN python3 -m venv /home/appuser/venv
ENV PATH="/home/appuser/venv/bin:$PATH"

# Step 5: Copy entire project (everything lives at root level)
COPY --chown=appuser:appuser . .

# Step 6: Install Python and Node dependencies
RUN pip install --upgrade pip && pip install -r requirements.txt
RUN npm install

# Step 7: Build Rust
RUN cargo build --release

# Step 8: Define final working directory and run script
WORKDIR /home/appuser/app
ENTRYPOINT ["python3", "src/run.py"]