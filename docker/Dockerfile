# This is a test Dockerfile. Do not use it for production.

# build
FROM rustlang/rust:nightly AS builder

RUN apt-get update && apt-get install -y iproute2 iputils-ping && rm -rf /var/lib/apt/lists/*

# This step compiles only our dependencies and saves them in a layer. This is the most impactful time savings
# Note the use of --mount=type=cache. On subsequent runs, we'll have the crates already downloaded
RUN cargo new --vcs none /rings-node && \
    touch /rings-node/src/lib.rs && \
    mkdir /rings-node/bin && \
    cp /rings-node/src/main.rs /rings-node/bin/main.rs && \
    cp /rings-node/src/main.rs /rings-node/bin/daemon.rs && \
    cargo new --vcs none --lib /rings-node/rings-core
WORKDIR /rings-node
ENV CARGO_TARGET_DIR /rings-node/target/docker
COPY rings-node/Cargo.toml rings-node/Cargo.lock ./
COPY rings-node/rings-core/Cargo.toml ./rings-core
RUN --mount=type=cache,target=/local/cargo/registry cargo build --all

# Cover fake codes
COPY ./rings-node .

# We're mounting that cache again to use during the build, otherwise it's not present and we'll have to download those again
# Rust here is a bit fiddly, so we'll touch the files (even though we copied over them) to force a new build
RUN --mount=type=cache,target=/local/cargo/registry touch /rings-node/rings-core/lib.rs /rings-node/bin/main.rs && \
                                                    rm /rings-node/rust-toolchain.toml && \
                                                    cargo clean -p rings-core && \
                                                    cargo build --all && \
                                                    mv /rings-node/target/docker/debug/rings-node . && \
                                                    rm -rf target/docker

# Export bins. Keep image slim
FROM debian:bullseye-slim AS app
RUN apt-get update && apt-get install -y iproute2 iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=builder /rings-node/rings-node /bin/rings-node
CMD ["sleep", "infinity"]
