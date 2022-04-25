# This is a test Dockerfile. Do not use it for production.

# build
FROM rust AS build

# This step compiles only our dependencies and saves them in a layer. This is the most impactful time savings
# Note the use of --mount=type=cache. On subsequent runs, we'll have the crates already downloaded
RUN cargo new /src/bns-node && \
    mkdir /src/bns-node/bin && \
    cp /src/bns-node/src/main.rs /src/bns-node/bin/main.rs && \
    cp /src/bns-node/src/main.rs /src/bns-node/bin/daemon.rs && \
    cargo new --lib /src/bns-node/bns-core
WORKDIR /src/bns-node
COPY bns-node/Cargo.toml bns-node/Cargo.lock bns-node/rust-toolchain.toml ./
COPY bns-node/bns-core/Cargo.toml ./bns-core
RUN --mount=type=cache,target=/local/cargo/registry cargo build --all

# Cover fake codes
COPY ./bns-node .

# We're mounting that cache again to use during the build, otherwise it's not present and we'll have to download those again
# Rust here is a bit fiddly, so we'll touch the files (even though we copied over them) to force a new build
RUN --mount=type=cache,target=/local/cargo/registry touch /src/bns-node/bns-core/lib.rs /src/bns-node/bin/main.rs && cargo clean -p bns-core && cargo build --all

# Export bins. Keep image slim
FROM debian:bullseye-slim AS app
RUN apt-get update && apt-get install -y iproute2 iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/bns-node/target/debug/bns-node /bin/bns-node
CMD ["sleep", "infinity"]
