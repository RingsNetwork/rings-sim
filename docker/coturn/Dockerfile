FROM coturn/coturn:4.5.2-r11

USER root
RUN apt-get update && apt-get -y install iproute2 && rm -rf /var/lib/apt/lists/*
USER nobody:nogroup
