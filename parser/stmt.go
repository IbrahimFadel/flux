package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (p *Parser) ParseBlockStmt() (ast.BlockStmt, error) {
	var block ast.BlockStmt

	p.Expect(ast.TokenTypeOpenCurlyBracket, "expected '{' at beggining of block statement")
	startPos := p.CurTok.Pos
	p.EatToken()

	for p.CurTok.TokenType != ast.TokenTypeCloseCurlyBracket {
		stmt, err := p.ParseStatement()
		if err != nil {
			return block, fmt.Errorf("could not parse statement: %s", err.Error())
		}
		block.List = append(block.List, stmt)
	}

	endPos := p.CurTok.Pos
	p.EatToken()

	block.Start = startPos
	block.End = endPos

	return block, nil
}

func (p *Parser) ParseStatement() (ast.Stmt, error) {
	var stmt ast.Stmt

	switch p.CurTok.TokenType {
	default:
		return stmt, fmt.Errorf("no method for parsing statement: %s", p.CurTok.Value)
	case ast.TokenTypeReturn:
		return p.ParseReturn()
	case ast.TokenTypeMut, ast.TokenTypeConst:
		return p.ParseVarDecl()
	case ast.TokenTypeIdentifier:
		return p.ParseExpr()
	}
}

func (p *Parser) ParseReturn() (ast.ReturnStmt, error) {
	var ret ast.ReturnStmt

	retPos := p.CurTok.Pos

	p.EatToken()

	ret.Type = p.CurFunc.FuncType.Return
	p.CurType = ret.Type
	retVal, err := p.ParseExpr()
	if err != nil {
		return ret, fmt.Errorf("could not parse expression: %s", err.Error())
	}
	ret.Value = retVal
	ret.ReturnPos = retPos

	return ret, nil
}
