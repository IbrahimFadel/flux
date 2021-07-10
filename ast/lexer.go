package ast

import (
	"strings"
	"unicode"

	"github.com/IbrahimFadel/pi-lang/utils"
)

type TokenType int

const (
	TokenTypeI64 TokenType = iota
	TokenTypeU64
	TokenTypeI32
	TokenTypeU32
	TokenTypeI16
	TokenTypeU16
	TokenTypeI8
	TokenTypeU8
	TokenTypeF64
	TokenTypeF32
	TokenTypeBool
	TokenTypeVoid
	TokenTypeNullptr

	TokenTypeFn
	TokenTypeIf
	TokenTypeFor
	TokenTypeReturn
	TokenTypeImport
	TokenTypePub
	TokenTypeMut
	TokenTypeConst
	TokenTypeWhile
	TokenTypeClass
	TokenTypeConstructor
	TokenTypeNew

	TokenTypeCompareEq
	TokenTypeCompareNe
	TokenTypeCompareLt
	TokenTypeCompareGt
	TokenTypeCompareGtEq
	TokenTypeCompareLtEq
	TokenTypeAnd
	TokenTypeOr

	TokenTypeColon
	TokenTypeSemicolon
	TokenTypeComma
	TokenTypePeriod
	TokenTypeOpenParen
	TokenTypeCloseParen
	TokenTypeOpenCurlyBracket
	TokenTypeCloseCurlyBracket
	TokenTypeOpenSquareBracket
	TokenTypeCloseSquareBracket

	TokenTypeEq
	TokenTypeAsterisk
	TokenTypeSlash
	TokenTypePlus
	TokenTypeMinus
	TokenTypeArrow
	TokenTypeAmpersand

	TokenTypeNumberLiteral
	TokenTypeStringLiteral
	TokenTypeTrue
	TokenTypeFalse
	TokenTypeIdentifier

	TokenTypeEOF
)

type LexerState int

const (
	LexerStateNormal LexerState = iota
	LexerStateString
	LexerStateLineComment
	LexerStateBlockComment
)

var singleCharTokens = [...]byte{'(', ')', ';', ',', '+', '*', '/', '-'}

type TokenPos struct {
	Row, Col int
}

type Token struct {
	TokenType TokenType
	Value     string
	Pos       TokenPos
}

type Lexer struct {
	Tokens []Token
	Pos    TokenPos
	State  LexerState
	Line   string

	Token string // Current token (gets chars appended to it as it reads the file, and gets added to Tokens and reset often)
}

func (lexer *Lexer) Tokenize(content []string) {
	lexer.Tokens = nil
	lexer.Pos.Row = 1
	lexer.Pos.Col = 1
	lexer.Token = ""
	lexer.State = LexerStateNormal

	for _, line := range content {
		lexer.Line = line
		for i := 0; i < len(line); i++ {
			c := line[i]
			shouldContinue := lexer.UpdateState(c, &i)
			if shouldContinue {
				continue
			}

			if lexer.State == LexerStateString {
				lexer.Token += string(c)
				continue
			}

			if c == '\n' || c == ' ' {
				lexer.AddTokenIfValid('\x00')
			} else if utils.ContainsByte(singleCharTokens[:], c) {
				// Some single char tokens are special
				// '-' can either be a 'minus' or the beginning of an '->' operator
				if c == '-' {
					if lexer.Line[i+1] == '>' {
						lexer.AddTokenIfValid('\x00')
						lexer.Token += "->"
						lexer.AddTokenIfValid('\x00')
						i++
						continue
					} else if utils.IsNumber(string(lexer.Line[i+1])) {
						lexer.Token += string(c)
						continue
					} else {
						lexer.AddTokenIfValid(c)
						continue
					}
				} else {
					// If it's not a special single char token, just add it normally
					lexer.AddTokenIfValid(c)
					continue
				}
			}

			lexer.Token += string(c)
			lexer.Pos.Col++
		}
		lexer.Pos.Row++
		lexer.Pos.Col = 1
	}

	eofTok := Token{
		Pos:       TokenPos{-1, -1},
		Value:     "EOF",
		TokenType: TokenTypeEOF,
	}
	lexer.Tokens = append(lexer.Tokens, eofTok)
}

func (lexer *Lexer) UpdateState(c byte, i *int) bool {
	switch lexer.State {
	case LexerStateNormal:
		if c == '"' {
			lexer.State = LexerStateString
			return true
		} else if c == '/' && lexer.Line[*i+1] == '/' {
			lexer.State = LexerStateLineComment
			*i++
			return true
		} else if c == '/' && lexer.Line[*i+1] == '*' {
			lexer.State = LexerStateBlockComment
			*i++
			return true
		}
	case LexerStateString:
		if c == '"' && lexer.Line[*i-1] != '\\' {
			lexer.AddTokenIfValid('\x00')
			lexer.State = LexerStateNormal
			return true
		}
	case LexerStateLineComment:
		if c != '\n' {
			return true
		}
		lexer.State = LexerStateNormal
		return true
	case LexerStateBlockComment:
		if c == '*' && lexer.Line[*i+1] == '/' {
			lexer.State = LexerStateNormal
			*i++
			return true
		}
		return true
	}
	return false
}

