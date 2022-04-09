### For Dev
you can check doc of netsim here:

https://bnsnet.github.io/netsim-embed/

### For Macbook M1

* Setup:

```
git submodule update --init --recursive
vagrant up --provider=parallels
```

* Build:
```
vagrant ssh -c "cargo build --manifest-path sim/Cargo.toml"
```
