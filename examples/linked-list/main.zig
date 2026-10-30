const std = @import("std");

const Node = struct {
    val: i32,
    next: ?*Node,
};

const LinkedList = struct {
    head: ?*Node,
    allocator: std.mem.Allocator,

    fn init(allocator: std.mem.Allocator) LinkedList {
        return .{ .head = null, .allocator = allocator };
    }

    fn deinit(self: *LinkedList) void {
        var current = self.head;
        while (current) |node| {
            current = node.next;
            self.allocator.destroy(node);
        }
    }

    fn prepend(self: *LinkedList, val: i32) !void {
        const node = try self.allocator.create(Node);
        node.* = .{ .val = val, .next = self.head };
        self.head = node;
    }

    fn len(self: *const LinkedList) usize {
        var count: usize = 0;
        var n = self.head;
        while (n) |node| : (n = node.next) count += 1;
        return count;
    }

    fn print(self: *const LinkedList, writer: anytype) !void {
        var n = self.head;
        while (n) |node| {
            try writer.print("{d} -> ", .{node.val});
            n = node.next;
        }
        try writer.print("nil\n", .{});
    }
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    const stdout = std.io.getStdOut().writer();

    var l = LinkedList.init(allocator);
    defer l.deinit();

    try l.prepend(3);
    try l.prepend(2);
    try l.prepend(1);

    try stdout.print("list: ", .{});
    try l.print(stdout);
    try stdout.print("len:  {d}\n", .{l.len()});
}
