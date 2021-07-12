package main

import "fmt"

type Animal interface {
	Hello()
}

type Human interface {
	Hello()
}

type Dog struct {
}

func (d Dog) Hello() {

}

func main() {
	var animal Animal
	var human Human

	dog := new(Dog)

	animal = *dog
	human = *dog

	fmt.Printf("animal: %v\n", animal)
	fmt.Printf("human: %v\n", human)
}
