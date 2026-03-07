load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _ntsgui_deps_impl(ctx):
    http_archive(
        name = "ntsgui_raylib",
        build_file = "//ntsgui/deps/raylib:raylib.BUILD",
        sha256 = "2e205f6d373393019aefe2f5bbca2e5df41e63b384f9dffa148431dfb7f5e63f",
        strip_prefix = "raylib-3e926d65a0dab63b098a9161075a2cf634d3ef23",
        urls = ["https://github.com/raysan5/raylib/archive/3e926d65a0dab63b098a9161075a2cf634d3ef23.tar.gz"],
    )
    http_archive(
        name = "ntsgui_raylib_nuklear",
        build_file = "//ntsgui/deps/raylib_nuklear:raylib_nuklear.BUILD",
        sha256 = "d0566d8391e9866df6af2c24649e0cecfbba0ea5821cf4198e4fba84e1d7a628",
        strip_prefix = "raylib-nuklear-ccb16d9b91517387701f57976952c0d46b1ed6a1",
        urls = ["https://github.com/RobLoach/raylib-nuklear/archive/ccb16d9b91517387701f57976952c0d46b1ed6a1.tar.gz"],
        patch_cmds = [
            "sed -i 's/RAYLIB_VERSION_MINOR == 0/0/g' include/raylib-nuklear.h",
        ],
    )
    http_archive(
        name = "ntsgui_raylib_zig",
        build_file = "//ntsgui/deps/raylib_zig:raylib_zig.BUILD",
        sha256 = "ecc16c9bb98fd0853fe1cae677b27e34fb7d56fcda506fd1fe0e9d944d93bb7f",
        strip_prefix = "raylib-zig-cd71c85d571027ac8033357f83b124ee051825b3",
        urls = ["https://github.com/raylib-zig/raylib-zig/archive/cd71c85d571027ac8033357f83b124ee051825b3.tar.gz"],
    )

ntsgui_deps = module_extension(implementation = _ntsgui_deps_impl)
