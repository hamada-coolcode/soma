const std = @import("std");

// Zig generics = comptime type parameter that builds a distinct type
fn Stack(comptime T: type) type {
    return struct {
        const Self = @This();

        items: std.ArrayList(T),
        allocator: std.mem.Allocator,

        fn init(allocator: std.mem.Allocator) Self {
            return .{ .items = std.ArrayList(T).init(allocator), .allocator = allocator };
        }

        fn deinit(self: *Self) void {
            self.items.deinit();
        }

        fn push(self: *Self, item: T) !void {
            try self.items.append(item);
        }

        fn pop(self: *Self) ?T {
            return self.items.pop();
        }

        fn peek(self: *Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.items[self.items.items.len - 1];
        }

        fn len(self: *Self) usize {
            return self.items.items.len;
        }
    };
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    const stdout = std.io.getStdOut().writer();

    var s = Stack(i32).init(allocator);
    defer s.deinit();
    try s.push(1);
    try s.push(2);
    try s.push(3);
    try stdout.print("peek: {d}\n", .{s.peek().?});
    try stdout.print("pop:  {d}\n", .{s.pop().?});
    try stdout.print("len:  {d}\n", .{s.len()});

    var words = Stack([]const u8).init(allocator);
    defer words.deinit();
    try words.push("hello");
    try words.push("world");
    try stdout.print("pop:  {s}\n", .{words.pop().?});
}
