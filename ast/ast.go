package ast

import "github.com/llir/llvm/ir/value"

type NumberType int

// const (
// 	I64 NumberType = iota
// 	U64
// 	I32
// 	U32
// 	I16
// 	U16
// 	I8
// 	U8
// 	F64
// 	F32
// 	Bool
// )

// Stmt := Decl | ReturnStmt | IfStmt | ...
// Decl := MutDecl | ConstDecl | TypeDecl

type Node interface{}

type Expr interface {
	Node
}

type Stmt interface {
	Node
}

type Decl interface {
	Node
}

// This doesn't really fit into Expr Stmt or Decl...
type PackageClause struct {
	Name string
}

type (
	Param struct {
		Mut  bool
		Type Expr
		Name string
	}

	ParamList struct {
		Start  TokenPos
		Params []Param
		End    TokenPos
	}

	Method struct {
		Name   string
		Params ParamList
		Return Expr
	}

	MethodList struct {
		Start   TokenPos
		Methods []Method
		End     TokenPos
	}

	Property struct {
		Pub   bool
		Mut   bool
		Type  Expr
		Names []string // Several properties with the same pub/mut/type -- pub mut i32 x, y, z or mut i32 x, y, z, etc.
	}

	PropertyList struct {
		Start      TokenPos
		Properties []Property
		End        TokenPos
	}
)

type (
	EmptyExpr struct {
	}

	IdentifierExpr struct {
		NamePos TokenPos
		Name    string
	}

	NumberLitExpr struct {
		ValuePos TokenPos
		Type     Expr
		Value    string
	}

	StringLitExpr struct {
		ValuePos TokenPos
		Value    string
	}

	BinaryExpr struct {
		X     Expr
		OpPos TokenPos
		Op    TokenType
		Y     Expr
	}

	UnaryExpr struct {
		OpPos TokenPos
		Op    TokenType
		X     Expr
	}

	CallExpr struct {
		Fn            Expr
		OpenParenPos  TokenPos
		Args          []Expr
		CloseParenPos TokenPos
	}

	NullExpr struct {
		Pos  TokenPos
		Type Expr
	}

	VoidExpr struct {
		Pos TokenPos
	}

	VarRefExpr struct {
		Pos  TokenPos
		Name string
	}

	PrimitiveTypeExpr struct {
		PrimitiveType TokenType
		Pos           TokenPos
	}

	PointerTypeExpr struct {
		PointerToType Expr // What is it a pointer of?
		Pos           TokenPos
	}

	InterfaceTypeExpr struct {
		InterfacePos TokenPos
		Methods      MethodList
	}

	StructTypeExpr struct {
		StructPos  TokenPos
		Properties PropertyList
		Name       string
	}
)

type (
	DeclStmt struct {
		Decl Decl
	}

	EmptyStmt struct { // use for for loops?
		SemicolonPos TokenPos
	}

	ExprStmt struct {
		X Expr
	}

	BlockStmt struct {
		Start     TokenPos
		Name      string
		List      []Stmt
		End       TokenPos
		Constants map[string]value.Value
		Mutables  map[string]value.Value
	}

	ReturnStmt struct {
		ReturnPos TokenPos
		Type      Expr
		Value     Expr
	}
)

type (
	FuncReceiver struct {
		Pos  TokenPos
		Name string
		Type Expr
	}

	FuncType struct {
		FuncPos TokenPos
		Params  ParamList
		Return  Expr
	}

	FuncDecl struct {
		Receiver FuncReceiver
		Name     string
		FuncType FuncType
		Body     BlockStmt
	}
)

type (
	VarDecl struct {
		Mut    bool
		Type   Expr
		Names  []string
		Values []Expr
	}

	TypeDecl struct {
		Name string
		Type Expr
	}
)
