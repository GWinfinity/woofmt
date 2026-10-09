package main

import (
	"fmt"
	"strings"
)

// Process demonstrates control flow, switch, defer, and generics.
func Process(items []string, mode string) (result []string, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("panic: %v", r)
		}
	}()

	switch mode {
	case "upper":
		for _, it := range items {
			result = append(result, strings.ToUpper(it))
		}
	case "skip":
		for i := 0; i < len(items); i++ {
			if i%2 == 0 {
				continue
			}
			result = append(result, items[i])
		}
	default:
		return nil, fmt.Errorf("unknown mode: %s", mode)
	}
	return result, nil
}

type Number interface {
	~int | ~float64
}

func Sum[T Number](xs []T) T {
	var total T
	for _, x := range xs {
		total += x
	}
	return total
}
