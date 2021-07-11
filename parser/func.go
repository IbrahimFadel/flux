package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (p *Parser) ParseFn() (ast.Node, error) {
	var fnDec ast.FuncDecl
	p.CurFunc = &fnDec

	p.EatToken()

	if p.CurTok.TokenType == ast.TokenTypeOpenParen {
		recv, err := p.ParseFuncReceiver()
		if err != nil {
			return fnDec, fmt.Errorf("could not parse function receiver: %s", err.Error())
		}
		fnDec.Receiver = recv
	}

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
	fnBody.Name = "entry"
	fnDec.Body = fnBody

	return fnDec, nil
}

func (p *Parser) ParseFuncReceiver() (ast.FuncReceiver, error) {
	p.Expect(ast.TokenTypeOpenParen, "expected '(' in function receiver")
	pos := p.CurTok.Pos
	p.EatToken()

	p.Expect(ast.TokenTypeIdentifier, "expected identifier in function receiver")
	name := p.CurTok.Value
	p.EatToken()

	ty, err := p.ParseType()
	if err != nil {
		return ast.FuncReceiver{}, fmt.Errorf("could not parse type: %s", err.Error())
	}

	p.Expect(ast.TokenTypeCloseParen, "expected ')' at end of function receiver")
	p.EatToken()

	return ast.FuncReceiver{Pos: pos, Name: name, Type: ty}, nil
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
		returnType, err := p.ParseType()
		if err != nil {
			return fnType, fmt.Errorf("could not parse return type: %s", err.Error())
		}
		fnType.Return = returnType
	} else {
		// If you don't specify return type, it's a void function
		fnType.Return = ast.PrimitiveTypeExpr{PrimitiveType: ast.TokenTypeVoid, Pos: p.CurTok.Pos}
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

	paramType, err := p.ParseType()
	if err != nil {
		return param, fmt.Errorf("could not parse parameter type: %s", err.Error())
	}
	param.Type = paramType

	p.Expect(ast.TokenTypeIdentifier, "expected identifier following parameter type")
	param.Name = p.CurTok.Value
	p.EatToken()

	return param, nil
}
