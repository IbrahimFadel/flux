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

func (parser *Parser) Init(tokens []ast.Token) {
	parser.Tokens = tokens
	parser.CurTok = parser.Tokens[0]
	parser.TokIndex = 0
	parser.BinopPrecedence = map[string]int{
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

func (parser *Parser) GenerateAST(tokens []ast.Token) {
	parser.Init(tokens)

	for parser.CurTok.TokenType != ast.TokenTypeEOF {
		node, err := parser.ParseToken(parser.CurTok)
		if err != nil {
			utils.FatalError(err.Error())
		}
		parser.Nodes = append(parser.Nodes, node)
	}
}

func (parser *Parser) EatToken() {
	parser.TokIndex++
	parser.CurTok = parser.Tokens[parser.TokIndex]
}

/*
 * Expect the CurTok to be of type
 * If not, throw FatalError
 */
func (parser *Parser) Expect(tokType ast.TokenType, msg string) {
	if parser.CurTok.TokenType != tokType {
		utils.FatalError(fmt.Sprintf(msg+" at pos: %v", parser.CurTok.Pos))
	}
}

func (parser *Parser) TokenPrcedence(tok ast.Token) int {
	precedence := parser.BinopPrecedence[tok.Value]
	if precedence <= 0 {
		return -1
	}
	return precedence
}

func (parser *Parser) ParseToken(token ast.Token) (ast.Node, error) {
	var node ast.Node

	switch token.TokenType {
	default:
		return node, fmt.Errorf("could not parse token '%s'", parser.CurTok.Value)
	case ast.TokenTypeFn:
		return parser.ParseFn()
	}
}

func (parser *Parser) ParseType() string {
	// While there are only certain things that make a valid type, we won't check here because it would be tedious (types can be somewhat complex)
	// Error checking for valid types will be done later when the type(string) is converted to type(llvm)
	// Maybe i'll revisit down the line if I think it could be done better

	_type := parser.CurTok.Value
	parser.EatToken()
	for parser.CurTok.TokenType == ast.TokenTypeAsterisk || parser.CurTok.TokenType == ast.TokenTypeAmpersand {
		_type += parser.CurTok.Value
		parser.EatToken()
	}

	return _type
}
