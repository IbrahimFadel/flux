package ast

type NodeType int

const (
	NodeTypeNullExpr NodeType = iota
	NodeTypeBinaryExpr
	NodeTypeUnaryExpr
	NodeTypeCallExpr
	NodeTypeExprStmt
	NodeTypeNumberLitExpr
	NodeTypeStringLitExpr
	NodeTypeVarRefExpr
	NodeTypeFuncDecl
	NodeTypePackageClause
	NodeTypeTypeDecl
	NodeTypeReturnStmt
	NodeTypeMutDecl
	NodeTypeConstDecl
)

// Stmt := Decl | ReturnStmt | IfStmt | ...
// Decl := MutDecl | ConstDecl | TypeDecl

type Node interface {
	Type() NodeType
}

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
		Pos TokenPos
	}

	VarRefExpr struct {
		Pos  TokenPos
		Name string
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

	FuncDecl struct {
		Name     string
		FuncType FuncType
		Body     BlockStmt
	}
)

type (
	MutDecl struct {
		MutType string
		Names   []string
		Values  []Expr
	}

	ConstDecl struct {
		ConstType string
		Names     []string
		Values    []Expr
	}

	TypeDecl struct {
	}
)

func (nullExpr PackageClause) Type() NodeType {
	return NodeTypePackageClause
}

func (nullExpr TypeDecl) Type() NodeType {
	return NodeTypeTypeDecl
}

func (nullExpr ReturnStmt) Type() NodeType {
	return NodeTypeReturnStmt
}

func (nullExpr MutDecl) Type() NodeType {
	return NodeTypeMutDecl
}

func (nullExpr ConstDecl) Type() NodeType {
	return NodeTypeConstDecl
}

func (nullExpr NullExpr) Type() NodeType {
	return NodeTypeNullExpr
}

func (nullExpr BinaryExpr) Type() NodeType {
	return NodeTypeBinaryExpr
}

func (nullExpr UnaryExpr) Type() NodeType {
	return NodeTypeUnaryExpr
}

func (nullExpr CallExpr) Type() NodeType {
	return NodeTypeCallExpr
}

func (nullExpr ExprStmt) Type() NodeType {
	return NodeTypeExprStmt
}

func (nullExpr NumberLitExpr) Type() NodeType {
	return NodeTypeNumberLitExpr
}

func (nullExpr StringLitExpr) Type() NodeType {
	return NodeTypeStringLitExpr
}

func (nullExpr VarRefExpr) Type() NodeType {
	return NodeTypeVarRefExpr
}

func (nullExpr FuncDecl) Type() NodeType {
	return NodeTypeFuncDecl
}
