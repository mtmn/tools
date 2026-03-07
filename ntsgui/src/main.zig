const std = @import("std");
const nts = @import("nts.zig");

const rl = @import("raylib");

const c = @cImport({
    @cInclude("raylib-nuklear.h");
});

// Embed assets
const FONT_DATA = @embedFile("assets/font.ttf");
// Output buffer for GUI console
var output_buffer: std.ArrayListUnmanaged(u8) = .{};
var output_mutex: std.Thread.Mutex = .{};
var is_running: bool = false;
var selected_line: ?usize = null;

fn runNTSTask(allocator: std.mem.Allocator, url: []const u8) !void {
    defer allocator.free(url);

    {
        output_mutex.lock();
        is_running = true;
        selected_line = null;
        output_buffer.clearRetainingCapacity();
        output_mutex.unlock();
    }

    const result = nts.fetchTracklist(allocator, url);

    output_mutex.lock();
    defer output_mutex.unlock();
    is_running = false;

    if (result) |data| {
        defer allocator.free(data);
        output_buffer.appendSlice(allocator, data) catch {};
    } else |err| {
        const msg = std.fmt.allocPrint(allocator, "Error: {any}\n", .{err}) catch return;
        defer allocator.free(msg);
        output_buffer.appendSlice(allocator, msg) catch {};
    }
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    defer output_buffer.deinit(allocator);

    const screenWidth = 1200;
    const screenHeight = 800;

    rl.setConfigFlags(rl.ConfigFlags{ .window_resizable = true });
    rl.initWindow(screenWidth, screenHeight, "ntsgui");
    defer rl.closeWindow();

    rl.setTargetFPS(60);

    const fontSize = 22;

    // Load embedded font
    var font_loaded = false;
    const font = rl.loadFontFromMemory(".ttf", FONT_DATA, fontSize, null) catch blk: {
        break :blk try rl.getFontDefault();
    };
    if (font.texture.id != 0) {
        font_loaded = true;
    }

    const ctx = c.InitNuklearEx(@bitCast(font), fontSize);
    defer {
        c.UnloadNuklear(ctx);
        if (font_loaded) {
            rl.unloadFont(font);
        }
    }

    var url_buf: [1024]u8 = undefined;
    @memset(&url_buf, 0);
    var url_len: c_int = 0;

    while (!rl.windowShouldClose()) {
        c.UpdateNuklear(ctx);

        if (c.nk_begin(ctx, "NTS Downloader", c.nk_rect(0, 0, @floatFromInt(rl.getScreenWidth()), @floatFromInt(rl.getScreenHeight())), c.NK_WINDOW_NO_SCROLLBAR)) {
            // Input Row
            c.nk_layout_row_dynamic(ctx, 30, 1);

            var should_run = false;

            // Trigger run on Ctrl+V OR Enter
            if ((rl.isKeyDown(.left_control) and rl.isKeyPressed(.v)) or rl.isKeyPressed(.enter)) {
                should_run = true;
            }

            url_len = @intCast(std.mem.len(@as([*:0]u8, @ptrCast(&url_buf))));
            _ = c.nk_edit_string(ctx, c.NK_EDIT_FIELD | c.NK_EDIT_SIG_ENTER | c.NK_EDIT_CLIPBOARD, &url_buf, &url_len, url_buf.len, c.nk_filter_default);
            url_buf[@intCast(url_len)] = 0;

            if (should_run and !is_running) {
                if (url_len > 0) {
                    const url_slice = std.mem.span(@as([*:0]u8, @ptrCast(&url_buf)));
                    if (allocator.dupe(u8, url_slice)) |url_copy| {
                        const thread: ?std.Thread = std.Thread.spawn(.{}, runNTSTask, .{ allocator, url_copy }) catch blk: {
                            allocator.free(url_copy);
                            output_mutex.lock();
                            output_buffer.appendSlice(allocator, "Error: Failed to spawn thread.\n") catch {};
                            output_mutex.unlock();
                            break :blk null;
                        };
                        if (thread) |t| {
                            t.detach();
                        }
                    } else |_| {
                        output_mutex.lock();
                        output_buffer.appendSlice(allocator, "Error: Out of memory.\n") catch {};
                        output_mutex.unlock();
                    }
                }
            }

            // Output Display: Scaled to window height
            const used_height = 45; // 30px input + 15px padding
            const screen_h = rl.getScreenHeight();
            var group_height: i32 = 400; // Fallback
            if (screen_h > used_height) {
                group_height = @intCast(screen_h - used_height);
            }

            c.nk_layout_row_dynamic(ctx, @floatFromInt(group_height), 1);

            // Removed NK_WINDOW_TITLE, set title to empty string
            if (c.nk_group_begin(ctx, "", c.NK_WINDOW_BORDER | c.NK_WINDOW_NO_SCROLLBAR)) {
                output_mutex.lock();
                defer output_mutex.unlock();

                if (output_buffer.items.len > 0) {
                    var it = std.mem.splitScalar(u8, output_buffer.items, '\n');
                    var i: usize = 0;
                    var buf: [4096]u8 = undefined;

                    while (it.next()) |line| {
                        // Skip empty lines to avoid Nuklear assertion failures
                        if (line.len == 0) {
                            continue;
                        }

                        c.nk_layout_row_dynamic(ctx, 30, 1);

                        const len = @min(line.len, buf.len - 1);
                        @memcpy(buf[0..len], line[0..len]);
                        buf[len] = 0;

                        var is_selected: bool = (selected_line == i);
                        if (c.nk_selectable_label(ctx, &buf, c.NK_TEXT_LEFT, @ptrCast(&is_selected))) {
                            selected_line = i;
                            // Copy to clipboard on click
                            rl.setClipboardText(std.mem.span(@as([*:0]u8, @ptrCast(&buf))));
                        }
                        i += 1;
                    }
                } else {
                    c.nk_layout_row_dynamic(ctx, 30, 1);
                    c.nk_label(ctx, "", c.NK_TEXT_LEFT);
                }

                c.nk_group_end(ctx);
            }
        }
        c.nk_end(ctx);

        rl.beginDrawing();
        defer rl.endDrawing();
        rl.clearBackground(rl.Color.white);
        c.DrawNuklear(ctx);
    }
}
