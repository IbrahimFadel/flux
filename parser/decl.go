package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

// TODO: Assign actually *can* be done on ConstDecl, initial declaration... figure this out

func (parser *Parser) ParseMut() (ast.AssignStmt, error) {
	var assign ast.AssignStmt

	parser.EatToken()
	mutTypes := parser.ParseType()

	for parser.CurTok.TokenType != ast.TokenTypeComma && parser.CurTok.TokenType != ast.TokenTypeEq {
		parser.Expect(ast.TokenTypeIdentifier, "expected identifier following mutable declaration type")
		mut := ast.MutDecl{
			Type: mutTypes,
			Name: parser.CurTok.Value,
		}
		assign.Left = append(assign.Left, mut)
		parser.EatToken()
		if parser.CurTok.TokenType == ast.TokenTypeComma {
			parser.EatToken()
		} else if parser.CurTok.TokenType != ast.TokenTypeEq {
			// If there's no '=', assign them all to null
			nullExpr := ast.NullExpr{Pos: parser.CurTok.Pos}
			assign.Right = make([]ast.Expr, len(assign.Left))
			for i := range assign.Right {
				assign.Right[i] = nullExpr
			}
			return assign, nil
		}
	}
	parser.Expect(ast.TokenTypeEq, "expected '=' following mutable declaration list") // bad msg, + idek if this check is necessary, come back to this
	parser.EatToken()

	for i := 0; i < len(assign.Left); i++ {
		expr, err := parser.ParseExpr()
		if err != nil {
			return assign, fmt.Errorf("could not parse expression following mutable declaration list")
		}
		assign.Right = append(assign.Right, expr)
		if parser.CurTok.TokenType == ast.TokenTypeComma {
			parser.EatToken()
		}
	}

	return assign, nil
}

// func (parser *Parser) ParseConst() (ast.AssignStmt, error) {
// 	var assign ast.AssignStmt

// 	parser.EatToken()
// 	mutTypes := parser.ParseType()

// 	for parser.CurTok.TokenType != ast.TokenTypeComma && parser.CurTok.TokenType != ast.TokenTypeEq {
// 		parser.Expect(ast.TokenTypeIdentifier, "expected identifier following mutable declaration type")
// 		mut := ast.MutDecl{
// 			Type: mutTypes,
// 			Name: parser.CurTok.Value,
// 		}
// 		assign.Left = append(assign.Left, mut)
// 		parser.EatToken()
// 		if parser.CurTok.TokenType == ast.TokenTypeComma {
// 			parser.EatToken()
// 		} else if parser.CurTok.TokenType != ast.TokenTypeEq {
// 			// If there's no '=', assign them all to null
// 			nullExpr := ast.NullExpr{Pos: parser.CurTok.Pos}
// 			assign.Right = make([]ast.Expr, len(assign.Left))
// 			for i := range assign.Right {
// 				assign.Right[i] = nullExpr
// 			}
// 			return assign, nil
// 		}
// 	}
// 	parser.Expect(ast.TokenTypeEq, "expected '=' following mutable declaration list") // bad msg, + idek if this check is necessary, come back to this
// 	parser.EatToken()

// 	for i := 0; i < len(assign.Left); i++ {
// 		expr, err := parser.ParseExpr()
// 		if err != nil {
// 			return assign, fmt.Errorf("could not parse expression following mutable declaration list")
// 		}
// 		assign.Right = append(assign.Right, expr)
// 		if parser.CurTok.TokenType == ast.TokenTypeComma {
// 			parser.EatToken()
// 		}
// 	}

// 	return assign, nil
// }
