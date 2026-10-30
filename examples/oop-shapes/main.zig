const std = @import("std");

// Zig has no inheritance; a tagged union models the "shape family" statically
const Shape = union(enum) {
    circle: f64, // radius
    rectangle: struct { width: f64, height: f64 },

    fn area(self: Shape) f64 {
        switch (self) {
            .circle => |r| return std.math.pi * r * r,
            .rectangle => |r| return r.width * r.height,
        }
    }

    fn name(self: Shape) []const u8 {
        switch (self) {
            .circle => return "circle",
            .rectangle => return "rectangle",
        }
    }
};

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();

    const shapes = [_]Shape{
        .{ .circle = 2.0 },
        .{ .rectangle = .{ .width = 3.0, .height = 4.0 } },
    };

    for (shapes) |s| {
        try stdout.print("{s} area = {d:.2}\n", .{ s.name(), s.area() });
    }
}
