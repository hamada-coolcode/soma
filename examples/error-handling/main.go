package main

import (
	"errors"
	"fmt"
	"strconv"
)

func parseAndSum(values []string) (int, error) {
	total := 0
	for _, v := range values {
		n, err := strconv.Atoi(v)
		if err != nil {
			return 0, fmt.Errorf("bad int %q: %w", v, err)
		}
		total += n
	}
	return total, nil
}

func safeDiv(a, b int) (int, error) {
	if b == 0 {
		return 0, errors.New("division by zero")
	}
	return a / b, nil
}

func main() {
	good := []string{"10", "20", "30"}
	bad := []string{"10", "oops", "30"}

	if total, err := parseAndSum(good); err != nil {
		fmt.Println("error:", err)
	} else {
		fmt.Println("sum =", total)
		if q, err := safeDiv(total, 3); err != nil {
			fmt.Println("error:", err)
		} else {
			fmt.Println("sum / 3 =", q)
		}
		if _, err := safeDiv(total, 0); err != nil {
			fmt.Println("error:", err)
		}
	}

	if _, err := parseAndSum(bad); err != nil {
		fmt.Println("error:", err)
	}
}
