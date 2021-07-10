package utils

import (
	"log"
	"os"
)

const (
	RED   = "\033[31m"
	RESET = "\033[0m"
)

func Error(msg string) {
	log.Printf(RED + "error: " + msg + RESET)
}

func FatalError(msg string) {
	Error(msg)
	os.Exit(1)
}
