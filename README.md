# Ring Simulation
Simulate ring behind NAT in docker to test [bns-node](https://github.com/BNSnet/bns-node).

### Build test images
```shell
git submodule update --init --recursive

# This may take a while...
python nind.py build_image

# Show the built images
docker images | grep bnsnet
```

### Prepare stun sever in docker
```shell
docker run -d --rm --name coturn coturn/coturn

# Get its ip address, the default port is 3478
docker container inspect -f '{{ .NetworkSettings.Networks.bridge.IPAddress }}' coturn
```
