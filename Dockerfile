# While piping a curl command directly into a shell to install Rust (via rustup) is common, it inherently
# carries risk because it executes remote code without additional verification. In a highly secure environment,
# you might want to verify checksums or signatures for the installation script to further ensure its integrity.

# ──────────────────────────────────────────────────────────
# 1. Base Build Stage
# ──────────────────────────────────────────────────────────
# Begins the build process with the python:3.12-slim image, tagging this stage as builder. 
# This image is lightweight and based on Debian slim, providing Python while keeping the footprint low.
FROM python:3.12-slim as builder

# Install build dependencies
# --no-install-recommends flag ensures that only the necessary packages are installed
# at end it it cleans up the local package lists to reduce the image size
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with explicit path usage
# Downloads and runs the Rust installer script (rustup). The options -y (automatic yes) and --profile minimal 
# install a minimal Rust toolchain non-interactively, ensuring only what’s needed for the build is installed.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
# Adds the Rust and Cargo binary directory (/root/.cargo/bin) to the system PATH.
# This makes the cargo command (Rust’s build tool) available in subsequent commands.
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify Rust installation
RUN cargo --version

# ──────────────────────────────────────────────────────────
# 2. Build Rust Binary
# ──────────────────────────────────────────────────────────
WORKDIR /build
COPY . .

# Use absolute path to cargo and direct build
# 1)  Uses grep on Cargo.toml to extract the project’s package name. The command looks for the line 
# 2) starting with name = and then uses cut to get the name between the quotes.
# 3) Runs cargo build --release --locked to compile the Rust project in release mode (optimized) 
# while ensuring the Cargo.lock file is respected (--locked)
# 4) Uses the strip command to remove debugging symbols, further reducing the size of the binary

RUN PACKAGE_NAME=$(grep '^name =' Cargo.toml | cut -d '"' -f2) && \
    /root/.cargo/bin/cargo build --release --locked && \
    mv target/release/${PACKAGE_NAME} /usr/local/bin/ && \
    strip /usr/local/bin/${PACKAGE_NAME}

# ──────────────────────────────────────────────────────────
# 3. Runtime Stage
# ──────────────────────────────────────────────────────────
FROM python:3.12-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create secure user and directories
RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /bin/false appuser && \
    mkdir -p /app/output && \
    chown -R appuser:appgroup /app

# Copy artifacts
COPY --from=builder /usr/local/bin/* /usr/local/bin/
COPY --chown=appuser:appgroup . /app

# Install Python dependencies
WORKDIR /app
RUN pip install --no-cache-dir -r requirements.txt

# Final config
USER appuser
VOLUME ["/app/output"]
ENTRYPOINT ["python3", "src/run.py"]