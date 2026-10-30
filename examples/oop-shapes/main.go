package main

import "fmt"

type Shape interface {
	Area() float64
	Name() string
}

type Circle struct{ radius float64 }
type Rectangle struct{ width, height float64 }

func (c Circle) Area() float64 { return 3.141592653589793 * c.radius * c.radius }
func (c Circle) Name() string  { return "circle" }
func (r Rectangle) Area() float64 {
	return r.width * r.height
}
func (r Rectangle) Name() string { return "rectangle" }

func main() {
	shapes := []Shape{Circle{2.0}, Rectangle{3.0, 4.0}}
	for _, s := range shapes {
		fmt.Printf("%s area = %.2f\n", s.Name(), s.Area())
	}
}
