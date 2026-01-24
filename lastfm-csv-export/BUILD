load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")

rust_binary(
    name = "lastfm-csv-export-debug",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
	"-Zunstable-options",
    ],
    deps = all_crate_deps(),
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "lastfm-csv-export",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
        "-Copt-level=3",
        "-Ccodegen-units=1",
    ],
    deps = all_crate_deps(),
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "lastfm-csv-export-profiling",
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
    deps = all_crate_deps(),
    visibility = ["//visibility:public"],
)

rust_test(
    name = "lastfm-csv-export_test",
    crate = ":lastfm-csv-export",
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":lastfm-csv-export"],
)
