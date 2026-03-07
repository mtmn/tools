const std = @import("std");
const http = std.http;
const json = std.json;
const fs = std.fs;
const flate = std.compress.flate;

pub const Track = struct {
    artist: ?[]const u8,
    title: ?[]const u8,
};

pub const TracklistResult = struct {
    results: []Track,
};

pub const Embeds = struct {
    tracklist: TracklistResult,
};

pub const EpisodeData = struct {
    name: ?[]const u8 = null,
    embeds: Embeds,
};

// Error set for NTS operations
pub const NTSError = error{
    InvalidURL,
    NetworkError,
    ParseError,
    FileError,
    OutOfMemory,
} || std.http.Client.RequestError || std.mem.Allocator.Error || std.fmt.ParseIntError || std.Uri.ParseError || std.fs.File.OpenError || std.fs.File.WriteError || std.json.ParseError(std.json.Scanner) || std.Io.Reader.LimitedAllocError || flate.Container.Error || flate.Decompress.Error;

pub fn extractPath(url: []const u8) ?[]const u8 {
    // Basic URL parsing to find /shows/SHOW_NAME/episodes/EPISODE_NAME
    const shows_idx = std.mem.indexOf(u8, url, "shows/") orelse return null;
    return url[shows_idx..];
}

pub fn extractEpisodeName(path: []const u8) ?[]const u8 {
    if (std.mem.lastIndexOf(u8, path, "/")) |idx| {
        if (idx + 1 < path.len) {
            return path[idx + 1 ..];
        }
    }
    return null;
}

pub fn fetchTracklist(allocator: std.mem.Allocator, url: []const u8) ![]u8 {
    const path = extractPath(url) orelse return NTSError.InvalidURL;
    const episode_name = extractEpisodeName(path) orelse return NTSError.InvalidURL;

    // Construct API URL
    const api_url = try std.fmt.allocPrint(allocator, "https://www.nts.live/api/v2/{s}", .{path});
    defer allocator.free(api_url);

    var client = http.Client{ .allocator = allocator };
    defer client.deinit();

    // Parse uri
    const uri = try std.Uri.parse(api_url);

    var req = try client.request(.GET, uri, .{
        .headers = .{
            .user_agent = .{ .override = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36" },
        },
    });
    defer req.deinit();

    try req.sendBodiless();

    var header_buffer: [4096]u8 = undefined;
    var response = req.receiveHead(&header_buffer) catch |err| {
        if (err == error.HttpHeadTooLarge) return NTSError.NetworkError;
        return err;
    };

    if (response.head.status != .ok) {
        return NTSError.NetworkError;
    }

    var transfer_buffer: [4096]u8 = undefined;
    var reader = response.reader(&transfer_buffer);

    // Read response body using std.Io.Reader.allocRemaining
    // Use a reasonable limit (e.g. 10MB) to prevent OOM on huge responses
    const raw_body = try reader.allocRemaining(allocator, std.Io.Limit.limited(10 * 1024 * 1024));
    defer allocator.free(raw_body);

    var body_to_parse = raw_body;
    var decompressed_body: []u8 = &.{};

    if (response.head.content_encoding == .gzip) {
        var fbs = std.io.fixedBufferStream(raw_body);
        var fbs_reader = fbs.reader();

        var adapter_buffer: [4096]u8 = undefined;
        // adaptToNewApi returns adapter with .new_interface (Reader struct)
        var adapter = fbs_reader.adaptToNewApi(&adapter_buffer);

        // Decompress GZIP
        var window_buffer: [flate.max_window_len]u8 = undefined;
        // init takes *Reader (so &adapter.new_interface)
        var decompressor = flate.Decompress.init(&adapter.new_interface, .gzip, &window_buffer);

        decompressed_body = try decompressor.reader.allocRemaining(allocator, std.Io.Limit.limited(10 * 1024 * 1024));
        body_to_parse = decompressed_body;
    }
    defer if (decompressed_body.len > 0) allocator.free(decompressed_body);

    // Parse JSON
    const parsed = try json.parseFromSlice(EpisodeData, allocator, body_to_parse, .{ .ignore_unknown_fields = true });
    defer parsed.deinit();

    const tracks = parsed.value.embeds.tracklist.results;

    var output = std.ArrayList(u8){};
    errdefer output.deinit(allocator);

    // Save to file
    const filename = try std.fmt.allocPrint(allocator, "{s}.txt", .{episode_name});
    defer allocator.free(filename);

    var file = try fs.cwd().createFile(filename, .{});
    defer file.close();

    for (tracks) |track| {
        const artist = track.artist orelse "Unknown Artist";
        const title = track.title orelse "Unknown Title";

        const line = try std.fmt.allocPrint(allocator, "{s} - {s}\n", .{ artist, title });
        defer allocator.free(line);

        try output.appendSlice(allocator, line);
        try file.writeAll(line);
    }

    return output.toOwnedSlice(allocator);
}
