package main

import (
	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/codegen"
	"github.com/IbrahimFadel/pi-lang/parser"
	"github.com/IbrahimFadel/pi-lang/utils"
)

func main() {
	fileContent := utils.ReadFileContent("./testData/test-input.pi")

	var lexer ast.Lexer
	lexer.Tokenize(fileContent)

	// fmt.Println("---- Tokens ----")
	// for _, value := range lexer.Tokens {
	// 	str := utils.PrettyPrint(value)
	// 	fmt.Println(str)
	// }
	// fmt.Println("----------------")

	var parser parser.Parser
	parser.GenerateAST(lexer.Tokens)

	var ast string

	// fmt.Print("\n\n")

	// fmt.Println("----- AST -----")
	for _, value := range parser.Nodes {
		str := utils.PrettyPrint(value)
		ast += str + ",\n"
		// 	fmt.Println(str)
	}
	// fmt.Println("---------------")

	utils.WriteFile(ast, "ast.txt")

	var gen codegen.IRGenerator
	gen.GenerateIR(parser.Nodes)
}
