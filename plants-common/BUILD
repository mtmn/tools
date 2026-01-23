load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")

rust_library(
    name = "plants-common",
    srcs = glob(["src/**/*.rs"]),
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
    ],
    deps = all_crate_deps(),
    visibility = ["//visibility:public"],
)
