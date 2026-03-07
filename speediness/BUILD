load("@rules_zig//zig:defs.bzl", "zig_binary")
load("//bazel:local-deploy.bzl", "local_deploy")

zig_binary(
    name = "speediness",
    srcs = glob(["src/*.zig"]),
    main = "src/main.zig",
    visibility = ["//visibility:public"],
    zigopts = [
        "-O",
        "ReleaseSafe",
        "-Doptimize=ReleaseSafe",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":speediness"],
)
