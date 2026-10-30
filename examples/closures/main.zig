const std = @import("std");

fn lessThan(_: void, a: i32, b: i32) bool {
    return a < b;
}

fn greaterThan(_: void, a: i32, b: i32) bool {
    return a > b;
}

fn apply(x: i32, f: *const fn (i32) i32) i32 {
    return f(x);
}

fn double(x: i32) i32 {
    return x * 2;
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    const stdout = std.io.getStdOut().writer();

    var numbers = [_]i32{ 5, 2, 9, 1, 7, 3 };

    std.mem.sort(i32, &numbers, {}, lessThan);
    try stdout.print("sorted asc:  {d}\n", .{numbers});

    std.mem.sort(i32, &numbers, {}, greaterThan);
    try stdout.print("sorted desc: {d}\n", .{numbers});

    // map
    var doubled = try allocator.alloc(i32, numbers.len);
    defer allocator.free(doubled);
    for (numbers, 0..) |x, i| doubled[i] = x * 2;

    // filter
    var evens = std.ArrayList(i32).init(allocator);
    defer evens.deinit();
    for (numbers) |x| if (x % 2 == 0) try evens.append(x);

    // reduce
    var sum: i32 = 0;
    for (numbers) |x| sum += x;

    try stdout.print("doubled: {d}\n", .{doubled});
    try stdout.print("evens:   {d}\n", .{evens.items});
    try stdout.print("sum:     {d}\n", .{sum});

    // Zig has no closures; pass context explicitly (here via a struct + fn ptr)
    const scale = struct {
        fn call(factor: i32, x: i32) i32 {
            return x * factor;
        }
    };
    try stdout.print("scale(5): {d}\n", .{scale.call(3, 5)});

    try stdout.print("apply(5, double): {d}\n", .{apply(5, double)});
}
