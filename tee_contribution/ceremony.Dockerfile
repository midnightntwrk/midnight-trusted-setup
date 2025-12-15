FROM ubuntu:jammy-20251013 AS builder

ARG SNAPSHOT=20251013T000000Z
ARG GITHUB_URL_PREFIX="https://github.com"
ARG RUST_TOOLCHAIN="1.84.1"

SHELL ["/bin/bash", "-euxo", "pipefail", "-c"]

# Allows retries on the apt-get snapshot repository.
# Considering that the snapshot repository is slow and subject to timeouts,
# we set a retry count and a timeout for apt-get operations.
# This helps to avoid build failures due to transient network issues.
RUN echo 'Acquire::Retries "3"; Acquire::http::Timeout "30";' \
    > /etc/apt/apt.conf.d/99retries

# Needed to be able to download packages from snapshot.ubuntu.com
RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && \
    DEBIAN_FRONTEND=noninteractive \
    apt-get install -y --no-install-recommends \
        ca-certificates=20240203~22.04.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# System dependencies
# We use the snapshot repository to ensure reproducibility.
RUN --mount=type=cache,target=/var/cache/apt \
    printf 'deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/%s jammy main restricted universe\n\
deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/%s jammy-updates main restricted universe\n\
deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/%s jammy-security main restricted universe\n' \
        "${SNAPSHOT}" "${SNAPSHOT}" "${SNAPSHOT}" > /etc/apt/sources.list && \
    apt-get update && \
    DEBIAN_FRONTEND=noninteractive \
    apt-cache madison perl-modules perl-base libperl5.34 build-essential libssl-dev git && \
    apt-get install -y --allow-downgrades --no-install-recommends \
        perl-base=5.34.0-3ubuntu1.5 \
        libperl5.34=5.34.0-3ubuntu1.5 \
        perl-modules-5.34=5.34.0-3ubuntu1.5 \
        curl=7.81.0-1ubuntu1.21 \
        wget=1.21.2-2ubuntu1.1 \
        git=1:2.34.1-1ubuntu1 \
        libssl3=3.0.2-0ubuntu1.20 \
        libssl-dev=3.0.2-0ubuntu1.20 \
        build-essential=12.9ubuntu3 && \
    rm -rf /var/lib/apt/lists/*

# Rust
RUN curl -fsSL https://sh.rustup.rs -o /tmp/rustup-init && \
    chmod +x /tmp/rustup-init && \
    /tmp/rustup-init -y --no-modify-path --default-toolchain "${RUST_TOOLCHAIN}" && \
    . "/root/.cargo/env" && \
    rustup set auto-self-update disable && \
    rustc -V && cargo -V && \
    rm /tmp/rustup-init
ENV PATH="/root/.cargo/bin:$PATH" \
    RUSTFLAGS="-C link-arg=-Wl,--build-id=none"

# Build everything and print hashes. We also save hashes in /artifacts/hashes.txt for convenience.
WORKDIR /build
RUN mkdir -p /artifacts

# Clone and build the trusted-setup-management-server and midnight-trusted-setup repositories.
# We checkout specific commits to ensure reproducibility.
RUN git clone "${GITHUB_URL_PREFIX}/input-output-hk/trusted-setup-management-server.git" server && \
    cd server && git checkout 1b374bd8bda535999515f4c80c5fa3ec1c66453c && \
    export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) && \
    cargo build --release --locked && \
    install -Dm755 target/release/srs-srv /artifacts/srs-srv && \
    sha256sum /artifacts/srs-srv | tee -a /artifacts/hashes.txt && \
    cp srs_server.service /artifacts/srs_server.service && \
    cd .. && \
    \
    git clone "${GITHUB_URL_PREFIX}/midnightntwrk/midnight-trusted-setup.git" tee && \
    cd tee && git checkout 7e84b09abfce31a94e876480de5fff4af6d785e4 && \
    export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) && \
    cargo build --release --locked && \
    install -Dm755 target/release/srs_utils /artifacts/srs_utils && \
    sha256sum /artifacts/srs_utils | tee -a /artifacts/hashes.txt && \
    cd .. && rm -rf server tee

# Print binary hashes
# Allows to compare the hashes calculated here with the one in the attestation.
# It is also possible to copy the binairies from the /artifacts directory and run the 
# hash calculation locally.
FROM busybox:uclibc 
COPY --from=builder /artifacts /artifacts
CMD ["/bin/sh", "-c", "cat /artifacts/hashes.txt"]