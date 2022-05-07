FROM debian:bullseye

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install iptables iproute2 && rm -rf /var/lib/apt/lists/*

CMD ["sleep", "infinity"]
