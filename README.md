# Ring Simulation
Simulate ring behind NAT in docker to test [bns-node](https://github.com/BNSnet/bns-node).

### Build test images
```shell
❯ git submodule update --init --recursive

# This may take a while...
❯ python nind.py build_image

# Show the built images
❯ docker images | grep bnsnet
```

### Prepare stun sever in docker
```shell
❯ python nind.py create_coturn
```

### Run two nodes behind same NAT
```shell
python nind.py create_nat
# Some logs...
# It will give you <nat id> and <router id> at the end.

# Do it twice
python nind.py create_node -l <nat id> -r <router id>
```
