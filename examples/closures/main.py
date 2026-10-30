from functools import reduce


def main():
    numbers = [5, 2, 9, 1, 7, 3]

    numbers.sort()
    print("sorted asc: ", numbers.copy())

    numbers.sort(reverse=True)
    print("sorted desc:", numbers)

    doubled = list(map(lambda x: x * 2, numbers))
    evens = list(filter(lambda x: x % 2 == 0, numbers))
    total = reduce(lambda acc, x: acc + x, numbers, 0)

    print("doubled:", doubled)
    print("evens:  ", evens)
    print("sum:    ", total)

    # lambda capturing environment
    factor = 3
    scale = lambda x: x * factor
    print("scale(5):", scale(5))

    print("apply(5, double):", apply(5, lambda x: x * 2))


def apply(x, f):
    return f(x)


if __name__ == "__main__":
    main()
