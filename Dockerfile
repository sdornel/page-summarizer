FROM python:3.12

# Step 1: Install system packages (as root)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev ca-certificates gnupg \
  && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
  && apt-get install -y nodejs \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user and give full ownership
RUN useradd -m -u 1000 appuser \
 && mkdir -p /home/appuser/app \
 && chown -R appuser:appuser /home/appuser/app

RUN mkdir -p /home/appuser/app/target && chown -R appuser:appuser /home/appuser/app/target

# Switch to non-root user
USER appuser
WORKDIR /home/appuser/app

# Step 3: Install Rust as appuser
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/home/appuser/.cargo/bin:$PATH"

# Step 4: Set up Python virtualenv
RUN python3 -m venv /home/appuser/venv
ENV PATH="/home/appuser/venv/bin:$PATH"

# Copy your code
COPY --chown=appuser:appuser . .

# Install Node and Python dependencies
RUN cd /home/appuser/app && npm install
RUN pip install --upgrade pip && pip install -r requirements.txt

# Step 6: Build Rust project
RUN cargo build --release

# Step 7: Set working directory and run script
WORKDIR /home/appuser/app
ENTRYPOINT ["python3", "src/run.py"]