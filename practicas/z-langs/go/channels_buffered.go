package main

import (
	"fmt"
	"time"
)

func main() {

	var channel = make(chan int, 10)

	go func() {
		time.Sleep(2 * time.Second)
		fmt.Println("ME DESPERTE")
		var v = <-channel
		fmt.Println("Recibi ", v)
		time.Sleep(1 * time.Second)
		fmt.Println("no esperó!")
	}()

	fmt.Println("Antes de enviar")
	for i := 0; i < 10; i++ {
		channel <- i
	}
	fmt.Println("Despues de enviar")
	channel <- 11
	fmt.Println("Terminé")

}