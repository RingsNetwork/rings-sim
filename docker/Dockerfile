# This is a test Dockerfile. Do not use it for production.
FROM rust

RUN apt-get update && apt-get install -y iproute2 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/bns-node
COPY . .

RUN cargo build --all

CMD ["sleep", "infinity"]
