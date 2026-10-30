import math
from abc import ABC, abstractmethod


class Shape(ABC):
    @abstractmethod
    def area(self) -> float: ...

    @abstractmethod
    def name(self) -> str: ...


class Circle(Shape):
    def __init__(self, radius: float) -> None:
        self.radius = radius

    def area(self) -> float:
        return math.pi * self.radius**2

    def name(self) -> str:
        return "circle"


class Rectangle(Shape):
    def __init__(self, width: float, height: float) -> None:
        self.width = width
        self.height = height

    def area(self) -> float:
        return self.width * self.height

    def name(self) -> str:
        return "rectangle"


def main() -> None:
    shapes: list[Shape] = [Circle(2.0), Rectangle(3.0, 4.0)]
    for s in shapes:
        print(f"{s.name()} area = {s.area():.2f}")


if __name__ == "__main__":
    main()
