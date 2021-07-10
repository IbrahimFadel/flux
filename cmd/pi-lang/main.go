package main

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/parser"
	"github.com/IbrahimFadel/pi-lang/utils"
)

func main() {
	fileContent := utils.ReadFileContent("./testData/test-input.pi")

	var lexer ast.Lexer
	lexer.Tokenize(fileContent)

	fmt.Println("---- Tokens ----")
	for _, value := range lexer.Tokens {
		fmt.Printf("- %v\n", value)
	}
	fmt.Println("----------------")

	var parser parser.Parser
	parser.GenerateAST(lexer.Tokens)

	fmt.Print("\n\n")

	fmt.Println("----- AST -----")
	for _, value := range parser.Nodes {
		fmt.Printf("- %v\n", value)
	}
	fmt.Println("---------------")
}
