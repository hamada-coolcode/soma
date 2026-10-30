const std = @import("std");

// Zig tagged unions + switch = exhaustive pattern matching
const Expr = union(enum) {
    num: i32,
    add: struct { a: *const Expr, b: *const Expr },
    mul: struct { a: *const Expr, b: *const Expr },
};

fn eval(e: *const Expr) i32 {
    switch (e.*) {
        .num => |v| return v,
        .add => |p| return eval(p.a) + eval(p.b),
        .mul => |p| return eval(p.a) * eval(p.b),
    }
}

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();

    // (2 + 3) * 4
    const two = Expr{ .num = 2 };
    const three = Expr{ .num = 3 };
    const four = Expr{ .num = 4 };
    const sum = Expr{ .add = .{ .a = &two, .b = &three } };
    const product = Expr{ .mul = .{ .a = &sum, .b = &four } };

    try stdout.print("eval = {d}\n", .{eval(&product)});
}
