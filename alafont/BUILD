load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")

rust_binary(
    name = "alafont-debug",
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
    name = "alafont",
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
    name = "alafont-profiling",
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
    name = "alafont_test",
    crate = ":alafont",
    edition = "2024",
    rustc_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
    ],
)

local_deploy(
    name = "deploy",
    srcs = [":alafont"],
)
