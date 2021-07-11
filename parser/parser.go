package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/utils"
)

type Parser struct {
	Nodes           []ast.Node
	Tokens          []ast.Token
	CurTok          ast.Token
	TokIndex        int
	BinopPrecedence map[string]int
}

func (p *Parser) Init(tokens []ast.Token) {
	p.Tokens = tokens
	p.CurTok = p.Tokens[0]
	p.TokIndex = 0
	p.BinopPrecedence = map[string]int{
		"=":  2,
		"||": 5,
		"&&": 3,
		"<":  10,
		">":  10,
		"<=": 10,
		">=": 10,
		"==": 10,
		"!=": 10,
		"+":  20,
		"-":  20,
		"*":  40,
		"/":  40,
		".":  50,
		"->": 50,
	}
}

func (p *Parser) GenerateAST(tokens []ast.Token) {
	p.Init(tokens)

	for p.CurTok.TokenType != ast.TokenTypeEOF {
		node, err := p.ParseToken(p.CurTok)
		if err != nil {
			utils.FatalError(err.Error())
		}
		p.Nodes = append(p.Nodes, node)
	}
}

func (p *Parser) EatToken() {
	p.TokIndex++
	p.CurTok = p.Tokens[p.TokIndex]
}

/*
 * Expect the CurTok to be of type
 * If not, throw FatalError
 */
func (p *Parser) Expect(tokType ast.TokenType, msg string) {
	if p.CurTok.TokenType != tokType {
		utils.FatalError(fmt.Sprintf(msg+" at pos: %v", p.CurTok.Pos))
	}
}

func (p *Parser) TokenPrecedence(tok ast.Token) int {
	precedence := p.BinopPrecedence[tok.Value]
	if precedence <= 0 {
		return -1
	}
	return precedence
}

func (p *Parser) ParseToken(token ast.Token) (ast.Node, error) {
	var node ast.Node

	switch token.TokenType {
	default:
		return node, fmt.Errorf("could not parse token '%s'", p.CurTok.Value)
	case ast.TokenTypeFn:
		return p.ParseFn()
	case ast.TokenTypePackage:
		return p.ParsePackageClause()
	case ast.TokenTypeType:
		return p.ParseTypeDecl()
	case ast.TokenTypeIdentifier:
		return p.ParseExpr()
	}
}

func (p *Parser) ParseType() string {
	// While there are only certain things that make a valid type, we won't check here because it would be tedious (types can be somewhat complex)
	// Error checking for valid types will be done later when the type(string) is converted to type(llvm)
	// Maybe i'll revisit down the line if I think it could be done better

	_type := p.CurTok.Value
	p.EatToken()
	for p.CurTok.TokenType == ast.TokenTypeAsterisk || p.CurTok.TokenType == ast.TokenTypeAmpersand {
		_type += p.CurTok.Value
		p.EatToken()
	}

	return _type
}

func (p *Parser) ParsePackageClause() (ast.PackageClause, error) {
	p.EatToken()
	p.Expect(ast.TokenTypeIdentifier, "expected identifier following 'package'")
	name := p.CurTok.Value
	p.EatToken()
	return ast.PackageClause{Name: name}, nil
}
