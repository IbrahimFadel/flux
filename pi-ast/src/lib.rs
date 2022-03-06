use std::fmt;

use pi_lexer::token::TokenKind;
use smol_str::SmolStr;

mod print;

#[derive(Debug)]
pub struct AST {
	pub top_level_declarations: Vec<Decl>,
	// pub functions: Vec<FnDecl>,
}

impl AST {
	pub fn new(top_level_declarations: Vec<Decl>) -> AST {
		AST {
			top_level_declarations,
		}
	}
}

#[derive(Debug, PartialEq)]

pub enum Decl {
	FnDecl(FnDecl),
	TypeDecl(TypeDecl),
	Error,
}

#[derive(Debug, PartialEq)]
pub struct FnDecl {
	pub name: SmolStr,
	pub generics: GenericTypes,
	pub params: Vec<FnParam>,
	pub ret_ty: Expr,
	pub block: BlockStmt,
}

impl FnDecl {
	pub fn new(
		name: SmolStr,
		generics: GenericTypes,
		params: Vec<FnParam>,
		ret_ty: Expr,
		block: BlockStmt,
	) -> FnDecl {
		FnDecl {
			name,
			generics,
			params,
			ret_ty,
			block,
		}
	}
}

impl fmt::Display for FnDecl {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#?}", self)
	}
}

#[derive(Debug, PartialEq)]
pub struct FnParam {
	pub mut_: bool,
	pub type_: Expr,
	pub name: SmolStr,
}

impl FnParam {
	pub fn new(mut_: bool, type_: Expr, name: SmolStr) -> FnParam {
		FnParam { mut_, type_, name }
	}
}

#[derive(Debug, PartialEq)]
pub struct TypeDecl {}

#[derive(Debug, PartialEq, Clone)]
pub enum OpKind {
	Plus,
	Minus,
	Asterisk,
	Slash,
	CmpLT,
	CmpLTE,
	CmpGT,
	CmpGTE,
	CmpEQ,
	CmpNE,
	CmpAnd,
	CmpOr,
	Doublecolon,
	Illegal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
	Ident(Ident),
	BinOp(BinOp),
	IntLit(IntLit),
	FloatLit(FloatLit),
	CharLit(CharLit),
	StringLit(StringLit),
	BoolLit(BoolLit),
	PrimitiveType(PrimitiveType),
	CallExpr(CallExpr),
	Paren(Box<Expr>),
	Unary(Unary),
	Void,
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
	op: TokenKind,
	val: Box<Expr>,
}

impl Unary {
	pub fn new(op: TokenKind, val: Box<Expr>) -> Unary {
		Unary { op, val }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpr {
	callee: Box<Expr>,
	args: Vec<Box<Expr>>,
}

impl CallExpr {
	pub fn new(callee: Box<Expr>, args: Vec<Box<Expr>>) -> CallExpr {
		CallExpr { callee, args }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinOp {
	x: Box<Expr>,
	op: OpKind,
	y: Box<Expr>,
}

impl BinOp {
	pub fn new(x: Box<Expr>, op: OpKind, y: Box<Expr>) -> BinOp {
		BinOp { x, op, y }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveKind {
	I64,
	U64,
	I32,
	U32,
	I16,
	U16,
	I8,
	U8,
	F64,
	F32,
	Bool,
	Void,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveType {
	pub kind: PrimitiveKind,
}

impl PrimitiveType {
	pub fn new(kind: PrimitiveKind) -> PrimitiveType {
		PrimitiveType { kind }
	}
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
	BlockStmt(BlockStmt),
	VarDecl(VarDecl),
	If(If),
	For(For),
	ExprStmt(Expr),
	Return(Return),
	Error,
}

#[derive(Debug, PartialEq)]
pub struct VarDecl {
	pub type_: Expr,
	pub names: Vec<Ident>,
	pub values: Option<Vec<Expr>>,
}

impl VarDecl {
	pub fn new(type_: Expr, names: Vec<Ident>, values: Option<Vec<Expr>>) -> VarDecl {
		VarDecl {
			type_,
			names,
			values,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct If {
	pub condition: Box<Expr>,
	pub then: BlockStmt,
	pub else_: Option<BlockStmt>,
}

impl If {
	pub fn new(condition: Box<Expr>, then: BlockStmt, else_: Option<BlockStmt>) -> If {
		If {
			condition,
			then,
			else_,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct For {}

impl For {
	pub fn new() -> For {
		For {}
	}
}

#[derive(Debug, PartialEq)]
pub struct Return {
	val: Option<Expr>,
}

impl Return {
	pub fn new(val: Option<Expr>) -> Return {
		Return { val }
	}
}

pub type Ident = SmolStr;
pub type IntLit = i128;
pub type FloatLit = f64;
pub type CharLit = char;
pub type StringLit = SmolStr;
pub type BoolLit = bool;
pub type GenericTypes = Vec<Ident>;
pub type BlockStmt = Vec<Stmt>;
