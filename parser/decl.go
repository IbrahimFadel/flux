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

func (p *Parser) ParseVarDecl() (ast.Decl, error) {
	var varDecl ast.VarDecl

	if p.CurTok.TokenType == ast.TokenTypeMut {
		varDecl.Mut = true
		p.EatToken()
	} else if p.CurTok.TokenType == ast.TokenTypeConst {
		varDecl.Mut = false
		p.EatToken()
	} else {
		return varDecl, fmt.Errorf("expected 'mut' or 'const' in variable declaration")
	}

	ty, err := p.ParseType()
	if err != nil {
		return varDecl, fmt.Errorf("could not parse mutable type: %s", err.Error())
	}
	varDecl.Type = ty
	varDecl.Names = p.ParseIdentList()

	if p.CurTok.TokenType != ast.TokenTypeEq {
		// if there's no '=' assign each of them to null
		varDecl.Values = make([]ast.Expr, len(varDecl.Names))
		for i := range varDecl.Values {
			varDecl.Values[i] = ast.NullExpr{Pos: p.CurTok.Pos}
		}
		return varDecl, nil
	}

	p.Expect(ast.TokenTypeEq, "expected '=' following mutable declaration list") // bad msg, + idek if this check is necessary, come back to this
	p.EatToken()

	for i := 0; i < len(varDecl.Names); i++ {
		expr, err := p.ParseExpr()
		if err != nil {
			return varDecl, fmt.Errorf("could not parse expression following mutable declaration list")
		}
		varDecl.Values = append(varDecl.Values, expr)
		if p.CurTok.TokenType == ast.TokenTypeComma {
			p.EatToken()
		}
	}

	return varDecl, nil
}

func (p *Parser) ParseTypeDecl() (ast.TypeDecl, error) {
	var typeDecl ast.TypeDecl

	p.EatToken()

	p.Expect(ast.TokenTypeIdentifier, "expected identifier following type declaration")
	typeDecl.Name = p.CurTok.Value
	p.EatToken()

	typeValue, err := p.ParseType()
	if err != nil {
		return typeDecl, fmt.Errorf("could not parse type in type declaration: %s", err.Error()) // needs better msg
	}
	typeDecl.Type = typeValue

	p.KnownIdentifierTypes[typeDecl.Name] = &typeDecl

	return typeDecl, nil
}
