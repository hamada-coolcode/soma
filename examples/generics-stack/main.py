from typing import Generic, List, Optional, TypeVar

T = TypeVar("T")


class Stack(Generic[T]):
    def __init__(self) -> None:
        self._items: List[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> Optional[T]:
        return self._items.pop() if self._items else None

    def peek(self) -> Optional[T]:
        return self._items[-1] if self._items else None

    def __len__(self) -> int:
        return len(self._items)


def main() -> None:
    s: Stack[int] = Stack()
    s.push(1)
    s.push(2)
    s.push(3)
    print("peek:", s.peek())
    print("pop: ", s.pop())
    print("len: ", len(s))

    # same generic type reused with a different T
    words: Stack[str] = Stack()
    words.push("hello")
    words.push("world")
    print("pop: ", words.pop())


if __name__ == "__main__":
    main()
