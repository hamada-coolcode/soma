const std = @import("std");
const parseInt = std.fmt.parseInt;

// Zig error unions: the return type carries the set of possible errors
const SumError = error{ BadInt, DivisionByZero };

fn parseAndSum(values: []const []const u8) SumError!i32 {
    var total: i32 = 0;
    for (values) |v| {
        const n = parseInt(i32, v, 10) catch return error.BadInt;
        total += n;
    }
    return total;
}

fn safeDiv(a: i32, b: i32) SumError!i32 {
    if (b == 0) return error.DivisionByZero;
    return @divTrunc(a, b);
}

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();

    const good = [_][]const u8{ "10", "20", "30" };
    const bad = [_][]const u8{ "10", "oops", "30" };

    if (parseAndSum(&good)) |total| {
        try stdout.print("sum = {d}\n", .{total});

        if (safeDiv(total, 3)) |q| {
            try stdout.print("sum / 3 = {d}\n", .{q});
        } else |err| {
            try stdout.print("error: {}\n", .{err});
        }

        if (safeDiv(total, 0)) |q| {
            try stdout.print("sum / 0 = {d}\n", .{q});
        } else |err| {
            try stdout.print("error: {}\n", .{err});
        }
    } else |err| {
        try stdout.print("error: {}\n", .{err});
    }

    if (parseAndSum(&bad)) |_| {
        // no error
    } else |err| {
        try stdout.print("error: {}\n", .{err});
    }
}
