// speediness: download and upload speed benchmark

const std = @import("std");

fn mbps(bytes: usize, ns: u64) f64 {
    const secs = @as(f64, @floatFromInt(ns)) / 1e9;
    if (secs < 0.001) return 0;
    return (@as(f64, @floatFromInt(bytes)) / secs) / (1024.0 * 1024.0);
}

// ── Download ──────────────────────────────────────────────────────────────────

fn downloadTest(allocator: std.mem.Allocator) !void {
    const url = "https://proof.ovh.net/files/100Mb.dat";

    var client: std.http.Client = .{
        .allocator = allocator,
    };
    defer client.deinit();

    const uri = try std.Uri.parse(url);

    var req = try client.request(.GET, uri, .{});
    defer req.deinit();

    try req.sendBodiless();

    var redirect_buffer: [8192]u8 = undefined;
    var response = try req.receiveHead(&redirect_buffer);

    if (response.head.status != .ok) {
        std.debug.print("{}\n", .{response.head.status});
        return error.HttpError;
    }

    var buf: [131072]u8 = undefined;
    var total: usize = 0;
    var iter: usize = 0;
    const start = try std.time.Instant.now();
    var reader = response.reader(&.{});

    while (true) {
        const n = reader.readSliceShort(&buf) catch break;
        if (n == 0) break;
        total += n;
        iter += 1;
        if (iter % 8 == 0) {
            const now = try std.time.Instant.now();
            const speed = mbps(total, now.since(start));
            std.debug.print("\r{d:6.1} MB {d:6.2} MB/s {d:6.1} Mbps ", .{
                @as(f64, @floatFromInt(total)) / 1e6,
                speed,
                speed * 8,
            });
        }
    }

    const end = try std.time.Instant.now();
    const speed = mbps(total, end.since(start));
    const secs = @as(f64, @floatFromInt(end.since(start))) / 1e9;
    std.debug.print("\n{d:.1} MB in {d:.2}s => {d:.2} MB/s  ({d:.1} Mbps)\n", .{
        @as(f64, @floatFromInt(total)) / 1e6, secs, speed, speed * 8,
    });
}

// ── Upload ────────────────────────────────────────────────────────────────────

fn uploadTest(allocator: std.mem.Allocator) !void {
    const url = "https://speed.cloudflare.com/__upload";
    const upload_bytes: usize = 20 * 1024 * 1024; // 20 MB

    var client: std.http.Client = .{
        .allocator = allocator,
    };
    defer client.deinit();

    const uri = try std.Uri.parse(url);

    var req = try client.request(.POST, uri, .{
        .headers = .{
            .content_type = .{ .override = "application/octet-stream" },
        },
    });
    defer req.deinit();
    req.transfer_encoding = .{ .content_length = upload_bytes };

    var body = try req.sendBodyUnflushed(&.{});

    // Stream payload in 128 KB chunks
    const chunk_size: usize = 131072;
    const chunk = try allocator.alloc(u8, chunk_size);
    defer allocator.free(chunk);
    @memset(chunk, 0x55);

    var sent: usize = 0;
    const start = try std.time.Instant.now();

    while (sent < upload_bytes) {
        const to_send = @min(chunk_size, upload_bytes - sent);
        try body.writer.writeAll(chunk[0..to_send]);
        sent += to_send;

        const now = try std.time.Instant.now();
        const speed = mbps(sent, now.since(start));
        std.debug.print("\r {d:6.1} MB {d:6.2} MB/s {d:6.1} Mbps ", .{
            @as(f64, @floatFromInt(sent)) / 1e6,
            speed,
            speed * 8,
        });
    }

    try body.end();
    try req.connection.?.flush();

    // Finish timing after data is flushed
    const end = try std.time.Instant.now();

    var redirect_buffer: [8192]u8 = undefined;
    _ = try req.receiveHead(&redirect_buffer);

    const speed = mbps(sent, end.since(start));
    const secs = @as(f64, @floatFromInt(end.since(start))) / 1e9;
    std.debug.print("\n{d:.1} MB in {d:.2}s => {d:.2} MB/s  ({d:.1} Mbps)\n", .{
        @as(f64, @floatFromInt(sent)) / 1e6, secs, speed, speed * 8,
    });
}

// ── Main ──────────────────────────────────────────────────────────────────────

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    downloadTest(allocator) catch |err| {
        std.debug.print("Download error: {}\n", .{err});
    };

    uploadTest(allocator) catch |err| {
        std.debug.print("Upload error: {}\n", .{err});
    };
}
