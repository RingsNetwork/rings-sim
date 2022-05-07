# Ring Simulation
Simulate ring behind NAT in docker to test [rings-node](https://github.com/RingsNetwork/rings-node).

## Prepare

Build test images
```shell
❯ git submodule update --init --recursive

# May take a while...
❯ python nind.py build_image

# Show the built images
❯ docker images | grep ringsnetwork
```

Prepare stun sever in docker
```shell
❯ python nind.py create_coturn
```

## Run Test
```shell
cargo test
```

## Clean up
```shell
# Remove routers and nondes
> python nind.py clean

# Also remove coturn container and global network
> python nind.py clean --all

```
