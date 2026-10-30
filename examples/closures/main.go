package main

import (
	"fmt"
	"sort"
)

func main() {
	numbers := []int{5, 2, 9, 1, 7, 3}

	sort.Ints(numbers)
	fmt.Println("sorted asc: ", numbers)

	sort.Sort(sort.Reverse(sort.IntSlice(numbers)))
	fmt.Println("sorted desc:", numbers)

	// Go has no built-in map/filter/reduce, so we write them as higher-order funcs
	doubled := mapSlice(numbers, func(x int) int { return x * 2 })
	evens := filterSlice(numbers, func(x int) bool { return x%2 == 0 })
	sum := reduce(numbers, 0, func(acc, x int) int { return acc + x })

	fmt.Println("doubled:", doubled)
	fmt.Println("evens:  ", evens)
	fmt.Println("sum:    ", sum)

	// closure capturing environment
	factor := 3
	scale := func(x int) int { return x * factor }
	fmt.Println("scale(5):", scale(5))

	fmt.Println("apply(5, double):", apply(5, func(x int) int { return x * 2 }))
}

func apply(x int, f func(int) int) int { return f(x) }

func mapSlice(in []int, f func(int) int) []int {
	out := make([]int, len(in))
	for i, v := range in {
		out[i] = f(v)
	}
	return out
}

func filterSlice(in []int, f func(int) bool) []int {
	var out []int
	for _, v := range in {
		if f(v) {
			out = append(out, v)
		}
	}
	return out
}

func reduce(in []int, init int, f func(int, int) int) int {
	acc := init
	for _, v := range in {
		acc = f(acc, v)
	}
	return acc
}
