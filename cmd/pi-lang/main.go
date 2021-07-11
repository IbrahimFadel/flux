package main

import (
	"flag"
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/codegen"
	"github.com/IbrahimFadel/pi-lang/parser"
	"github.com/IbrahimFadel/pi-lang/utils"
)

func main() {
	emitTokens := flag.Bool("emit-tokens", false, "print lexed tokens")
	emitAst := flag.Bool("emit-ast", false, "print AST and write it to file")
	emitIR := flag.Bool("emit-ir", false, "print IR and write it to file")
	flag.Parse()

	fileContent := utils.ReadFileContent("./testData/test-input.pi")

	var lexer ast.Lexer
	lexer.Tokenize(fileContent)

	if *emitTokens {
		fmt.Println("---- Tokens ----")
		for _, value := range lexer.Tokens {
			str := utils.PrettyPrint(value)
			fmt.Println(str)
		}
		fmt.Println("----------------")
	}

	var parser parser.Parser
	parser.GenerateAST(lexer.Tokens)

	if *emitAst {
		var ast string
		fmt.Print("\n\n")

		fmt.Println("----- AST -----")
		for _, value := range parser.Nodes {
			str := utils.PrettyPrint(value)
			ast += str + ",\n"
			fmt.Println(str)
		}
		fmt.Println("---------------")

		utils.WriteFile(ast, "ast.txt")
	}

	var gen codegen.IRGenerator
	gen.GenerateIR(parser.Nodes)

	if *emitIR {
		fmt.Print("\n\n")
		fmt.Println("----- IR -----")
		fmt.Println(gen.Module)
		utils.WriteFile(gen.Module.String(), "module.ll")
		fmt.Println("---------------")
	}
}
