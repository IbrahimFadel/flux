package utils

import (
	"bufio"
	"os"
	"strconv"
)

func ReadFileContent(filePath string) []string {
	var content []string

	file, err := os.Open(filePath)
	if err != nil {
		FatalError(err.Error())
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	// optionally, resize scanner's capacity for lines over 64K, see next example
	for scanner.Scan() {
		content = append(content, scanner.Text()+"\n")
	}

	if err := scanner.Err(); err != nil {
		FatalError(err.Error())
	}

	return content
}

func ContainsByte(arr []byte, el byte) bool {
	for _, a := range arr {
		if a == el {
			return true
		}
	}
	return false
}

func IsNumber(str string) bool {
	_, err := strconv.ParseFloat(str, 64)
	return err == nil
}
