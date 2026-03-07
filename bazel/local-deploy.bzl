"""
Rule for local *nix deployment of a binary build artifact.

This is useful for local development but is not intended for production deployments.
"""

def _local_deploy_impl(ctx):
    target = ctx.attr.target
    copy_runfiles = ctx.attr.copy_runfiles

    # We use a set to avoid duplicates if multiple targets provide the same file
    files_map = {}
    transitive_runfiles = []

    for t in ctx.attr.srcs:
        if DefaultInfo in t:
            info = t[DefaultInfo]
            transitive_runfiles.append(info.default_runfiles)

            # If it's an executable, we prefer that.
            if info.files_to_run.executable:
                 exe = info.files_to_run.executable
                 files_map[exe.short_path] = exe
            else:
                 # Otherwise take all files
                 for f in info.files.to_list():
                     files_map[f.short_path] = f

    # Sort for determinism
    sorted_paths = sorted(files_map.keys())

    final_files = []

    script_content = [
        "#!/bin/bash",
        "set -e",
        "TARGET_DIR=%s" % target,
        "sudo mkdir -p \"$TARGET_DIR\"",
        "",
        "# Locate the runfiles directory of this deploy script",
        "if [ -n \"$RUNFILES_DIR\" ]; then",
        "    RUNFILES_SOURCE=\"$RUNFILES_DIR\"",
        "elif [ -d \"${0}.runfiles\" ]; then",
        "    RUNFILES_SOURCE=\"${0}.runfiles\"",
        "else",
        "    RUNFILES_SOURCE=\"\"",
        "fi",
        ""
    ]

    for path in sorted_paths:
        # If "foo.sh" and "foo" exist, skip "foo.sh".
        is_shadowed = False
        for ext in [".sh", ".py"]:
            if path.endswith(ext):
                wrapper = path[:-len(ext)]
                if wrapper in files_map:
                    is_shadowed = True
                    break

        if is_shadowed:
            continue

        f = files_map[path]
        final_files.append(f)

        # Add copy command
        script_content.extend([
            "SRC=\"%s\"" % f.short_path,
            "REAL=$(readlink -f \"$SRC\")",
            "echo \"Copying $SRC to $TARGET_DIR\"",
            "sudo cp \"$REAL\" \"$TARGET_DIR/$(basename \"$SRC\")\"",
        ])

        # Copy runfiles if enabled
        if copy_runfiles:
            script_content.extend([
                "if [ -n \"$RUNFILES_SOURCE\" ]; then",
                "    DEST_RUNFILES=\"$TARGET_DIR/$(basename \"$SRC\").runfiles\"",
                "    echo \"Copying runfiles to $DEST_RUNFILES\"",
                "    sudo rm -rf \"$DEST_RUNFILES\"",
                "    sudo cp -r \"$RUNFILES_SOURCE\" \"$DEST_RUNFILES\"",
                "fi",
                "",
            ])

    ctx.actions.write(
        output = ctx.outputs.executable,
        is_executable = True,
        content = "\n".join(script_content)
    )

    runfiles = ctx.runfiles(files = final_files)
    for r in transitive_runfiles:
        runfiles = runfiles.merge(r)

    return DefaultInfo(
        executable = ctx.outputs.executable,
        runfiles = runfiles,
    )

local_deploy = rule(
    executable = True,
    implementation = _local_deploy_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "target": attr.string(default = "/opt/tools"),
        "copy_runfiles": attr.bool(default = False),
    },
)
