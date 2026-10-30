package main

import (
	"fmt"
	"sync"
)

func worker(id int) int64 {
	var sum int64
	for j := 0; j < 1_000_000; j++ {
		sum += int64(j)
	}
	fmt.Printf("worker %d sum = %d\n", id, sum)
	return sum
}

func main() {
	var wg sync.WaitGroup
	results := make([]int64, 4)

	for i := 0; i < 4; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			results[i] = worker(i)
		}(i)
	}
	wg.Wait()

	var total int64
	for _, r := range results {
		total += r
	}
	fmt.Println("total =", total)
}
