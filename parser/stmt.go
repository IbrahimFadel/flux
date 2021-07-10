package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (parser *Parser) ParseBlockStmt() (ast.BlockStmt, error) {
	var block ast.BlockStmt

	parser.Expect(ast.TokenTypeOpenCurlyBracket, "expected '{' at beggining of block statement")
	parser.EatToken()

	for parser.CurTok.TokenType != ast.TokenTypeCloseCurlyBracket {
		stmt, err := parser.ParseStatement()
		if err != nil {
			return block, fmt.Errorf("could not parse statement: %s", err.Error())
		}
		block.List = append(block.List, stmt)
	}

	parser.EatToken()

	return block, nil
}

func (parser *Parser) ParseStatement() (ast.Stmt, error) {
	var stmt ast.Stmt

	switch parser.CurTok.TokenType {
	default:
		return stmt, fmt.Errorf("no method for parsing statement: %s", parser.CurTok.Value)
	case ast.TokenTypeReturn:
		return parser.ParseReturn()
	case ast.TokenTypeMut:
		return parser.ParseMut()
	case ast.TokenTypeConst:
		return parser.ParseConst()
	}
}

func (parser *Parser) ParseReturn() (ast.ReturnStmt, error) {
	var ret ast.ReturnStmt

	parser.EatToken()

	retVal, err := parser.ParseExpr()
	if err != nil {
		return ret, fmt.Errorf("could not parse expression: %s", err.Error())
	}
	ret.Value = retVal

	return ret, nil
}
