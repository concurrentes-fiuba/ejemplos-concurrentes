package main

import (
	"fmt"
)

func main() {
	var channel = make(chan int)
	var done = make(chan bool)
	go infiniteConsumer(channel, done)
	for i:= 0; i< 10; i++ {
		channel <- i
	}
	fmt.Println("Cierro el channel")
 	close(channel)
	fmt.Println("Espero terminar")
	<- done
}

func infiniteConsumer(channel chan int, done chan bool) {
	for v := range channel {
		fmt.Println("Recibi ", v)
	}
	fmt.Println("Salí")
	done <- true
}
