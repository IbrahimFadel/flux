package ast

type NodeType int

// Maybe in the future populate with data...
type Node interface{}

// Maybe the inheritance is confusing since they're all literally the same...

// Stmt := Decl | ReturnStmt | IfStmt | ...
// Decl := MutDecl | ConstDecl | TypeDecl(maybe)

type Expr interface {
	Node
}

type Stmt interface {
	Node
}

type Decl interface {
	Node
}

type (
	Param struct {
		Mut  bool
		Type string
		Name string
	}

	ParamList struct {
		Start  TokenPos
		Params []Param
		End    TokenPos
	}
)

type (
	IdentifierExpr struct {
		NamePos TokenPos
		Name    string
	}

	NumberLitExpr struct {
		ValuePos TokenPos
		Value    string
	}

	StringLitExpr struct {
		ValuePos TokenPos
		Value    string
	}

	BinOpExpr struct {
		X     Expr
		OpPos TokenPos
		Op    TokenType
		Y     Expr
	}

	UnaryOpExpr struct {
		OpPos TokenPos
		Op    TokenType
		X     Expr
	}

	CallExpr struct {
		Fn            Expr
		OpenParenPos  TokenPos
		Args          []Expr
		EllipsisPos   TokenPos //Potentially implement
		CloseParenPos TokenPos
	}

	NullExpr struct {
		Pos TokenPos
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

	AssignStmt struct {
		Left  []MutDecl // If it's being assigned, it must be a Mutable
		Right []Expr
	}

	BlockStmt struct {
		Start TokenPos
		List  []Stmt
		End   TokenPos
	}

	ReturnStmt struct {
		ReturnPos TokenPos
		Value     Expr
	}
)

type (
	FuncType struct {
		FuncPos TokenPos
		Params  ParamList
		Return  string
	}

	FuncDec struct {
		Name string
		Type FuncType
		Body BlockStmt
	}
)

type (
	MutDecl struct {
		Type string
		Name string
	}

	ConstDecl struct {
	}

	TypeDecl struct {
	}
)
