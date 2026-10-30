from __future__ import annotations

from typing import Optional


class Node:
    def __init__(self, val: int, next: Optional[Node] = None) -> None:
        self.val = val
        self.next = next


class LinkedList:
    def __init__(self) -> None:
        self.head: Optional[Node] = None

    def prepend(self, val: int) -> None:
        self.head = Node(val, self.head)

    def __len__(self) -> int:
        count = 0
        n = self.head
        while n:
            count += 1
            n = n.next
        return count

    def __str__(self) -> str:
        parts = []
        n = self.head
        while n:
            parts.append(str(n.val))
            n = n.next
        return " -> ".join(parts) + " -> nil"


def main() -> None:
    l = LinkedList()
    l.prepend(3)
    l.prepend(2)
    l.prepend(1)
    print("list:", l)
    print("len: ", len(l))


if __name__ == "__main__":
    main()
