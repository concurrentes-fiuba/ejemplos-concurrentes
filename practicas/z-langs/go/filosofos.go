package main

import (
	"fmt"
	"math/rand"
	"runtime"
	"time"
)

const FILOSOFOS = 5

type Filosofo struct {
	id int
	izq chan bool
	der chan bool
}

var mesa [FILOSOFOS] Filosofo

func main() {

	for id := 1; id < FILOSOFOS; id++ {
		var izq chan bool
		if id > 0 {
			izq = mesa[id - 1].der
		}
		mesa[id] = Filosofo{ id: id, izq: izq, der: make(chan bool)}
	}
	mesa[0].izq = mesa[FILOSOFOS - 1].der

	for _, filosofo := range mesa {
		go filosofo.Run()
	}

	for {
		runtime.Gosched()
	}
}

func (f Filosofo) Run() {

	var palitoIzq = f.id == 0
	var palitoDer = f.id != (FILOSOFOS - 1)

	for {
		var hambriento = false
		fmt.Println("[", f.id, "] durmiendo")
		comer := time.After(time.Duration(rand.Int31n(2000)) * time.Millisecond)

		for !hambriento {
			select {
			case <-f.izq:
				if palitoIzq {
					fmt.Println("[", f.id, "] entrego palito izq")
					f.izq <- true
					palitoIzq = false
				}
			case <-f.der:
				if palitoDer {
					fmt.Println("[", f.id, "] entrego palito derecho")
					f.der <- true
					palitoDer = false
				}
			case <-comer:
				hambriento = true
			}
		}
		fmt.Println("[", f.id, "] hambriento")

		if !palitoIzq {
			fmt.Println("[", f.id, "] pido palito izq")
			f.izq <- false
			palitoIzq = <-f.izq
		}

		if !palitoDer {
			fmt.Println("[", f.id, "] pido palito derecho")
			f.der <- false
			palitoDer = <-f.der
		}

		fmt.Println("[", f.id, "] comiendo")
		time.Sleep(time.Duration(rand.Int31n(2000)) * time.Millisecond)

	}

}
