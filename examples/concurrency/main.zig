const std = @import("std");

const Ctx = struct {
    id: i32,
    out: *i64,
};

fn worker(arg: ?*anyopaque) ?*anyopaque {
    const ctx: *Ctx = @ptrCast(@alignCast(arg));
    var sum: i64 = 0;
    var j: i64 = 0;
    while (j < 1_000_000) : (j += 1) sum += j;
    ctx.out.* = sum;
    std.debug.print("worker {d} sum = {d}\n", .{ ctx.id, sum });
    return null;
}

pub fn main() !void {
    var results: [4]i64 = undefined;
    var contexts: [4]Ctx = undefined;
    var threads: [4]std.Thread = undefined;

    for (0..4) |i| {
        contexts[i] = .{ .id = @intCast(i), .out = &results[i] };
        threads[i] = try std.Thread.spawn(
            .{},
            worker,
            .{@as(?*anyopaque, @ptrCast(&contexts[i]))},
        );
    }
    for (threads) |t| t.join();

    var total: i64 = 0;
    for (results) |r| total += r;
    std.debug.print("total = {d}\n", .{total});
}
