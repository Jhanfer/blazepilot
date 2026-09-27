FROM rust:1.98-bookworm AS builder

ENV DEBIAN_FRONTEND=noninteractive

# Install essential packages first with resource limits
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    cmake \
    meson \
    ninja-build \
    nasm \
    yasm \
    pkg-config \
    git \
    ca-certificates \
    curl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install audio/video development libraries
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
    libasound2-dev \
    libdav1d-dev \
    && rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install X11 and graphics development libraries
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
    libx11-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    libwayland-dev \
    libgl1-mesa-dev \
    libegl1-mesa-dev \
    libgles2-mesa-dev \
    libvulkan-dev \
    && rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install system libraries
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
    libdbus-1-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

RUN cargo build \
    --release \
    --locked \
    --features build-ci

RUN mkdir -p /artifact/release \
    && cp target/release/blazepilot /artifact/release/blazepilot