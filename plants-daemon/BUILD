load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")

rust_binary(
    name = "plants-daemon",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
        "-Copt-level=3",
        "-Ccodegen-units=1",
    ],
    deps = all_crate_deps() + ["//plants-common:plants-common"],
    aliases = {
        "//plants-common:plants-common": "common",
    },
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "plants-daemon-debug",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
	"-Zunstable-options",
    ],
    deps = all_crate_deps() + ["//plants-common:plants-common"],
    aliases = {
        "//plants-common:plants-common": "common",
    },
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "plants-daemon-profiling",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
        "-Copt-level=3",
        "-Cdebuginfo=2",
        "-Ccodegen-units=64",
        "-Clto=off",
    ],
    deps = all_crate_deps() + ["//plants-common:plants-common"],
    aliases = {
        "//plants-common:plants-common": "common",
    },
    visibility = ["//visibility:public"],
)

rust_test(
    name = "plants-daemon_test",
    crate = ":plants-daemon",
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":plants-daemon"],
)
