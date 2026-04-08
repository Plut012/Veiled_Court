# Spirit Animals Go - Production Dockerfile
# Multi-stage build: compile Rust in a builder, copy binary to slim runtime

# --- Builder stage ---
FROM rust:1.87-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

RUN cargo build --release

# --- Runtime stage ---
FROM nvidia/cuda:12.1.0-cudnn8-runtime-ubuntu22.04

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Download KataGo binary (CUDA 12.1 version)
# The release zip contains an AppImage; extract it so it runs without FUSE
RUN wget -q https://github.com/lightvector/KataGo/releases/download/v1.16.4/katago-v1.16.4-cuda12.1-cudnn8.9.7-linux-x64.zip && \
    unzip katago-*.zip && \
    chmod +x katago && \
    ./katago --appimage-extract && \
    mv squashfs-root /opt/katago && \
    ln -sf /opt/katago/AppRun /usr/local/bin/katago && \
    rm -rf katago katago-*.zip *.cfg *.txt *.pem

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /build/target/release/animal_go ./animal_go

# Copy static assets (configs, frontend)
COPY configs/ ./configs/
COPY frontend/ ./frontend/

# Neural nets are mounted as a volume, not baked in
# Mount host nets/ to /app/nets/ via docker-compose
VOLUME /app/nets

# Environment variables
ENV KATAGO_BINARY=/usr/local/bin/katago
ENV KATAGO_MODEL=/app/nets/kata1-b28c512nbt.bin.gz
ENV KATAGO_HUMAN_MODEL=/app/nets/b18c384nbt-humanv0.bin.gz
ENV ANIMAL_GO_CONFIG_DIR=/app/configs
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./animal_go"]
