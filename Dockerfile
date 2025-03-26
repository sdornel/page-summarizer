FROM python:3.12

# Install essential system packages (as root)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Set a non-root user (best security practice)
RUN useradd -m -u 1000 appuser
USER appuser

# Set working directory (home directory of the appuser)
WORKDIR /home/appuser/app

# Install Rust as the non-root user
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/home/appuser/.cargo/bin:$PATH"

# Set up Python virtualenv
RUN python3 -m venv /home/appuser/venv
ENV PATH="/home/appuser/venv/bin:$PATH"

# Install Python dependencies
COPY --chown=appuser:appuser requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt

# Copy and build your Rust project
COPY --chown=appuser:appuser rust-scraper ./rust-scraper
WORKDIR /home/appuser/app/rust-scraper
RUN cargo build --release

# Copy your Python scripts and the rest of your project
WORKDIR /home/appuser/app
COPY --chown=appuser:appuser . .

# Default command to run Python script
CMD ["python3", "run.py"]