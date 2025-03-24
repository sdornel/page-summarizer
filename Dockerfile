FROM rust:1.84-bullseye

# Install Python + venv tools
RUN apt-get update && \
    apt-get install -y python3 python3-venv python3-pip && \
    apt-get clean

# Set working dir
WORKDIR /app

# Copy project files
COPY . .

# Install Python + venv tools
RUN apt-get update && \
    apt-get install -y python3 python3-pip python3-venv python3-distutils && \
    python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --upgrade pip && \
    /opt/venv/bin/pip install -r /app/requirements.txt && \
    apt-get clean

# Tell Rust code to use the virtualenv Python
ENV PATH="/app/venv/bin:$PATH"

# Build Rust project
RUN cargo build

# Default command
CMD ["cargo", "run"]
