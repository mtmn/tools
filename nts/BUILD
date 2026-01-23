load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "nts",
    srcs = ["nts.py"],
    main = "nts.py",
    python_version = "PY3",
    deps = [
        "@pypi//requests",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":nts"],
)
