# Build context is the repo root (see docker-compose.yml) because this is
# a Cargo workspace — cargo-chef needs every member's Cargo.toml to produce
# an accurate dependency recipe, even though this image only ships the
# core-service binary.
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
# cmake/build-essential: rdkafka's vendored librdkafka (cmake-build feature).
# libssl-dev/zlib1g-dev: librdkafka's own build deps.
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake build-essential pkg-config libssl-dev zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p core-service
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY services/core-service services/core-service
COPY services/media-worker services/media-worker
COPY proto proto
RUN cargo build --release -p core-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# sqlx::migrate! embeds the migration SQL into the binary at compile
# time, so nothing from services/core-service/migrations needs to be
# copied into the runtime image.
COPY --from=builder /app/target/release/core-service /usr/local/bin/core-service
EXPOSE 8080 50051
ENTRYPOINT ["/usr/local/bin/core-service"]
