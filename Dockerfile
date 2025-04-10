FROM docker.io/rust:1.85-bookworm

# Optional: Install Node.js if your project uses it.
RUN apt-get update && apt-get install -y \
    curl \
    gnupg \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Install cargo-watch to automatically run your app when files change
RUN cargo install cargo-watch

# Use cargo-watch to rebuild and run the app on changes.
CMD ["cargo", "watch", "-x", "run"]

