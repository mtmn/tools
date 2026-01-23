# tools
I wanted to learn about [Bazel](https://bazel.build) but instead got addicted to watching my builds exit out with nothingburger error messages, one at a time.

It was as if I was tilting in a high stakes poker game but instead of money I lost my sanity. And a lot of time. 

![](misc/hehe.png)

I really like it though.

### bootstrap
```bash
sudo pacman -Sy bazelisk mold
bazel fetch //...
```

### query
```bash
bazel query "kind('rust_binary', //...) except attr('name', '.*-(debug|profiling)$', //...)"
bazel query "kind('py_binary', //...)"
bazel query "kind('cc_binary', //...)"
bazel query "kind('sh_binary', //...)"
```

### build
```bash
bazel build //magnolia
```

### deploy 
> `bazel-bin` -> ``/opt/bazel-tools``
```bash
sudo install -d -m 700 -o $USER /opt/bazel-tools
bazel run //magnolia:deploy
```

### misc
```bash
# updates root `Cargo.lock` (rust tools are nested in a workspace)
cargo generate-lockfile
# builds everything (including `debug` and `profiling` binaries)
bazel build //...
# updates python packages (`requirements.in` -> `requirements.out`)
bazel run //:requirements.update
# runs a `Makefile` that contains all of the necessary commands
make help
```
