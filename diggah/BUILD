load("@rules_python//python:defs.bzl", "py_binary")
load("//bazel:local-deploy.bzl", "local_deploy")

py_binary(
    name = "diggah",
    srcs = ["diggah.py"],
    main = "diggah.py",
    python_version = "PY3",
    deps = [
        "@pypi//fire",
        "@pypi//paramiko",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":diggah"],
    copy_runfiles = True,
)