func (lexer *Lexer) AddTokenIfValid(c byte) {
	// Strip whitespace
	if lexer.State != LexerStateString {
		lexer.Token = strings.Map(func(r rune) rune {
			if unicode.IsSpace(r) {
				return -1
			}
			return r
		}, lexer.Token)
	}

	var tok Token

	switch lexer.Token {
	case "i64":
		tok = lexer.ConstructToken(TokenTypeI64)
	case "u64":
		tok = lexer.ConstructToken(TokenTypeU64)
	case "i32":
		tok = lexer.ConstructToken(TokenTypeI32)
	case "u32":
		tok = lexer.ConstructToken(TokenTypeU32)
	case "i16":
		tok = lexer.ConstructToken(TokenTypeI16)
	case "u16":
		tok = lexer.ConstructToken(TokenTypeU16)
	case "i8":
		tok = lexer.ConstructToken(TokenTypeI8)
	case "u8":
		tok = lexer.ConstructToken(TokenTypeU8)
	case "f64":
		tok = lexer.ConstructToken(TokenTypeF64)
	case "f32":
		tok = lexer.ConstructToken(TokenTypeF32)
	case "bool":
		tok = lexer.ConstructToken(TokenTypeBool)
	case "void":
		tok = lexer.ConstructToken(TokenTypeVoid)
	case "nullptr":
		tok = lexer.ConstructToken(TokenTypeNullptr)

	case "fn":
		tok = lexer.ConstructToken(TokenTypeFn)
	case "if":
		tok = lexer.ConstructToken(TokenTypeIf)
	case "for":
		tok = lexer.ConstructToken(TokenTypeFor)
	case "return":
		tok = lexer.ConstructToken(TokenTypeReturn)
	case "import":
		tok = lexer.ConstructToken(TokenTypeImport)
	case "pub":
		tok = lexer.ConstructToken(TokenTypePub)
	case "mut":
		tok = lexer.ConstructToken(TokenTypeMut)
	case "const":
		tok = lexer.ConstructToken(TokenTypeConst)
	case "while":
		tok = lexer.ConstructToken(TokenTypeWhile)
	case "class":
		tok = lexer.ConstructToken(TokenTypeClass)
	case "constructor":
		tok = lexer.ConstructToken(TokenTypeConstructor)
	case "new":
		tok = lexer.ConstructToken(TokenTypeNew)

	case "==":
		tok = lexer.ConstructToken(TokenTypeCompareEq)
	case "!=":
		tok = lexer.ConstructToken(TokenTypeCompareNe)
	case "<":
		tok = lexer.ConstructToken(TokenTypeCompareLt)
	case ">":
		tok = lexer.ConstructToken(TokenTypeCompareGt)
	case "<=":
		tok = lexer.ConstructToken(TokenTypeCompareLtEq)
	case ">=":
		tok = lexer.ConstructToken(TokenTypeCompareGtEq)
	case "&&":
		tok = lexer.ConstructToken(TokenTypeAnd)
	case "||":
		tok = lexer.ConstructToken(TokenTypeOr)

	case ":":
		tok = lexer.ConstructToken(TokenTypeColon)
	case ";":
		tok = lexer.ConstructToken(TokenTypeSemicolon)
	case ",":
		tok = lexer.ConstructToken(TokenTypeComma)
	case ".":
		tok = lexer.ConstructToken(TokenTypePeriod)
	case "(":
		tok = lexer.ConstructToken(TokenTypeOpenParen)
	case ")":
		tok = lexer.ConstructToken(TokenTypeCloseParen)
	case "{":
		tok = lexer.ConstructToken(TokenTypeOpenCurlyBracket)
	case "}":
		tok = lexer.ConstructToken(TokenTypeCloseCurlyBracket)
	case "[":
		tok = lexer.ConstructToken(TokenTypeOpenSquareBracket)
	case "]":
		tok = lexer.ConstructToken(TokenTypeCloseSquareBracket)

	case "=":
		tok = lexer.ConstructToken(TokenTypeEq)
	case "*":
		tok = lexer.ConstructToken(TokenTypeAsterisk)
	case "/":
		tok = lexer.ConstructToken(TokenTypeSlash)
	case "+":
		tok = lexer.ConstructToken(TokenTypePlus)
	case "-":
		tok = lexer.ConstructToken(TokenTypeMinus)
	case "->":
		tok = lexer.ConstructToken(TokenTypeArrow)
	case "&":
		tok = lexer.ConstructToken(TokenTypeAmpersand)

	case "true":
		tok = lexer.ConstructToken(TokenTypeTrue)
	case "false":
		tok = lexer.ConstructToken(TokenTypeFalse)

	default:
		if lexer.State == LexerStateString {
			tok = lexer.ConstructToken(TokenTypeStringLiteral)
		} else if utils.IsNumber(lexer.Token) {
			tok = lexer.ConstructToken(TokenTypeNumberLiteral)
		} else {
			tok = lexer.ConstructToken(TokenTypeIdentifier)
		}
	}

	// tok &&
	if len(lexer.Token) > 0 {
		lexer.Tokens = append(lexer.Tokens, tok)
		lexer.Token = ""
	}
	if c != '\x00' {
		lexer.Token = string(c)
		lexer.AddTokenIfValid('\x00')
		lexer.Pos.Col++
	}
}

func (lexer *Lexer) ConstructToken(tokenType TokenType) Token {
	var tok Token
	tok.TokenType = tokenType
	tok.Value = lexer.Token
	tok.Pos.Col = lexer.Pos.Col
	tokLen := len(lexer.Token)
	if tokLen > 1 {
		tok.Pos.Col -= tokLen
	}
	tok.Pos.Row = lexer.Pos.Row
	return tok
}
