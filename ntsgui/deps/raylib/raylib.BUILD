load("@rules_cc//cc:defs.bzl", "cc_library")

cc_library(
    name="raylib",
    srcs=glob(
        ["src/*.c", "src/external/**/*.h"], exclude=["src/rglfw.c"], allow_empty=True
    )
    + select(
        {
            "@platforms//os:linux": ["src/rglfw.c"],
            "//conditions:default": [],
        }
    ),
    hdrs=glob(["src/*.h"], allow_empty=True),
    textual_hdrs=glob(
        ["src/external/**/*.c", "src/platforms/**/*.c"], allow_empty=True
    ),
    includes=["src", "src/external/glfw/include"],
    defines=["PLATFORM_DESKTOP", "_GLFW_X11"],
    linkopts=[
        "-target",
        "x86_64-linux-gnu.2.38",
        "-L/usr/lib",
        "-lX11",
        "-lGL",
        "-lm",
        "-lpthread",
        "-ldl",
        "-lrt",
        "-lXrandr",
        "-lXinerama",
        "-lXi",
        "-lXcursor",
        "-lX11-xcb",
        "-lxcb",
    ],
    visibility=["//visibility:public"],
)
