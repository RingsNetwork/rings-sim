### For Dev
you can check doc of netsim here:

https://bnsnet.github.io/netsim-embed/

### For Macbook M1

* Setup:

```shell
git submodule update --init --recursive
vagrant up --provider=parallels
```

* Run test:
```shell
vagrant ssh -c "cd sim && RUST_LOG=info cargo test -- --test-threads=1"
```
