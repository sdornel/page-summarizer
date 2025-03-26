FROM python:3.12

# Install system packages
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser
USER appuser

# Set working directory
WORKDIR /home/appuser/app

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/home/appuser/.cargo/bin:$PATH"

# Set up Python virtualenv
RUN python3 -m venv /home/appuser/venv
ENV PATH="/home/appuser/venv/bin:$PATH"

# Copy and install Python deps
COPY --chown=appuser:appuser requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt

# Copy and build Rust project
COPY --chown=appuser:appuser rust-scraper ./rust-scraper
WORKDIR /home/appuser/app/rust-scraper
RUN cargo build --release

# Go back to app dir
WORKDIR /home/appuser/app

# Ensure run.py is explicitly copied
COPY --chown=appuser:appuser run.py .

# (Optional) Copy any other needed files or folders
# COPY --chown=appuser:appuser . .

# Run your Python script
CMD ["python3", "run.py"]