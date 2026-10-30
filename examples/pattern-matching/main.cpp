#include <iostream>
#include <memory>

// C++ has no native enums/ADTs; model with a tagged struct + switch
struct Expr {
    enum Tag { Num, Add, Mul } tag;
    int v;
    std::shared_ptr<Expr> a, b;
};

std::shared_ptr<Expr> num(int v) {
    auto e = std::make_shared<Expr>();
    e->tag = Expr::Num;
    e->v = v;
    return e;
}

std::shared_ptr<Expr> add(std::shared_ptr<Expr> a, std::shared_ptr<Expr> b) {
    auto e = std::make_shared<Expr>();
    e->tag = Expr::Add;
    e->a = a;
    e->b = b;
    return e;
}

std::shared_ptr<Expr> mul(std::shared_ptr<Expr> a, std::shared_ptr<Expr> b) {
    auto e = std::make_shared<Expr>();
    e->tag = Expr::Mul;
    e->a = a;
    e->b = b;
    return e;
}

int eval(const std::shared_ptr<Expr>& e) {
    switch (e->tag) {
        case Expr::Num: return e->v;
        case Expr::Add: return eval(e->a) + eval(e->b);
        case Expr::Mul: return eval(e->a) * eval(e->b);
    }
    return 0;
}

int main() {
    // (2 + 3) * 4
    auto e = mul(add(num(2), num(3)), num(4));
    std::cout << "eval = " << eval(e) << "\n";
}
