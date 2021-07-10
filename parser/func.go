package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (parser *Parser) ParseFn() (ast.Node, error) {
	var fnDec ast.FuncDec

	parser.EatToken()

	parser.Expect(ast.TokenTypeIdentifier, "expected identifier following 'fn'")
	fnDec.Name = parser.CurTok.Value
	parser.EatToken()

	fnType, err := parser.ParseFuncType()
	if err != nil {
		return nil, fmt.Errorf("could not parse function type: %s", err.Error())
	}
	fnDec.Type = fnType

	fnBody, err := parser.ParseBlockStmt()
	if err != nil {
		return nil, fmt.Errorf("could not parse function body: %s", err.Error())
	}
	fnDec.Body = fnBody

	return fnDec, nil
}

func (parser *Parser) ParseFuncType() (ast.FuncType, error) {
	var fnType ast.FuncType

	parser.Expect(ast.TokenTypeOpenParen, "expected '(' following function name")
	pos := parser.CurTok.Pos
	fnType.FuncPos = pos
	parser.EatToken()

	params, err := parser.ParseParamList()
	if err != nil {
		return fnType, fmt.Errorf("could not parse param list: %s", err.Error())
	}
	fnType.Params = params

	if parser.CurTok.TokenType == ast.TokenTypeArrow {
		parser.EatToken()
		returnType := parser.ParseType()
		fnType.Return = returnType
	} else {
		// If you don't specify return type, it's a void function
		fnType.Return = "void"
	}

	return fnType, nil
}

func (parser *Parser) ParseParamList() (ast.ParamList, error) {
	var paramList ast.ParamList

	for parser.CurTok.TokenType != ast.TokenTypeCloseParen {
		param, err := parser.ParseParam()
		if err != nil {
			return paramList, fmt.Errorf("could not parse param: %s", err.Error())
		}
		paramList.Params = append(paramList.Params, param)

		if parser.CurTok.TokenType == ast.TokenTypeComma {
			parser.EatToken()
		} else if parser.CurTok.TokenType != ast.TokenTypeCloseParen {
			return paramList, fmt.Errorf("expected ')' at end of parameter list")
		}
	}

	parser.EatToken()

	return paramList, nil
}

func (parser *Parser) ParseParam() (ast.Param, error) {
	var param ast.Param

	param.Mut = false
	if parser.CurTok.TokenType == ast.TokenTypeMut {
		param.Mut = true
		parser.EatToken()
	}

	paramType := parser.ParseType()
	param.Type = paramType

	parser.Expect(ast.TokenTypeIdentifier, "expected identifier following parameter type")
	param.Name = parser.CurTok.Value
	parser.EatToken()

	return param, nil
}
