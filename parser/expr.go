package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (parser *Parser) ParseExpr() (ast.Expr, error) {
	lhs, err := parser.ParseExprPrimary()
	if err != nil {
		var expr ast.Expr // TODO: is there a way to avoid this?? (inline empty struct for the return stmt)
		return expr, fmt.Errorf("could not parse expression primary: %s", err.Error())
	}

	expr, err := parser.ParseBinOpRHS(0, lhs)
	if err != nil {
		return expr, fmt.Errorf("could not parse rhs of binop expression: %s", err.Error())
	}

	return expr, nil
}

func (parser *Parser) ParseExprPrimary() (ast.Expr, error) {

	switch parser.CurTok.TokenType {
	default:
		var expr ast.Expr
		return expr, fmt.Errorf(fmt.Sprintf("unknown expression: %s", parser.CurTok.Value))
	case ast.TokenTypeNumberLiteral:
		return parser.ParseNumberLit()
	case ast.TokenTypeStringLiteral:
		return parser.ParseStringLit()
	}
}

func (parser *Parser) ParseBinOpRHS(exprPrecedence int, lhs ast.Expr) (ast.Expr, error) {
	var binOpExpr ast.BinOpExpr
	for {
		tokPrecedence := parser.TokenPrcedence(parser.CurTok)
		if tokPrecedence < exprPrecedence {
			return lhs, nil
		}

		op := parser.CurTok.TokenType
		opPos := parser.CurTok.Pos
		parser.EatToken()

		rhs, err := parser.ParseExprPrimary()
		if err != nil {
			return lhs, fmt.Errorf("could not parse rhs expression: %s", err.Error())
		}

		nextPrecedence := parser.TokenPrcedence(parser.CurTok)
		if tokPrecedence < nextPrecedence {
			rhs, err = parser.ParseBinOpRHS(tokPrecedence+1, rhs)
			if err != nil {
				return lhs, fmt.Errorf("could not parse rhs of binop expression") // This is confusingly similar to the previous err msg, consider rewording
			}
		}

		binOpExpr = ast.BinOpExpr{
			X:     lhs,
			Op:    op,
			OpPos: opPos,
			Y:     rhs,
		}
		return binOpExpr, nil
	}
}

func (parser *Parser) ParseNumberLit() (ast.NumberLitExpr, error) {
	num := ast.NumberLitExpr{ValuePos: parser.CurTok.Pos, Value: parser.CurTok.Value}
	parser.EatToken()
	return num, nil
}

func (parser *Parser) ParseStringLit() (ast.StringLitExpr, error) {
	str := ast.StringLitExpr{ValuePos: parser.CurTok.Pos, Value: parser.CurTok.Value}
	parser.EatToken()
	return str, nil
}
