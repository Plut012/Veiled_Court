# Veiled Court - Production Dockerfile
# CUDA-enabled for NVIDIA GPU support

FROM nvidia/cuda:12.1.0-cudnn8-runtime-ubuntu22.04

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
# The release zip contains an AppImage; extract it so it runs without FUSE
RUN wget -q https://github.com/lightvector/KataGo/releases/download/v1.16.4/katago-v1.16.4-cuda12.1-cudnn8.9.7-linux-x64.zip && \
    unzip katago-*.zip && \
    chmod +x katago && \
    ./katago --appimage-extract && \
    mv squashfs-root /opt/katago && \
    ln -sf /opt/katago/AppRun /usr/local/bin/katago && \
    rm -rf katago katago-*.zip *.cfg *.txt *.pem

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

# Copy neural networks into image
COPY assets/katago/kata1-b28c512nbt.bin.gz /app/models/
COPY assets/katago/b18c384nbt-humanv0.bin.gz /app/models/

# Build Rust application (release mode)
RUN cargo build --release

# Expose port
EXPOSE 3000

# Environment variables
ENV KATAGO_BINARY=/usr/local/bin/katago
ENV KATAGO_MODEL=/app/models/kata1-b28c512nbt.bin.gz
ENV KATAGO_HUMAN_MODEL=/app/models/b18c384nbt-humanv0.bin.gz
ENV KITSUNE_CONFIG_DIR=/app/configs
ENV RUST_LOG=info

# Run the application
CMD ["./target/release/kitsune"]
