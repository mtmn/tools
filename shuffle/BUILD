load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "shuffle",
    srcs = ["shuffle.py"],
    main = "shuffle.py",
    python_version = "PY3",
)

local_deploy(
    name = "deploy",
    srcs = [":shuffle"],
)
