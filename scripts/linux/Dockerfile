FROM ubuntu:22.04

ARG RUST_VERSION=1.99.0
ARG NODE_VERSION=24.21.0

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    CARGO_TARGET_DIR=/target \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    PATH=/usr/local/cargo/bin:/usr/local/node/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl file git perl pkg-config xz-utils patchelf \
        libssl-dev libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} \
    && mkdir -p /usr/local/node \
    && curl -fsSL https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz | tar -xJ -C /usr/local/node --strip-components=1 \
    && git config --system --add safe.directory '*'

COPY build.sh /usr/local/bin/build.sh

ENTRYPOINT ["sh", "/usr/local/bin/build.sh"]
