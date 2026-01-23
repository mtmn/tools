load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "lazymaster",
    srcs = ["lazymaster.py"],
    main = "lazymaster.py",
    python_version = "PY3",
    deps = [
        "@pypi//requests",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":lazymaster"],
)
