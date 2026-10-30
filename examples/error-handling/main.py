def parse_and_sum(values: list[str]) -> int:
    total = 0
    for v in values:
        try:
            total += int(v)
        except ValueError:
            raise ValueError(f"bad int {v!r}")
    return total


def safe_div(a: int, b: int) -> int:
    if b == 0:
        raise ZeroDivisionError("division by zero")
    return a // b


def main() -> None:
    good = ["10", "20", "30"]
    bad = ["10", "oops", "30"]

    try:
        total = parse_and_sum(good)
        print("sum =", total)
        print("sum / 3 =", safe_div(total, 3))
        print("sum / 0 =", safe_div(total, 0))
    except (ZeroDivisionError, ValueError) as e:
        print("error:", e)

    try:
        parse_and_sum(bad)
    except ValueError as e:
        print("error:", e)


if __name__ == "__main__":
    main()
