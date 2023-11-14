package main

import "time"

var a, b int

func f() {
	a = 1
	b = 2 + a


}

func g() {
	print(b)
	print(a)
}

func main() {
	go g()
	go f()
	time.Sleep(time.Second)
}