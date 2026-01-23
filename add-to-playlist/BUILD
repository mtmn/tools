load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_cc//cc:defs.bzl", "cc_binary")

cc_binary(
    name = "add-to-playlist",
    srcs = ["add-to-playlist.c"],
    copts = [
        "-Wall",
        "-Wextra",
        "-O2",
    ],
    linkopts = ["-lmpdclient"],
    visibility = ["//visibility:public"],
)

local_deploy(
    name = "deploy",
    srcs = [":add-to-playlist"],
)
