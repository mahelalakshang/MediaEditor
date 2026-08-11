# Build context is the repo root (see docker-compose.yml) — same
# workspace-wide reasoning as services/core-service/Dockerfile.
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY services/core-service services/core-service
COPY services/media-worker services/media-worker
COPY proto proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake build-essential pkg-config libssl-dev zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p media-worker
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY services/core-service services/core-service
COPY services/media-worker services/media-worker
COPY proto proto
RUN cargo build --release -p media-worker

FROM debian:bookworm-slim AS runtime
# The whole point of this service: real ffmpeg/ffprobe binaries, invoked
# as subprocesses at runtime (see services/media-worker/src/video.rs).
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/media-worker /usr/local/bin/media-worker
ENTRYPOINT ["/usr/local/bin/media-worker"]
