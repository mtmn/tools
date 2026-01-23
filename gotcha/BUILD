# note(mtmn): this currently fails on bazel but works fine with `cargo build` 

# load("//bazel:local-deploy.bzl", "local_deploy")
# load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test", "rust_library")
# load("@crates//:defs.bzl", "all_crate_deps")

# rust_binary(
#     name = "gotcha",
#     srcs = glob(["src/**/*.rs"]),
#     edition = "2024",
#     rustc_flags = [
#         "-Ctarget-cpu=native",
#         "-Clink-arg=-fuse-ld=mold",
#         "-Copt-level=3",
#         "-Ccodegen-units=1",
#     ],
#     deps = all_crate_deps(),
#     visibility = ["//visibility:public"],
# )

# rust_binary(
#     name = "gotcha-debug",
#     srcs = glob(["src/**/*.rs"]),
#     edition = "2024",
#     rustc_flags = [
#         "-Ctarget-cpu=native",
#         "-Clink-arg=-fuse-ld=mold",
#     ],
#     deps = all_crate_deps(),
#     visibility = ["//visibility:public"],
# )

# rust_binary(
#     name = "gotcha-profiling",
#     srcs = glob(["src/**/*.rs"]),
#     edition = "2024",
#     rustc_flags = [
#         "-Ctarget-cpu=native",
#         "-Clink-arg=-fuse-ld=mold",
#         "-Copt-level=3",
#         "-Cdebuginfo=2",
#         "-Ccodegen-units=64",
#         "-Clto=off",
#     ],
#     deps = all_crate_deps(),
#     visibility = ["//visibility:public"],
# )

# rust_test(
#     name = "gotcha_test",
#     crate = ":gotcha",
#     edition = "2024",
#     rustc_flags = [
#         "-Ctarget-cpu=native",
#         "-Clink-arg=-fuse-ld=mold",
#     ],
# )

# local_deploy(
#     name = "deploy",
#     srcs = [":gotcha"],
# )
