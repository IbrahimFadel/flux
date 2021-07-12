package utils

import (
	"bufio"
	"bytes"
	"io/ioutil"
	"os"
	"strconv"

	"github.com/llir/llvm/ir/types"
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

func WriteFile(content string, path string) {
	err := ioutil.WriteFile(path, []byte(content), 0644)
	if err != nil {
		FatalError(err.Error())
	}
}

func ContainsByte(arr []byte, el byte) bool {
	return bytes.IndexByte(arr, el) != -1
}

func IsNumber(str string) bool {
	_, err := strconv.ParseFloat(str, 64)
	return err == nil
}

func TypeArraysEqual(a, b []types.Type) bool {
	if len(a) != len(b) {
		return false
	}
	for i, v := range a {
		if !v.Equal(b[i]) {
			return false
		}
	}
	return true
}

func TypeArrayContains(s []types.Type, e types.Type) bool {
	for _, a := range s {
		if a.Equal(e) {
			return true
		}
	}
	return false
}
