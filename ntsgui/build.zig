const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const exe_mod = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Create executable
    const exe = b.addExecutable(.{
        .name = "ntsgui",
        .root_module = exe_mod,
    });

    // Link raylib via raylib-zig
    const raylib_dep = b.dependency("raylib_zig", .{
        .target = target,
        .optimize = optimize,
    });
    const raylib = raylib_dep.module("raylib");
    const raylib_artifact = raylib_dep.artifact("raylib");

    if (target.result.os.tag == .macos) {
        raylib_artifact.root_module.addCMacro("kAudioObjectPropertyElementMain", "kAudioObjectPropertyElementMaster");
    }

    if (target.result.os.tag == .macos) {
        if (b.sysroot) |sysroot| {
            const framework_path = b.fmt("{s}/System/Library/Frameworks", .{sysroot});
            exe.addFrameworkPath(.{ .cwd_relative = framework_path });
            raylib_artifact.addFrameworkPath(.{ .cwd_relative = framework_path });
        }
    }

    exe.root_module.addImport("raylib", raylib);
    exe.root_module.linkLibrary(raylib_artifact);
    exe.linkLibC();

    // Add include paths for nuklear headers
    const raylib_nuklear_dep = b.dependency("raylib-nuklear", .{});
    exe.addIncludePath(raylib_nuklear_dep.path("include"));
    exe.addCSourceFile(.{
        .file = b.path("src/nuklear_impl.c"),
        .flags = &.{},
    });

    b.installArtifact(exe);

    // Create run step
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the application");
    run_step.dependOn(&run_cmd.step);
}
