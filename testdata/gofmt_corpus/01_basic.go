package main

import (
	"fmt"
	"os"
)

// Greeter produces greeting messages.
type Greeter struct {
	Name string
	Age  int
}

func (g *Greeter) Greet() string {
	if g.Age > 100 {
		return fmt.Sprintf("Wow %s!", g.Name)
	}
	return "Hello, " + g.Name
}

func main() {
	g := &Greeter{Name: "world", Age: 30}
	fmt.Println(g.Greet())
	os.Exit(0)
}
