package main

import "fmt"

func sum(x int, y int) int {
	return x + y
}

func main() {
	fmt.Println("Hello world!", sum(2, 3))
}
