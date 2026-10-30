package main

import "fmt"

// Go has no enums/ADTs; model with an interface + type switch
type Expr interface {
	eval() int
}

type Num struct{ v int }
type Add struct{ a, b Expr }
type Mul struct{ a, b Expr }

func (n Num) eval() int { return n.v }
func (a Add) eval() int { return a.a.eval() + a.b.eval() }
func (m Mul) eval() int { return m.a.eval() * m.b.eval() }

func main() {
	// (2 + 3) * 4
	var e Expr = Mul{Add{Num{2}, Num{3}}, Num{4}}
	fmt.Println("eval =", e.eval())
}
