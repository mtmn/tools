load("@rules_cc//cc:defs.bzl", "cc_library")
load("@rules_zig//zig:defs.bzl", "zig_binary")
load("//bazel:local-deploy.bzl", "local_deploy")

cc_library(
    name = "nuklear_impl",
    srcs = ["src/nuklear_impl.c"],
    deps = [
        "@ntsgui_raylib//:raylib",
        "@ntsgui_raylib_nuklear//:raylib_nuklear",
    ],
)

zig_binary(
    name = "ntsgui",
    srcs = glob(["src/*.zig"]),
    extra_srcs = glob(["src/assets/*"]),
    main = "src/main.zig",
    visibility = ["//visibility:public"],
    deps = [
        ":nuklear_impl",
        "@ntsgui_raylib_zig//:raylib",
        "@rules_zig//zig/lib:libc",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":ntsgui"],
)
