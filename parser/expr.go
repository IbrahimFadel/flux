package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

func (p *Parser) ParseExpr() (ast.Expr, error) {
	x, err := p.ParseBinaryExpr(1)
	if err != nil {
		return x, fmt.Errorf("could not parse expression: %s", err.Error())
	}
	return x, nil
}

func (p *Parser) ParseBinaryExpr(prec1 int) (ast.Expr, error) {
	x, err := p.ParseUnaryExpr()
	if err != nil {
		return x, fmt.Errorf("could not parse unary expression: %s", err.Error())
	}

	for {
		oprec := p.TokenPrecedence(p.CurTok)
		op := p.CurTok.TokenType
		opPos := p.CurTok.Pos

		if oprec < prec1 {
			return x, nil
		}
		p.EatToken()
		y, err := p.ParseBinaryExpr(oprec + 1)
		if err != nil {
			return x, fmt.Errorf("could not parse binary expression: %s", err.Error())
		}
		x = ast.BinaryExpr{X: x, OpPos: opPos, Op: op, Y: y}
		x, err = p.ParsePostfixExpr(x)
		if err != nil {
			return x, fmt.Errorf("could not parse postfix expression: %s", err.Error())
		}
	}
}

// TODO: do i need to do err checking here for more detailed errors? too tired rn to think
func (p *Parser) ParsePostfixExpr(x ast.Expr) (ast.Expr, error) {
	switch p.CurTok.TokenType {
	default:
		return x, nil
	case ast.TokenTypeOpenParen:
		return p.ParseFnCallExpr(x)
	}
}

func (p *Parser) ParseFnCallExpr(x ast.Expr) (ast.Expr, error) {
	// I don't think this is necessary, but just in case
	p.Expect(ast.TokenTypeOpenParen, "expected '(' in function call expression")
	openParenPos := p.CurTok.Pos
	p.EatToken()

	args, err := p.ParseCallArgs()
	if err != nil {
		return x, fmt.Errorf("could not parse function call arguments: %s", err.Error())
	}
	p.Expect(ast.TokenTypeCloseParen, "expected ')' at end of function call argument list") // I think this is redundant, but just in case
	closeParenPos := p.CurTok.Pos
	p.EatToken()

	return ast.CallExpr{Fn: x, Args: args, OpenParenPos: openParenPos, CloseParenPos: closeParenPos}, nil
}

/*
 * This function expects '(' to have already been consumed
 * and it does *not* consume ')'.
 */
func (p *Parser) ParseCallArgs() ([]ast.Expr, error) {
	var args []ast.Expr
	for p.CurTok.TokenType != ast.TokenTypeCloseParen {
		x, err := p.ParseExpr()
		if err != nil {
			return args, fmt.Errorf("could not parse expression: %s", err.Error())
		}
		args = append(args, x)
		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		} else if p.CurTok.TokenType != ast.TokenTypeCloseParen {
			return args, fmt.Errorf("expected ')' at end of function call argument list")
		}
	}

	return args, nil
}

func (p *Parser) ParseUnaryExpr() (ast.Expr, error) {
	switch p.CurTok.TokenType {
	default:
		return p.ParsePrimaryExpr()
	case ast.TokenTypeAmpersand, ast.TokenTypeAsterisk:
		// TODO: implement
		return ast.ExprStmt{}, nil
	}
}

func (p *Parser) ParsePrimaryExpr() (ast.Expr, error) {
	switch p.CurTok.TokenType {
	default:
		var x ast.Expr
		return x, fmt.Errorf(fmt.Sprintf("unknown expression: %s", p.CurTok.Value))
	case ast.TokenTypeNumberLiteral:
		return p.ParseNumberLit()
	case ast.TokenTypeStringLiteral:
		return p.ParseStringLit()
	case ast.TokenTypeIdentifier:
		return p.ParseIdentifier()
	}
}

func (p *Parser) ParseNumberLit() (ast.NumberLitExpr, error) {
	num := ast.NumberLitExpr{ValuePos: p.CurTok.Pos, Value: p.CurTok.Value}
	p.EatToken()
	return num, nil
}

func (p *Parser) ParseStringLit() (ast.StringLitExpr, error) {
	str := ast.StringLitExpr{ValuePos: p.CurTok.Pos, Value: p.CurTok.Value}
	p.EatToken()
	return str, nil
}

func (p *Parser) ParseIdentifier() (ast.Expr, error) {
	// if parser.Tokens[parser.TokIndex+1].TokenType == ast.TokenTypeOpenParen {
	// 	return parser.ParseFnCall()
	// }

	name := p.CurTok.Value
	pos := p.CurTok.Pos
	p.EatToken()
	return ast.VarRefExpr{Name: name, Pos: pos}, nil
}

