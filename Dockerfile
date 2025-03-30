# While piping a curl command directly into a shell to install Rust (via rustup) is common, it inherently
# carries risk because it executes remote code without additional verification. In a highly secure environment,
# you might want to verify checksums or signatures for the installation script to further ensure its integrity.

# also consider pinning version

# ──────────────────────────────────────────────────────────
# 1. Builder Stage
# ──────────────────────────────────────────────────────────
FROM python:3.12-slim as builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN PACKAGE_NAME=$(grep '^name =' Cargo.toml | cut -d '"' -f2) && \
    cargo build --release --locked && \
    mv target/release/${PACKAGE_NAME} /usr/local/bin/ && \
    strip /usr/local/bin/${PACKAGE_NAME}

# ──────────────────────────────────────────────────────────
# 2. Runtime Stage
# ──────────────────────────────────────────────────────────
FROM python:3.12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /bin/false appuser && \
    mkdir -p /app/output && \
    chmod 700 /app/output && \
    chown -R appuser:appgroup /app

COPY --from=builder /usr/local/bin/* /usr/local/bin/
COPY --chown=appuser:appgroup src/run.py requirements.txt /app/
COPY --chown=appuser:appgroup src/ /app/src/

USER appuser
WORKDIR /app

RUN python -m venv /app/venv
ENV PATH="/app/venv/bin:$PATH"
RUN pip install --no-cache-dir -r requirements.txt

VOLUME ["/app/output"]
ENTRYPOINT ["python3", "src/run.py"]