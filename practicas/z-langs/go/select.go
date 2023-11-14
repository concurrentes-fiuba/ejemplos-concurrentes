package main

import (
	"fmt"
	"math/rand"
	"time"
)

func main() {

	producer1 := producer()
	producer2 := producer()

	for {
		select {
		case v := <-producer1:
			fmt.Println("p1", v)
		case v := <-producer2:
			fmt.Println("p2", v)
		case <-time.After(500 * time.Millisecond):
			fmt.Println("me aburro")
		}
	}

}

func producer() chan int32 {
	channel := make(chan int32)
	go func() {
		for {
			v := rand.Int31n(1000)
			time.Sleep(time.Duration(v) * time.Millisecond)
			channel <- v
		}
	}()
	return channel
}