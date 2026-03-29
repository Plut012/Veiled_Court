# Spirit Animals Go - Production Dockerfile
# CUDA-enabled for NVIDIA GPU support

FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    unzip \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (latest stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Download KataGo binary (CUDA 12.1 version)
RUN wget -q https://github.com/lightvector/KataGo/releases/download/v1.16.4/katago-v1.16.4-cuda12.1-cudnn8.9.7-linux-x64.zip && \
    unzip katago-*.zip && \
    mv katago /usr/local/bin/katago && \
    chmod +x /usr/local/bin/katago && \
    rm -rf katago-*.zip *.cfg *.txt *.pem

# Set working directory
WORKDIR /app

# Copy project files
COPY Cargo.toml ./
# Note: Not copying Cargo.lock - let cargo generate compatible versions
COPY src/ ./src/
COPY configs/ ./configs/
COPY configs-dev/ ./configs-dev/
COPY frontend/ ./frontend/
COPY scripts/ ./scripts/

# Copy neural networks (must be pre-downloaded on host)
# Run ./scripts/download_nets.sh on desktop before building
COPY nets/ /app/nets/

# Build Rust application (release mode)
RUN cargo build --release

# Expose port
EXPOSE 3000

# Environment variables (can be overridden)
ENV KATAGO_BINARY=/usr/local/bin/katago
ENV KATAGO_NETS_PATH=/app/nets
ENV ANIMAL_GO_CONFIG_DIR=/app/configs
ENV RUST_LOG=info

# Run the application
CMD ["./target/release/animal_go"]
