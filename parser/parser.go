package parser

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/utils"
)

type Parser struct {
	Nodes                []ast.Node
	Tokens               []ast.Token
	CurTok               ast.Token
	TokIndex             int
	OpPrecedence         map[string]int
	KnownIdentifierTypes map[string]*ast.TypeDecl
	CurType              ast.Expr
	CurFunc              *ast.FuncDecl
}

func (p *Parser) Init(tokens []ast.Token) {
	p.Tokens = tokens
	p.CurTok = p.Tokens[0]
	p.TokIndex = 0
	p.OpPrecedence = map[string]int{
		"=":  2,
		"&&": 3,
		"||": 5,
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
	p.KnownIdentifierTypes = make(map[string]*ast.TypeDecl)
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
	precedence := p.OpPrecedence[tok.Value]
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

func (p *Parser) ParseType() (ast.Expr, error) {

	// TODO: should i refactor this to a seperate function?
	if p.CurTok.TokenType == ast.TokenTypeIdentifier {
		if tyDecl, ok := p.KnownIdentifierTypes[p.CurTok.Value]; ok {
			p.EatToken()
			if p.CurTok.TokenType != ast.TokenTypeAsterisk {
				p.CurType = tyDecl
				return tyDecl, nil
			}

			pointerType := ast.PointerTypeExpr{PointerToType: tyDecl, Pos: p.CurTok.Pos}
			p.EatToken()
			for p.CurTok.TokenType == ast.TokenTypeAsterisk {
				pointerType.PointerToType = pointerType
				pointerType.Pos = p.CurTok.Pos
				p.EatToken()
			}
			p.CurType = pointerType
			return pointerType, nil
		}
	}

	switch p.CurTok.TokenType {
	default:
		return ast.EmptyExpr{}, fmt.Errorf("could not parse type: %s", p.CurTok.Value)
	case ast.TokenTypeI64, ast.TokenTypeU64, ast.TokenTypeI32, ast.TokenTypeU32, ast.TokenTypeI16, ast.TokenTypeU16, ast.TokenTypeI8, ast.TokenTypeU8, ast.TokenTypeF64, ast.TokenTypeF32, ast.TokenTypeBool, ast.TokenTypeString:
		ty := p.ParsePrimitiveType()
		p.CurType = ty
		return ty, nil
	case ast.TokenTypeInterface:
		interfaceTy, err := p.ParseInterfaceType()
		if err != nil {
			return interfaceTy, fmt.Errorf("could not parse interface type: %s", err.Error())
		}
		p.CurType = interfaceTy
		return interfaceTy, nil
	case ast.TokenTypeStruct:
		structTy, err := p.ParseStructType()
		if err != nil {
			return structTy, fmt.Errorf("could not parse struct type: %s", err.Error())
		}
		p.CurType = structTy
		return structTy, nil
	}
}

func (p *Parser) ParseStructType() (ast.StructTypeExpr, error) {
	var structType ast.StructTypeExpr

	structPos := p.CurTok.Pos
	p.EatToken()

	p.Expect(ast.TokenTypeOpenCurlyBracket, "expected '{' in struct type declaration")
	propertiesStartPos := p.CurTok.Pos
	p.EatToken()

	properties, err := p.ParsePropertyList()
	if err != nil {
		return structType, fmt.Errorf("could not parse property list: %s", err.Error())
	}
	properties.Start = propertiesStartPos

	structType.StructPos = structPos
	structType.Properties = properties
	return structType, nil
}

func (p *Parser) ParsePropertyList() (ast.PropertyList, error) {
	var properties ast.PropertyList
	for p.CurTok.TokenType != ast.TokenTypeCloseCurlyBracket {
		property, err := p.ParseProperty()
		if err != nil {
			return properties, fmt.Errorf("could not parse property: %s", err.Error())
		}
		properties.Properties = append(properties.Properties, property)
	}

	endPos := p.CurTok.Pos
	p.EatToken()
	properties.End = endPos
	return properties, nil
}

func (p *Parser) ParseProperty() (ast.Property, error) {
	var property ast.Property

	if p.CurTok.TokenType == ast.TokenTypePub {
		property.Pub = true
		p.EatToken()
	}
	if p.CurTok.TokenType == ast.TokenTypeMut {
		property.Mut = true
		p.EatToken()
	} else {
		p.Expect(ast.TokenTypeConst, "expected either 'mut' or 'const' in property")
		property.Mut = false
		p.EatToken()
	}

	propertyType, err := p.ParseType()
	if err != nil {
		return property, fmt.Errorf("could not parse property type: %s", err.Error())
	}
	property.Type = propertyType

	names := p.ParseIdentList()
	property.Names = names

	return property, nil
}

func (p *Parser) ParseInterfaceType() (ast.InterfaceTypeExpr, error) {
	var interfaceType ast.InterfaceTypeExpr
	interfaceType.InterfacePos = p.CurTok.Pos
	p.EatToken()

	p.Expect(ast.TokenTypeOpenCurlyBracket, "expected '{' in interface type declaration")
	methodsStart := p.CurTok.Pos
	p.EatToken()

	methods, err := p.ParseMethodList()
	methods.Start = methodsStart
	if err != nil {
		return interfaceType, fmt.Errorf("could not parse method list: %s", err.Error())
	}

	interfaceType.Methods = methods
	return interfaceType, nil
}

func (p *Parser) ParseMethodList() (ast.MethodList, error) {
	var methods ast.MethodList
	for p.CurTok.TokenType != ast.TokenTypeCloseCurlyBracket {
		method, err := p.ParseMethod()
		if err != nil {
			return methods, fmt.Errorf("could not parse method: %s", err.Error())
		}
		methods.Methods = append(methods.Methods, method)
	}

	methods.End = p.CurTok.Pos
	p.EatToken()

	return methods, nil
}

func (p *Parser) ParseMethod() (ast.Method, error) {
	var method ast.Method
	p.Expect(ast.TokenTypeIdentifier, "expected identifier in method declaration")
	method.Name = p.CurTok.Value
	p.EatToken()

	p.Expect(ast.TokenTypeOpenParen, "expected '(' before parameter list in method declaration")
	p.EatToken()

	params, err := p.ParseParamList()
	if err != nil {
		return method, fmt.Errorf("could not parse parameter list: %s", err.Error())
	}
	method.Params = params

	if p.CurTok.TokenType == ast.TokenTypeArrow {
		p.EatToken()
		retType, err := p.ParseType()
		if err != nil {
			return method, fmt.Errorf("could not parse return type: %s", err.Error())
		}
		method.Return = retType
	}
	return method, nil
}

func (p *Parser) ParsePrimitiveType() ast.Expr {
	var primitiveType ast.PrimitiveTypeExpr
	primitiveType.PrimitiveType = p.CurTok.TokenType
	primitiveType.Pos = p.CurTok.Pos
	p.EatToken()

	if p.CurTok.TokenType != ast.TokenTypeAsterisk {
		return primitiveType
	}

	pointerType := ast.PointerTypeExpr{PointerToType: primitiveType, Pos: primitiveType.Pos}
	p.EatToken()
	for p.CurTok.TokenType == ast.TokenTypeAsterisk {
		pointerType.PointerToType = pointerType
		pointerType.Pos = p.CurTok.Pos
		p.EatToken()
	}
	return pointerType
}

func (p *Parser) ParsePackageClause() (ast.PackageClause, error) {
	p.EatToken()
	p.Expect(ast.TokenTypeIdentifier, "expected identifier following 'package'")
	name := p.CurTok.Value
	p.EatToken()
	return ast.PackageClause{Name: name}, nil
}
