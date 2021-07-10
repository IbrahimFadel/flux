package ast_test

import (
	"testing"

	"github.com/IbrahimFadel/pi-lang/ast"
)

var lexer ast.Lexer

var (
	InputArithmeticNoSpaces  = []string{"x+y*5.0/-1 - 9\n"}
	OutputArithmeticNoSpaces = [...]ast.TokenType{ast.TokenTypeIdentifier, ast.TokenTypePlus, ast.TokenTypeIdentifier, ast.TokenTypeAsterisk, ast.TokenTypeNumberLiteral, ast.TokenTypeSlash, ast.TokenTypeNumberLiteral, ast.TokenTypeMinus, ast.TokenTypeNumberLiteral}

	InputFunctionNoSpaces  = []string{"fn main()->{\n", "\treturn\n", "}\n"}
	OutputFunctionNoSpaces = [...]ast.TokenType{ast.TokenTypeFn, ast.TokenTypeIdentifier, ast.TokenTypeOpenParen, ast.TokenTypeCloseParen, ast.TokenTypeArrow, ast.TokenTypeOpenCurlyBracket, ast.TokenTypeReturn, ast.TokenTypeCloseCurlyBracket}
)

func TestArithmeticNoSpaces(t *testing.T) {
	lexer.Tokenize(InputArithmeticNoSpaces)
	CheckExpectedNumberOfTokens(t, 9, len(lexer.Tokens))
	CheckTokenTypes(t, OutputArithmeticNoSpaces[:], lexer.Tokens)
}

func TestFunctionNoSpaces(t *testing.T) {
	lexer.Tokenize(InputFunctionNoSpaces)
	CheckExpectedNumberOfTokens(t, 8, len(lexer.Tokens))
	CheckTokenTypes(t, OutputFunctionNoSpaces[:], lexer.Tokens)
}

func CheckExpectedNumberOfTokens(t *testing.T, expected int, got int) {
	if expected != got {
		t.Errorf("Expected %d tokens but got %d", expected, got)
	}
}

func CheckTokenTypes(t *testing.T, expected []ast.TokenType, got []ast.Token) {
	for i, v := range got {
		if v.TokenType != expected[i] {
			t.Errorf("Expected %d'th token to have type %d but got type %d", i, expected[i], v.TokenType)
		}
	}
}
