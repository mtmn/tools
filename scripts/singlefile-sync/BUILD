load("//bazel:local-deploy.bzl", "local_deploy")
load("@rules_shell//shell:sh_binary.bzl", "sh_binary")

package(default_visibility = ["//visibility:public"])

sh_binary(
    name = "singlefile-sync",
    srcs = ["singlefile-sync.sh"],
)

local_deploy(
    name = "deploy",
    srcs = [":singlefile-sync"]
)
