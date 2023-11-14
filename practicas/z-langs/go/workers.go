package main

import (
	"fmt"
	"math/rand"
	"time"
)

const WORKERS = 10

func main() {

	var channel = make(chan int)

	for id := 0; id < WORKERS; id++ {
		go worker(id, channel)
	}

	for i := 0; i < 200; i++ {
		channel <- i
	}

	time.Sleep(1000 * time.Second)
}

func worker(id int, channel chan int) {
	for v := range channel {
		fmt.Println("worker ", id, " value ", v)
		time.Sleep(time.Duration(rand.Int31n(2000)) * time.Millisecond)
	}
}
