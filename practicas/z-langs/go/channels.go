package main

import (
	"fmt"
	"time"
)

func main() {

	var channel = make(chan string)
	go consumer(channel)
	fmt.Println("Antes de enviar")
	channel <- "Hola"
	fmt.Println("Despues de enviar")

}

func consumer(channel chan string) {
	fmt.Println("Estoy durmiendo")
	time.Sleep(2 * time.Second)
	fmt.Println("Me desperté")
	var v = <-channel
	fmt.Println("Recibi ", v)
}