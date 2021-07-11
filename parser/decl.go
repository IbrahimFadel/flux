package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
)

/*
 * Parse the 'x, y, z' in 'mut i32 x, y, z = 10, 2, 5'
 *
 * @return idents []string ["x", "y", "z"]
 */
func (p *Parser) ParseIdentList() []string {
	var idents []string

	for p.CurTok.TokenType != ast.TokenTypeComma && p.CurTok.TokenType != ast.TokenTypeEq {
		p.Expect(ast.TokenTypeIdentifier, "expected identifier following mutable declaration type")
		idents = append(idents, p.CurTok.Value)
		p.EatToken()

		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		} else {
			return idents
		}
	}

	return idents // i don't think this will ever be reached
}

// TODO: Look over ParseMut and ParseConst as they're basically the exact same thing
func (p *Parser) ParseMut() (ast.MutDecl, error) {
	var mut ast.MutDecl

	p.EatToken()
	mut.MutType = p.ParseType()
	mut.Names = p.ParseIdentList()

	if p.CurTok.TokenType != ast.TokenTypeEq {
		// if there's no '=' assign each of them to null
		mut.Values = make([]ast.Expr, len(mut.Names))
		for i := range mut.Values {
			mut.Values[i] = ast.NullExpr{Pos: p.CurTok.Pos}
		}
		return mut, nil
	}

	p.Expect(ast.TokenTypeEq, "expected '=' following mutable declaration list") // bad msg, + idek if this check is necessary, come back to this
	p.EatToken()

	for i := 0; i < len(mut.Names); i++ {
		expr, err := p.ParseExpr()
		if err != nil {
			return mut, fmt.Errorf("could not parse expression following mutable declaration list")
		}
		mut.Values = append(mut.Values, expr)
		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		}
	}

	return mut, nil
}

func (p *Parser) ParseConst() (ast.ConstDecl, error) {
	var constDecl ast.ConstDecl

	p.EatToken()
	constDecl.ConstType = p.ParseType()
	constDecl.Names = p.ParseIdentList()

	if p.CurTok.TokenType != ast.TokenTypeEq {
		// if there's no '=' assign each of them to null
		constDecl.Values = make([]ast.Expr, len(constDecl.Names))
		for i := range constDecl.Values {
			constDecl.Values[i] = ast.NullExpr{Pos: p.CurTok.Pos}
		}
		return constDecl, nil
	}

	p.Expect(ast.TokenTypeEq, "expected '=' following constant declaration list") // bad msg, + idek if this check is necessary, come back to this
	p.EatToken()

	for i := 0; i < len(constDecl.Names); i++ {
		expr, err := p.ParseExpr()
		if err != nil {
			return constDecl, fmt.Errorf("could not parse expression following mutable declaration list")
		}
		constDecl.Values = append(constDecl.Values, expr)
		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		}
	}

	return constDecl, nil
}

// TODO: implement... i wanna go work on codegen :)
func (p *Parser) ParseTypeDecl() (ast.TypeDecl, error) {
	var typeDecl ast.TypeDecl

	p.EatToken()

	p.Expect(ast.TokenTypeIdentifier, "expected identifier following type declaration")
	// name := p.CurTok.Value
	p.EatToken()

	// typeValue := p.ParseType()

	return typeDecl, nil
}
