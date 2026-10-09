package main

import "sync"

// Counter is a concurrency-safe counter.
type Counter struct {
	mu sync.Mutex
	n  map[string]int
}

func NewCounter() *Counter {
	return &Counter{n: make(map[string]int)}
}

func (c *Counter) Inc(key string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.n[key]++
}

func (c *Counter) Get(key string) int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.n[key]
}

// FanOut launches workers and waits for all of them.
func FanOut(workers int, done chan<- struct{}) {
	var wg sync.WaitGroup
	ch := make(chan int, workers)
	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			ch <- id
		}(i)
	}
	go func() {
		wg.Wait()
		close(ch)
		done <- struct{}{}
	}()
}
