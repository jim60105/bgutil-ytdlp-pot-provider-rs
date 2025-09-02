# syntax=docker/dockerfile:1

#! This Containerfile is designed for SELinux-enabled systems.
#! On non-SELinux systems, remove the ",z" flags from --mount options.

ARG UID=1001
ARG VERSION=EDGE
ARG RELEASE=0
ARG NAME=bgutil-pot

########################################
# Chef base stage
########################################
FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1.89.0-slim AS chef
WORKDIR /app

# Create directories with correct permissions
ARG UID
RUN install -d -m 775 -o $UID -g 0 /licenses

# Enable static linking for Rust binaries
ENV RUSTFLAGS="-C target-feature=+crt-static"

########################################
# Planner stage
# Generate a recipe for the project, containing all dependencies information for cooking
########################################
FROM chef AS planner
RUN --mount=source=src,target=src,z \
    --mount=source=Cargo.toml,target=Cargo.toml,z \
    --mount=source=Cargo.lock,target=Cargo.lock,z \
    cargo chef prepare --recipe-path recipe.json

########################################
# Cook stage
# Build the project dependencies, so that they can be cached at separate layer
########################################
FROM chef AS cook

# RUN mount cache for multi-arch: https://github.com/docker/buildx/issues/549#issuecomment-1788297892
ARG TARGETARCH
ARG TARGETVARIANT
RUN --mount=type=cache,id=apt-$TARGETARCH$TARGETVARIANT,sharing=locked,target=/var/cache/apt \
    --mount=type=cache,id=aptlists-$TARGETARCH$TARGETVARIANT,sharing=locked,target=/var/lib/apt/lists \
    # dependencies for git2-rs and other system libs
    apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl

RUN --mount=source=/app/recipe.json,target=recipe.json,from=planner \
    cargo chef cook --release --target x86_64-unknown-linux-gnu --recipe-path recipe.json --all-targets --locked

########################################
# Test stage
########################################
FROM cook AS test

# Install cargo-nextest for running tests
# Temporarily unset RUSTFLAGS to allow proc-macro compilation for the host
RUN env -u RUSTFLAGS cargo install cargo-nextest --locked

RUN --mount=source=src,target=src,z \
    --mount=source=Cargo.toml,target=Cargo.toml,z \
    --mount=source=Cargo.lock,target=Cargo.lock,z \
    --mount=source=.config/nextest.toml,target=.config/nextest.toml,z \
    cargo nextest run --release --target x86_64-unknown-linux-gnu --all-targets --locked

########################################
# Builder stage
# This stage relies on test stage passing
# Note that we already have project built in test stage, so this will be fast
########################################
FROM test AS builder

ARG NAME
RUN --mount=source=src,target=src,z \
    --mount=source=Cargo.toml,target=Cargo.toml,z \
    --mount=source=Cargo.lock,target=Cargo.lock,z \
    cargo build --release --target x86_64-unknown-linux-gnu --bin ${NAME} --locked

########################################
# Compress stage
########################################
FROM chef AS compress

# RUN mount cache for multi-arch: https://github.com/docker/buildx/issues/549#issuecomment-1788297892
ARG TARGETARCH
ARG TARGETVARIANT

# Compress dist and dumb-init with upx
ARG NAME
RUN --mount=type=cache,id=apt-$TARGETARCH$TARGETVARIANT,sharing=locked,target=/var/cache/apt \
    --mount=type=cache,id=aptlists-$TARGETARCH$TARGETVARIANT,sharing=locked,target=/var/lib/apt/lists \
    --mount=from=builder,source=/app/target/x86_64-unknown-linux-gnu/release/${NAME},target=/tmp/app \
    echo "deb http://deb.debian.org/debian bookworm-backports main" >> /etc/apt/sources.list && \
    apt-get update && apt-get install -y -t bookworm-backports \
    upx-ucl && \
    apt-get install -y wget && \
    cp /tmp/app /${NAME} && \
    # Download static dumb-init binary \
    wget -O /dumb-init https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64 && \
    chmod +x /dumb-init && \
    #! UPX will skip small files and large files \
    # https://github.com/upx/upx/blob/5bef96806860382395d9681f3b0c69e0f7e853cf/src/p_unix.cpp#L80 \
    # https://github.com/upx/upx/blob/b0dc48316516d236664dfc5f1eb5f2de00fc0799/src/conf.h#L134 \
    (upx --best --lzma /${NAME} || true) && \
    (upx --best --lzma /dumb-init || true) && \
    apt-get remove -y upx-ucl wget

########################################
# Binary stage
# How to: docker build --output=. --target=binary .
########################################
FROM scratch AS binary

ARG NAME
COPY --chown=0:0 --chmod=777 --from=compress /${NAME} /${NAME}

########################################
# Final stage
########################################
FROM scratch AS final

# Copy CA trust store
# Rust seems to use this one: https://stackoverflow.com/a/57295149/8706033
COPY --from=chef /etc/ssl/certs/ca-certificates.crt /etc/ssl/cert.pem

ARG UID

# Copy static dumb-init binary
COPY --chown=$UID:0 --chmod=775 --from=compress /dumb-init /dumb-init

# Create directories with correct permissions
COPY --chown=$UID:0 --chmod=775 --from=chef /licenses /licenses

# Copy licenses (OpenShift Policy)
COPY --chown=$UID:0 --chmod=775 LICENSE /licenses/LICENSE

# Copy dist
ARG NAME
COPY --chown=$UID:0 --chmod=775 --from=compress /${NAME} /bgutil-pot

# Copy yt-dlp plugin for distribution with the container
COPY --chown=$UID:0 --chmod=775 plugin/yt_dlp_plugins /client/yt_dlp_plugins

ENV PATH="/"

WORKDIR /

VOLUME [ "/tmp" ]

EXPOSE 4416

USER $UID

STOPSIGNAL SIGINT

# Use dumb-init as PID 1 to handle signals properly
ENTRYPOINT ["/dumb-init", "--", "/bgutil-pot"]
CMD ["server", "--host", "0.0.0.0"]

ARG VERSION
ARG RELEASE
LABEL name="bgutil-pot" \
    # Authors for the main application
    vendor="Jim Chen" \
    # Maintainer for this container image
    maintainer="jim60105" \
    # Containerfile source repository
    url="https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs" \
    version=${VERSION} \
    # This should be a number, incremented with each change
    release=${RELEASE} \
    io.k8s.display-name="BgUtils POT Provider" \
    summary="High-performance YouTube POT (Proof-of-Origin Token) provider" \
    description="A Rust implementation of POT provider for yt-dlp to bypass YouTube's 'Sign in to confirm you're not a bot' restrictions. For more information about this tool, please visit the following website: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs"
