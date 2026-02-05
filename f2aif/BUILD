load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "f2aif",
    srcs = ["f2aif.py"],
    main = "f2aif.py",
    python_version = "PY3",
)

local_deploy(
    name = "deploy",
    srcs = [":f2aif"],
)
