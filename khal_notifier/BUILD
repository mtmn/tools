load("//bazel:local-deploy.bzl", "local_deploy")

# Assumes you've already built the binary with: mix escript.build
# This just copies the existing binary to /opt/tools

filegroup(
    name = "binary",
    srcs = ["khal_notifier"],
    visibility = ["//visibility:public"],
)

local_deploy(
    name = "deploy",
    srcs = [":binary"],
    target = "/opt/tools",
)
