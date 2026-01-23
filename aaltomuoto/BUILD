load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "aaltomuoto",
    srcs = ["aaltomuoto.py"],
    main = "aaltomuoto.py",
    python_version = "PY3",
    deps = [
        "@pypi//numpy",
        "@pypi//pillow",
        "@pypi//mutagen",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":aaltomuoto"],
)
