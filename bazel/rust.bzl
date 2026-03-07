load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("//bazel:local-deploy.bzl", "local_deploy")

def rust_app(
        name,
        srcs = None,
        deps = [],
        test_deps = [],
        edition = "2024",
        rustc_flags = [],
        visibility = ["//visibility:public"],
        **kwargs):
    """
    Defines a standard Rust application with optimized, debug, and profiling builds,
    along with unit tests and a local deployment target.
    """
    if srcs == None:
        srcs = native.glob(["src/**/*.rs"])

    common_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
    ] + rustc_flags

    rust_binary(
        name = name,
        srcs = srcs,
        edition = edition,
        rustc_flags = common_flags + [
            "-Copt-level=3",
            "-Ccodegen-units=1",
        ],
        visibility = visibility,
        deps = deps,
        **kwargs
    )

    rust_binary(
        name = name + "-debug",
        srcs = srcs,
        edition = edition,
        rustc_flags = common_flags,
        visibility = visibility,
        deps = deps,
        **kwargs
    )

    rust_binary(
        name = name + "-profiling",
        srcs = srcs,
        edition = edition,
        rustc_flags = common_flags + [
            "-Copt-level=3",
            "-Cdebuginfo=2",
            "-Ccodegen-units=64",
            "-Clto=off",
        ],
        visibility = visibility,
        deps = deps,
        **kwargs
    )

    rust_test(
        name = name + "_test",
        crate = ":" + name,
        edition = edition,
        rustc_flags = common_flags,
        deps = test_deps,
    )

    local_deploy(
        name = "deploy",
        srcs = [":" + name],
    )

def rust_lib(
        name,
        crate_name = None,
        srcs = None,
        deps = [],
        test_deps = [],
        edition = "2024",
        rustc_flags = [],
        visibility = ["//visibility:public"],
        **kwargs):
    """
    Defines a standard Rust library with optimized and debug builds,
    along with unit tests.
    """
    if srcs == None:
        srcs = native.glob(["src/**/*.rs"])

    if crate_name == None:
        crate_name = name

    common_flags = [
        "-Ctarget-cpu=native",
        "-Clink-arg=-fuse-ld=mold",
    ] + rustc_flags

    rust_library(
        name = name,
        crate_name = crate_name,
        srcs = srcs,
        edition = edition,
        rustc_flags = common_flags + [
            "-Copt-level=3",
            "-Ccodegen-units=1",
        ],
        visibility = visibility,
        deps = deps,
        **kwargs
    )

    rust_test(
        name = name + "_test",
        crate = ":" + name,
        edition = edition,
        rustc_flags = common_flags,
        deps = test_deps,
    )
