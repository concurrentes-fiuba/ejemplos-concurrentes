package main

import (
	"fmt"
	"time"
)

func sleep_print(seconds time.Duration, message string) {
	fmt.Println("antes " + message)
	time.Sleep(seconds * time.Second)
	fmt.Println("despues " + message)
}

func main() {
	fmt.Println("primero")
	sleep_print(1, "secuencial")
	go sleep_print(2, "concurrente 1")
	go sleep_print(1, "concurrente 2")
	time.Sleep(3 * time.Second)
}
