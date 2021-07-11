package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (p *Parser) ParseFn() (ast.Node, error) {
	var fnDec ast.FuncDecl

	p.EatToken()

	p.Expect(ast.TokenTypeIdentifier, "expected identifier following 'fn'")
	fnDec.Name = p.CurTok.Value
	p.EatToken()

	fnType, err := p.ParseFuncType()
	if err != nil {
		return nil, fmt.Errorf("could not parse function type: %s", err.Error())
	}
	fnDec.FuncType = fnType

	fnBody, err := p.ParseBlockStmt()
	if err != nil {
		return nil, fmt.Errorf("could not parse function body: %s", err.Error())
	}
	fnDec.Body = fnBody

	return fnDec, nil
}

func (p *Parser) ParseFuncType() (ast.FuncType, error) {
	var fnType ast.FuncType

	p.Expect(ast.TokenTypeOpenParen, "expected '(' following function name")
	pos := p.CurTok.Pos
	fnType.FuncPos = pos
	p.EatToken()

	params, err := p.ParseParamList()
	if err != nil {
		return fnType, fmt.Errorf("could not parse param list: %s", err.Error())
	}
	fnType.Params = params

	if p.CurTok.TokenType == ast.TokenTypeArrow {
		p.EatToken()
		returnType := p.ParseType()
		fnType.Return = returnType
	} else {
		// If you don't specify return type, it's a void function
		fnType.Return = "void"
	}

	return fnType, nil
}

func (p *Parser) ParseParamList() (ast.ParamList, error) {
	var paramList ast.ParamList

	for p.CurTok.TokenType != ast.TokenTypeCloseParen {
		param, err := p.ParseParam()
		if err != nil {
			return paramList, fmt.Errorf("could not parse param: %s", err.Error())
		}
		paramList.Params = append(paramList.Params, param)

		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		} else if p.CurTok.TokenType != ast.TokenTypeCloseParen {
			return paramList, fmt.Errorf("expected ')' at end of parameter list")
		}
	}

	p.EatToken()

	return paramList, nil
}

func (p *Parser) ParseParam() (ast.Param, error) {
	var param ast.Param

	param.Mut = false
	if p.CurTok.TokenType == ast.TokenTypeMut {
		param.Mut = true
		p.EatToken()
	}

	paramType := p.ParseType()
	param.Type = paramType

	p.Expect(ast.TokenTypeIdentifier, "expected identifier following parameter type")
	param.Name = p.CurTok.Value
	p.EatToken()

	return param, nil
}
