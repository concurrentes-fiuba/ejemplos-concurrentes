package main

import (
	"fmt"
	"math/rand"
	"time"
)

const INVERSORES = 5

func main() {
	var plata = 10000.0
	var inversores [INVERSORES]chan float64

	for id := 0; id < INVERSORES; id++ {
		inversores[id] = inversor(id)
	}

	for plata > 5.0 {
		var prestamo = plata / INVERSORES
		for _, inversor := range inversores {
			inversor <- prestamo
		}

		var plataSemana = 0.0
		for _, inversor := range inversores {
			plataSemana += <-inversor
		}
		fmt.Println("[Banquero] final de semana", plataSemana)
		plata = plataSemana
	}
}

func inversor(id int) chan float64 {
	var channel = make(chan float64)
	go func() {
		for plataInicial := range channel {
			fmt.Println("[Inversor ", id, "] me dan ", plataInicial)
			time.Sleep(2 * time.Second)
			var resultado = plataInicial * (0.5 + rand.Float64())
			fmt.Println("[Inversor ", id, "] devuelvo ", resultado)
			channel <- resultado
		}
	}()
	return channel
}
