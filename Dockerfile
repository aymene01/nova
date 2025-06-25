FROM rust:1.85.1-slim

WORKDIR /app

# Install system dependencies if needed
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Build the project
RUN cargo build --release

CMD ["./target/release/nova", "start"] 