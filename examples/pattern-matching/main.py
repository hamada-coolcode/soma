from dataclasses import dataclass


class Expr:
    pass


@dataclass
class Num(Expr):
    v: int


@dataclass
class Add(Expr):
    a: Expr
    b: Expr


@dataclass
class Mul(Expr):
    a: Expr
    b: Expr


def eval_expr(e: Expr) -> int:
    match e:  # structural pattern matching (Python 3.10+)
        case Num(v):
            return v
        case Add(a, b):
            return eval_expr(a) + eval_expr(b)
        case Mul(a, b):
            return eval_expr(a) * eval_expr(b)
    raise TypeError(f"unknown expr: {e!r}")


def main():
    # (2 + 3) * 4
    e = Mul(Add(Num(2), Num(3)), Num(4))
    print("eval =", eval_expr(e))


if __name__ == "__main__":
    main()