// func (p *Parser) ParseExpr() (ast.Expr, error) {
// 	lhs, err := parser.ParseExprPrimary()
// 	if err != nil {
// 		var expr ast.Expr // TODO: is there a way to avoid this?? (inline empty struct for the return stmt)
// 		return expr, fmt.Errorf("could not parse expression primary: %s", err.Error())
// 	}

// 	expr, err := parser.ParseBinOpRHS(0, lhs)
// 	if err != nil {
// 		return expr, fmt.Errorf("could not parse rhs of binop expression: %s", err.Error())
// 	}

// 	return expr, nil
// }

// func (p *Parser) ParseExprPrimary() (ast.Expr, error) {

// 	switch parser.CurTok.TokenType {
// 	default:
// 		var expr ast.Expr
// 		return expr, fmt.Errorf(fmt.Sprintf("unknown expression: %s", parser.CurTok.Value))
// 	case ast.TokenTypeNumberLiteral:
// 		return parser.ParseNumberLit()
// 	case ast.TokenTypeStringLiteral:
// 		return parser.ParseStringLit()
// 	case ast.TokenTypeIdentifier:
// 		return parser.ParseIdentifier()
// 	}
// }

// func (p *Parser) ParseBinOpRHS(exprPrecedence int, lhs ast.Expr) (ast.Expr, error) {
// 	var binaryExpr ast.BinaryExpr
// 	for {
// 		tokPrecedence := parser.TokenPrcedence(parser.CurTok)
// 		if tokPrecedence < exprPrecedence {
// 			return lhs, nil
// 		}

// 		op := parser.CurTok.TokenType
// 		opPos := parser.CurTok.Pos
// 		parser.EatToken()

// 		rhs, err := parser.ParseExprPrimary()
// 		if err != nil {
// 			return lhs, fmt.Errorf("could not parse rhs expression: %s", err.Error())
// 		}

// 		nextPrecedence := parser.TokenPrcedence(parser.CurTok)
// 		if tokPrecedence <= nextPrecedence {
// 			rhs, err = parser.ParseBinOpRHS(tokPrecedence+1, rhs)
// 			if err != nil {
// 				return lhs, fmt.Errorf("could not parse rhs of binop expression") // This is confusingly similar to the previous err msg, consider rewording
// 			}
// 		}

// 		binaryExpr = ast.BinaryExpr{
// 			X:     lhs,
// 			Op:    op,
// 			OpPos: opPos,
// 			Y:     rhs,
// 		}
// 		return binaryExpr, nil
// 	}
// }

// func (p *Parser) ParseNumberLit() (ast.NumberLitExpr, error) {
// 	num := ast.NumberLitExpr{ValuePos: parser.CurTok.Pos, Value: parser.CurTok.Value}
// 	parser.EatToken()
// 	return num, nil
// }

// func (p *Parser) ParseStringLit() (ast.StringLitExpr, error) {
// 	str := ast.StringLitExpr{ValuePos: parser.CurTok.Pos, Value: parser.CurTok.Value}
// 	parser.EatToken()
// 	return str, nil
// }

// func (p *Parser) ParseIdentifier() (ast.Expr, error) {
// 	// if parser.Tokens[parser.TokIndex+1].TokenType == ast.TokenTypeOpenParen {
// 	// 	return parser.ParseFnCall()
// 	// }

// 	name := parser.CurTok.Value
// 	parser.EatToken()
// 	return ast.VarRefExpr{Name: name}, nil
// }

// func (p *Parser) ParseFnCall() (ast.CallExpr, error) {
// 	var fnCall ast.CallExpr

// 	// Coming from ParseIdentifier, we already know that the first two tokens are the correct types... but just incase at another point
// 	// We want to call ParseFnCall from somewhere else, let's do some err checking

// 	fnCall.Fn = parser.CurTok.Value
// 	parser.EatToken()
// 	parser.Expect(ast.TokenTypeOpenParen, "expected '(' in function call expression")
// 	fnCall.OpenParenPos = parser.CurTok.Pos
// 	parser.EatToken()

// 	for parser.CurTok.TokenType != ast.TokenTypeCloseParen {
// 		expr, err := parser.ParseExpr()
// 		if err != nil {
// 			return fnCall, fmt.Errorf("could not parse expression in function call argument list: %s", err.Error())
// 		}
// 		fnCall.Args = append(fnCall.Args, expr)

// 		if parser.CurTok.TokenType == ast.TokenTypeComma {
// 			parser.EatToken()
// 		} else if parser.CurTok.TokenType != ast.TokenTypeCloseParen {
// 			return fnCall, fmt.Errorf("expected ')' at end of function call argument list")
// 		}
// 	}

// 	parser.EatToken()

// 	return fnCall, nil
// }
